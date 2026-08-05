# Security

Security and enterprise compliance features. This spec defines the security model; implementation is deferred to Phase 6 (with foundational decisions applied earlier).

## Design Principles

1. **Defense in depth.** Multiple layers of protection — encryption, access control, audit, sandboxing.
2. **Privacy by default.** Local-first architecture means documents stay on-device unless the user explicitly syncs.
3. **Enterprise control.** Admins can enforce policies that restrict features (cloud AI, plugins, sync).
4. **Transparent audit.** All security-relevant actions are logged and queryable.
5. **Standards compliance.** Target SOC 2 Type II, GDPR, and ISO 27001 readiness.

## Threat Model

| Threat | Mitigation | Phase |
|--------|------------|-------|
| Document data exfiltration via AI | Local AI mode, data classification policies | 4, 6 |
| Malicious plugin access | WASM sandbox, capability enforcement | 6 |
| Man-in-the-middle sync | TLS 1.3, certificate pinning | 5 |
| Unauthorized document access | Encryption at rest, access control | 6 |
| Credential theft | OS keychain integration, no plaintext secrets | 1, 6 |
| Supply chain attack (plugin) | Signature verification, marketplace review | 6 |
| CRDT state tampering | Signed updates, server validation | 5 |
| Memory exposure (crash dump) | Encrypted swap, secure memory allocation for keys | 6 |

## Document Encryption

### At Rest

```rust
pub struct DocumentEncryption {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivation,
    pub key_storage: KeyStorage,
}

pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

pub enum KeyDerivation {
    Pbkdf2 { iterations: u32 },
    Argon2id { memory_kb: u32, iterations: u32 },
}

pub enum KeyStorage {
    OsKeychain,       // macOS Keychain, Windows Credential Manager
    SecureEnclave,    // iOS Secure Enclave, Android Keystore
    UserPassword,     // derived from user password (no keychain)
    EnterpriseHsm,    // enterprise HSM integration
}
```

Documents can be encrypted:
- **User-initiated:** "Protect with password" (like Word's password protection)
- **Policy-enforced:** Enterprise policy requires encryption for all documents
- **Classification-based:** Documents tagged "Confidential" are auto-encrypted

Encrypted `.twdoc` and `.docx` files store encrypted content with metadata (algorithm, KDF params) in the file header. The encryption key never appears in the file.

### In Transit

All network communication uses TLS 1.3:
- Cloud sync: `wss://` WebSocket connections
- AI providers: HTTPS with certificate pinning
- Plugin network access: proxied through host with TLS inspection
- Marketplace: HTTPS only

### In Memory

- Encryption keys are allocated in secure memory (mlocked, zeroed on free)
- Document content in the Rust model is not encrypted in memory (performance requirement)
- Future: optional full-memory encryption for classified documents (significant performance cost)

## Authentication and Identity

### Single Sign-On (SSO)

```rust
pub trait IdentityProvider: Send + Sync {
    fn authenticate(&self, config: &SsoConfig) -> Result<AuthToken, AuthError>;
    fn refresh(&self, token: &AuthToken) -> Result<AuthToken, AuthError>;
    fn get_user_info(&self, token: &AuthToken) -> Result<UserInfo, AuthError>;
    fn logout(&self, token: &AuthToken) -> Result<(), AuthError>;
}

pub enum SsoProtocol {
    Saml2 { idp_metadata_url: String },
    Oidc { issuer: String, client_id: String },
}
```

Supported identity providers:
- Azure AD / Entra ID
- Okta
- Google Workspace
- Generic SAML 2.0 / OIDC

### Local Authentication

For offline/personal use:
- No authentication required (local-only mode)
- Optional app password / biometric lock (Face ID, Touch ID, Windows Hello)
- Biometric unlock uses OS secure APIs — no biometric data stored by the app

## Access Control

### Document-Level

```rust
pub enum DocumentPermission {
    Read,
    Comment,
    Edit,
    Admin,
}

pub struct AccessControlEntry {
    pub subject: AccessSubject,
    pub permission: DocumentPermission,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,
}

pub enum AccessSubject {
    User(String),
    Group(String),
    Role(WorkspaceRole),
    Public,
}
```

### Enterprise Policies

```rust
pub struct EnterprisePolicy {
    pub require_sso: bool,
    pub require_encryption: bool,
    pub allow_cloud_ai: bool,
    pub allow_local_ai: bool,
    pub allow_plugins: bool,
    pub allowed_plugin_capabilities: Vec<String>,
    pub allow_cloud_sync: bool,
    pub data_residency: Option<DataRegion>,
    pub audit_level: AuditLevel,
    pub max_document_size_mb: u32,
    pub password_policy: PasswordPolicy,
    pub session_timeout: Duration,
}
```

Policies are pushed from the admin console and enforced locally (offline enforcement).

## Audit Logging

```rust
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: AuditActor,
    pub action: AuditAction,
    pub resource: AuditResource,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub device_id: Option<String>,
    pub outcome: AuditOutcome,
}

pub enum AuditAction {
    DocumentOpen,
    DocumentEdit,
    DocumentSave,
    DocumentExport,
    DocumentShare,
    DocumentDelete,
    AiRequest,
    PluginActivate,
    PluginApiCall,
    Login,
    Logout,
    PolicyChange,
    AdminAction,
}
```

Audit logs are:
- Stored locally (encrypted)
- Synced to enterprise audit server (if configured)
- Retained per policy (default: 1 year)
- Queryable via admin console API
- Exportable for compliance reporting

## Data Classification

```rust
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

pub struct ClassificationPolicy {
    pub classification: DataClassification,
    pub require_encryption: bool,
    pub allow_cloud_ai: bool,
    pub allow_cloud_sync: bool,
    pub allow_export: bool,
    pub watermark: Option<String>,
}
```

Documents can be tagged with classification levels. Policies automatically enforce restrictions (e.g., "Confidential" documents cannot use cloud AI). `AiPolicy` supports per-provider allow/block lists and routing modes (Always Local, Always Cloud, Automatic) — see [ai-platform.md](ai-platform.md).

## Digital Signatures

```rust
pub struct DigitalSignature {
    pub signer: SignerInfo,
    pub timestamp: DateTime<Utc>,
    pub certificate: X509Certificate,
    pub signature_value: Vec<u8>,
    pub signed_content_hash: Vec<u8>,
}

pub struct SignerInfo {
    pub name: String,
    pub email: String,
    pub organization: Option<String>,
}
```

Document signing:
- Sign the document content hash with the signer's private key
- Embed signature in document metadata (`.twdoc` manifest, DOCX custom XML)
- Verification checks certificate chain and content integrity
- Supports PKCS#7 / CAdES signature formats for DOCX compatibility

## Protected Documents

Password-protected documents (compatible with Word's password protection):

```rust
pub struct DocumentProtection {
    pub password_hash: Vec<u8>,
    pub kdf_params: KeyDerivation,
    pub permissions: DocumentProtectionPermissions,
}

pub struct DocumentProtectionPermissions {
    pub allow_edit: bool,
    pub allow_print: bool,
    pub allow_copy: bool,
    pub allow_comment: bool,
}
```

Protection is applied at the file level — the entire file is encrypted. Opening requires the password. Permissions restrict what the user can do after opening.

## Compliance Targets

| Standard | Key Requirements | Phase |
|----------|-----------------|-------|
| GDPR | Data minimization, right to deletion, consent, DPA | 6 |
| SOC 2 Type II | Access control, encryption, audit, availability | 6 |
| ISO 27001 | ISMS, risk assessment, incident response | 6+ |
| HIPAA | PHI encryption, access logs, BAA (if healthcare) | 6+ |
| FedRAMP | Government cloud security (if targeting US gov) | Future |

## Secure Development

Applied from Phase 1:

| Practice | Implementation |
|----------|---------------|
| Dependency auditing | `cargo audit` in CI |
| Memory safety | Rust (no unsafe except FFI boundaries) |
| Fuzz testing | `cargo fuzz` on format parsers (Phase 3) |
| Secret scanning | Pre-commit hooks, CI scanning |
| Code signing | Signed releases for all platforms |
| SBOM | Software Bill of Materials generated per release |
| Penetration testing | Annual third-party pentest (Phase 6) |

## Privacy

| Data Type | Stored Where | User Control |
|-----------|-------------|--------------|
| Document content | Local device (default) | Full control |
| Document content (sync) | Cloud storage (opt-in) | Delete anytime |
| AI prompts | Provider's servers (cloud) or local | Opt-in, local alternative |
| Usage telemetry | Product analytics (opt-in) | Disable in settings |
| Audit logs | Local + enterprise server | Admin controlled |
| Crash reports | Error reporting service (opt-in) | Disable in settings |

No document content is sent to product analytics or crash reporting. AI prompts sent to cloud providers are covered by the provider's data processing agreement.

## Incident Response

```
Detect → Triage → Contain → Eradicate → Recover → Review
```

- Security vulnerabilities: security@[product-domain] email
- Responsible disclosure policy published on website
- Critical vulnerabilities patched within 72 hours
- Security advisories published for user-facing issues
