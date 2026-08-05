# Plugins

The plugin system enables third-party extensions for grammar checking, citations, diagrams, integrations, and custom workflows. This spec defines the SDK surface; implementation is deferred to Phase 6.

## Design Principles

1. **Sandboxed by default.** Plugins cannot access the filesystem, network, or document content beyond their declared capabilities.
2. **Capability-based permissions.** Plugins declare what they need; users approve on install.
3. **Language-agnostic SDK.** Plugin authors can use Rust, Python, or JavaScript.
4. **Non-blocking.** Plugin operations run asynchronously; they never block the UI thread or editing.
5. **Document-safe.** Plugins mutate documents only through the `Command` API, making all plugin edits undoable.

## Architecture

```
Plugin Marketplace
        │
        ▼
  Plugin Manager (tw-plugin)
  ├── Lifecycle (install, enable, disable, uninstall)
  ├── Sandbox (WASM or process isolation)
  ├── Capability enforcement
  └── Message routing
        │
        ▼
  Plugin Runtime
  ├── WASM runtime (wasmtime) — default
  ├── Python runtime (PyO3) — enterprise
  └── Node.js runtime — enterprise
        │
        ▼
  Plugin SDK
  ├── Document API (read content, apply edits)
  ├── UI API (sidebar panels, context menu items)
  ├── Event API (subscribe to document events)
  └── Storage API (plugin-local key-value store)
```

## Plugin Manifest

```json
{
  "id": "com.example.grammar-check",
  "name": "Grammar Check Pro",
  "version": "1.0.0",
  "author": "Example Corp",
  "description": "Advanced grammar and style checking",
  "runtime": "wasm",
  "entry": "plugin.wasm",
  "capabilities": [
    "document.read",
    "document.suggest",
    "ui.sidebar",
    "ui.context_menu"
  ],
  "permissions": {
    "network": ["api.grammarcheck.com"],
    "storage": true
  },
  "min_app_version": "1.0.0"
}
```

## Capabilities

| Capability | Description | Risk Level |
|------------|-------------|------------|
| `document.read` | Read document content and structure | Low |
| `document.edit` | Apply edits via Command API | Medium |
| `document.suggest` | Show suggestions (user accepts/rejects) | Low |
| `ui.sidebar` | Register a sidebar panel | Low |
| `ui.context_menu` | Add context menu items | Low |
| `ui.toolbar` | Add toolbar buttons | Low |
| `ui.dialog` | Show modal dialogs | Low |
| `events.document` | Subscribe to edit/save/open events | Low |
| `events.selection` | Subscribe to selection changes | Low |
| `network` | Make HTTP requests to declared hosts | High |
| `storage` | Plugin-local key-value storage | Low |
| `filesystem.read` | Read files from declared paths | High |
| `ai.provider` | Register as an AI provider | Medium |

Users approve capabilities on install. Enterprise admins can pre-approve or block capabilities via policy.

## Plugin SDK

### Document API

```rust
pub trait PluginDocument {
    fn get_text(&self, range: Option<DocRange>) -> String;
    fn get_paragraph_count(&self) -> u32;
    fn get_paragraph_text(&self, index: u32) -> Option<String>;
    fn get_selection(&self) -> Option<DocRange>;
    fn apply_edit(&self, command: Command) -> Result<(), PluginError>;
    fn suggest_edit(&self, suggestion: Suggestion) -> Result<(), PluginError>;
}
```

Plugins never access the raw document model. All access goes through this API, which enforces capability checks.

### UI API

```rust
pub trait PluginUi {
    fn register_sidebar_panel(&self, panel: SidebarPanel) -> PanelId;
    fn register_context_menu_item(&self, item: ContextMenuItem) -> MenuItemId;
    fn register_toolbar_button(&self, button: ToolbarButton) -> ButtonId;
    fn show_dialog(&self, dialog: DialogRequest) -> Result<DialogResponse, PluginError>;
    fn notify(&self, message: &str, level: NotificationLevel);
}
```

Flutter renders plugin UI elements natively. Plugins send UI definitions (JSON); Flutter creates the widgets.

### Event API

```rust
pub trait PluginEvents {
    fn on_document_edit(&self, callback: Box<dyn Fn(EditEvent)>);
    fn on_selection_change(&self, callback: Box<dyn Fn(SelectionEvent)>);
    fn on_document_save(&self, callback: Box<dyn Fn(SaveEvent)>);
    fn on_document_open(&self, callback: Box<dyn Fn(OpenEvent)>);
}
```

### Storage API

```rust
pub trait PluginStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), PluginError>;
    fn delete(&self, key: &str) -> Result<(), PluginError>;
    fn list_keys(&self) -> Vec<String>;
}
```

Plugin-local storage is sandboxed. Plugins cannot access other plugins' storage or application storage.

## Sandbox Models

| Model | Isolation | Performance | Use Case |
|-------|-----------|-------------|----------|
| WASM (default) | Full sandbox — no filesystem, network, or memory access beyond declared capabilities | Near-native | All marketplace plugins |
| Native process | OS process isolation — separate memory space | Native speed | Enterprise-approved plugins |
| In-process (dev only) | No isolation | Fastest | Development and debugging |

WASM plugins are compiled to `wasm32-wasi` and run in `wasmtime`. Capability enforcement is at the host level — WASM modules cannot syscalls beyond their permissions.

## Plugin Lifecycle

```
Install → Verify signature → Extract → Register capabilities
  → Enable → Activate (load runtime, call on_activate)
    → Running (handle events, respond to API calls)
  → Disable → Deactivate (call on_deactivate, unload runtime)
→ Uninstall → Remove files, clear storage
```

```rust
pub trait Plugin {
    fn on_activate(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn on_deactivate(&self) -> Result<(), PluginError>;
    fn on_command(&self, command: &str, args: &serde_json::Value) -> Result<serde_json::Value, PluginError>;
}
```

## Built-in Plugin Targets

These are the first-party plugins planned for the marketplace:

| Plugin | Capabilities | Phase |
|--------|-------------|-------|
| Mermaid diagrams | `document.read`, `document.edit`, `ui.dialog` | 6 |
| Draw.io integration | `document.read`, `document.edit`, `ui.dialog`, `network` | 6 |
| Zotero citations | `document.read`, `document.edit`, `network` | 6 |
| LaTeX equations | `document.read`, `document.edit`, `ui.dialog` | 4 |
| Git versioning | `document.read`, `events.document`, `filesystem.read` | 6 |
| Google Drive sync | `network`, `filesystem.read` | 6 |
| DeepL translation | `document.read`, `document.suggest`, `network` | 4 |

## Plugin Marketplace

```
REST /api/v1/marketplace
  GET    /plugins              List available plugins
  GET    /plugins/{id}         Plugin details + reviews
  POST   /plugins/{id}/install Download and install
  GET    /plugins/{id}/updates Check for updates
  POST   /plugins              Submit new plugin (developer)
```

Marketplace is hosted by the product team. Enterprise deployments can run a private marketplace or sideload plugins directly.

## Developer SDK Distribution

| Language | Package | Runtime |
|----------|---------|---------|
| Rust | `tw-plugin-sdk` crate | WASM |
| Python | `tutuaword-plugin` PyPI package | Native process |
| JavaScript | `@tutuaword/plugin-sdk` npm package | WASM or Node.js |

SDK includes:
- Type definitions for all API surfaces
- Plugin manifest schema and validator
- Local development server (hot reload)
- Test harness (mock document, mock UI)
- Publishing CLI (`tw-plugin publish`)

## Security Considerations

- All marketplace plugins must be signed by the marketplace key
- Enterprise admins can require additional signing (org key)
- WASM plugins cannot access memory outside their linear memory
- Network access is restricted to declared hosts (host-level proxy)
- Plugin updates require re-approval of capabilities if permissions changed
- Audit log records all plugin actions (install, enable, API calls)

See [security.md](security.md) for the full security model.
