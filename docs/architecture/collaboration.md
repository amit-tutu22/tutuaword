# Collaboration

Real-time multi-user editing powered by CRDTs. This spec defines the interface; implementation is deferred to Phase 5.

## Design Principles

1. **Offline-first.** Collaboration enhances the offline experience; it never blocks offline editing.
2. **Eventual consistency.** All replicas converge to the same document state without conflicts.
3. **Single mutation path.** CRDT operations translate to the same `Command` enum used by local editing (ADR-0007).
4. **Presence without persistence.** Live cursors and selections are ephemeral — not stored in the document.
5. **Comments are first-class.** Comments are document content, included in CRDT sync.

## Architecture

```
User A (local)                    User B (remote)
     │                                  │
     ▼                                  ▼
  tw-edit::apply()               tw-edit::apply()
     │                                  │
     ▼                                  ▼
  tw-crdt::translate()           tw-crdt::translate()
     │                                  │
     ▼                                  ▼
  Yjs Document                   Yjs Document
     │                                  │
     ▼                                  ▼
  Sync Engine ──── WebSocket ──── Sync Engine
                       │
                       ▼
                 Sync Server
                 (self-hosted or cloud)
```

## CRDT Selection: Yjs

See [ADR-0009](../adr/0009-crdt-selection.md) for the full decision record.

Yjs is selected because:
- Mature, battle-tested (used by Notion, Figma, JupyterLab)
- Efficient binary encoding (lib0)
- Built-in awareness protocol (cursors, presence)
- Rust bindings available (`yrs` crate)
- Offline support with merge-on-reconnect

## Operation Translation

Local edits and CRDT operations share the same mutation path:

```rust
pub trait CrdtTranslator {
    fn local_to_crdt(&self, command: &Command) -> Vec<CrdtOperation>;
    fn crdt_to_local(&self, ops: &[CrdtOperation]) -> Vec<Command>;
}

pub enum CrdtOperation {
    Insert { node_id: NodeId, offset: u32, text: String, attrs: YMap },
    Delete { node_id: NodeId, offset: u32, length: u32 },
    SetAttr { node_id: NodeId, key: String, value: YValue },
    InsertNode { parent_id: NodeId, index: u32, node: YNode },
    DeleteNode { node_id: NodeId },
    MoveNode { node_id: NodeId, new_parent: NodeId, new_index: u32 },
}
```

When a local edit occurs:
1. `tw-edit::apply(command)` mutates the model
2. `tw-crdt::local_to_crdt(command)` generates CRDT operations
3. CRDT operations are sent to the sync engine

When a remote edit arrives:
1. Sync engine delivers CRDT operations
2. `tw-crdt::crdt_to_local(ops)` generates Commands
3. `tw-edit::apply(command)` mutates the model (without generating new CRDT ops)

## Sync Engine

```rust
pub struct SyncEngine {
    doc: YDoc,
    provider: Box<dyn SyncProvider>,
    offline_queue: Vec<CrdtOperation>,
    connected: bool,
}

pub trait SyncProvider: Send + Sync {
    fn connect(&mut self, room_id: &str) -> Result<(), SyncError>;
    fn disconnect(&mut self);
    fn send(&self, update: &[u8]) -> Result<(), SyncError>;
    fn on_update(&self, callback: Box<dyn Fn(&[u8]) + Send>) -> Subscription;
    fn on_awareness(&self, callback: Box<dyn Fn(&[u8]) + Send>) -> Subscription;
}
```

### Sync Providers

| Provider | Transport | Use Case |
|----------|-----------|----------|
| WebSocket | `wss://` | Real-time cloud sync |
| WebRTC | P2P | Direct peer sync (no server) |
| File-based | Shared folder | Offline-first team sync |
| Custom | User-configured | Enterprise self-hosted |

### Offline Sync

When offline:
1. Local edits generate CRDT operations as normal
2. Operations are queued in `offline_queue`
3. On reconnect, queued operations are sent in batch
4. Remote operations received during offline period are merged
5. CRDT guarantees convergence — no conflict resolution UI needed

## Presence

Live cursors and selections use Yjs Awareness protocol:

```rust
pub struct PresenceState {
    pub user: UserInfo,
    pub cursor: Option<RemoteCursor>,
    pub selection: Option<RemoteSelection>,
    pub viewport: ViewportInfo,
    pub last_active: DateTime<Utc>,
}

pub struct RemoteCursor {
    pub page: PageIndex,
    pub x: f32,
    pub y: f32,
    pub color: Color,
}

pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub color: Color,  // assigned per-user for cursor/selection
}
```

Presence data is ephemeral — transmitted via Awareness protocol, not stored in the document or CRDT.

Flutter renders remote cursors as colored carets and selections as colored overlays on the document canvas.

## Comments

Comments are document content, synced via CRDT:

```rust
pub struct CommentThread {
    pub id: NodeId,
    pub anchor: CommentAnchor,
    pub messages: Vec<CommentMessage>,
    pub resolved: bool,
    pub participants: Vec<String>,
}

pub struct CommentMessage {
    pub id: NodeId,
    pub author: UserInfo,
    pub timestamp: DateTime<Utc>,
    pub body: Vec<Block>,
    pub mentions: Vec<String>,
}
```

Comments appear in the sidebar and as margin markers in the document view. Threaded replies are supported.

## Suggestions (Track Changes + Collaboration)

Suggestions combine track changes with collaborative review:

| Action | Behavior |
|--------|----------|
| User edits in suggestion mode | Changes marked as suggestions (not applied) |
| Document owner reviews | Accept (apply) or reject (discard) each suggestion |
| Multiple suggesters | Each suggestion tagged with author |
| Accept all / reject all | Batch operations |

Suggestions use the existing `Revision` metadata on nodes (see [document-model.md](document-model.md)).

## Version History

```rust
pub struct VersionEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub author: UserInfo,
    pub description: Option<String>,
    pub snapshot: DocumentSnapshot,
    pub crdt_state: Vec<u8>,
}

pub struct VersionHistory {
    pub entries: Vec<VersionEntry>,
    pub auto_save_interval: Duration,
    pub max_versions: u32,
}
```

Version snapshots are created:
- Automatically every N minutes (configurable)
- On explicit "Save version" action
- Before accepting/rejecting suggestions
- Before major AI operations

Diff view compares any two versions, highlighting insertions, deletions, and formatting changes.

## Team Workspaces

```rust
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub members: Vec<WorkspaceMember>,
    pub documents: Vec<DocumentRef>,
    pub settings: WorkspaceSettings,
}

pub struct WorkspaceMember {
    pub user: UserInfo,
    pub role: WorkspaceRole,
    pub joined_at: DateTime<Utc>,
}

pub enum WorkspaceRole {
    Owner,
    Editor,
    Commenter,
    Viewer,
}
```

Role permissions:

| Action | Owner | Editor | Commenter | Viewer |
|--------|-------|--------|-----------|--------|
| Edit document | Yes | Yes | No | No |
| Add comments | Yes | Yes | Yes | No |
| Suggest changes | Yes | Yes | Yes | No |
| Accept/reject suggestions | Yes | Yes | No | No |
| Share document | Yes | No | No | No |
| Manage members | Yes | No | No | No |

## Cloud Backend (Interface Only)

The sync server is a separate service (not part of the Rust engine). Interface:

```
WebSocket /ws/room/{room_id}
  ← client sends: Yjs update (binary)
  → server broadcasts: Yjs update (binary) to other clients in room
  ← client sends: Awareness update (binary)
  → server broadcasts: Awareness update (binary)

REST /api/v1/documents
  POST   /           Create document
  GET    /{id}       Get document metadata
  GET    /{id}/state Get latest CRDT state (for initial sync)
  GET    /{id}/versions  List version history

REST /api/v1/workspaces
  POST   /           Create workspace
  GET    /{id}/members  List members
  POST   /{id}/members  Add member
```

Backend technology choice is deferred (see [vision.md](../vision.md#open-questions)).

## FFI Surface

```dart
// Collaboration
Future<void> joinSession(int docId, String roomId, UserInfo user);
Future<void> leaveSession(int docId);
Stream<PresenceUpdate> presenceStream(int docId);
Stream<CommentThread> commentStream(int docId);

// Comments
Future<void> addComment(int docId, CommentAnchor anchor, String text);
Future<void> replyToComment(int docId, NodeId commentId, String text);
Future<void> resolveComment(int docId, NodeId commentId);

// Version history
Future<List<VersionEntry>> getVersionHistory(int docId);
Future<void> restoreVersion(int docId, String versionId);
Future<DocumentDiff> diffVersions(int docId, String v1, String v2);
```
