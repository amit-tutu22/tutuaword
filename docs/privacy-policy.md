# Privacy Policy

**Effective date:** August 11, 2026  
**Last updated:** August 11, 2026

## Who we are

**Tutuaword** (“we”, “us”, or “our”) is a document editor published by **amit.blr76** (application identifier: `com.manasai.tutuaword`). This Privacy Policy explains how Tutuaword handles information when you use our mobile, desktop, and web applications (collectively, the “App”).

If you have questions about this policy, contact us at **amit.blr76@gmail.com**.

---

## Summary

- **Your documents stay on your device by default.** Tutuaword is a local-first editor. We do not operate a cloud document service or user accounts in the current release.
- **We do not sell your data.** We do not run advertising or cross-app tracking.
- **We do not collect analytics or crash reports** in the current release.
- **Cloud AI is optional and user-controlled.** If you enter API keys and use AI features, selected text or document excerpts are sent to the third-party AI provider you choose (for example OpenAI or Google Gemini), subject to that provider’s terms and privacy policy.
- **Local AI is supported.** You may route AI requests to a local endpoint (for example a self-hosted Llama server) so content never leaves your network.

---

## Information processed by the App

### 1. Documents and files you create or open

When you create, edit, open, save, export, print, or share documents, that content is processed **on your device** (or in your browser session on web). Tutuaword reads and writes files only when you choose to open or save them, using the platform file picker or paths you select.

We do **not** upload your document content to Tutuaword servers.

### 2. Data stored locally on your device

Depending on your platform, Tutuaword may store the following **locally** to support app functionality:

| Data | Purpose | Typical location |
|------|---------|------------------|
| Autosave drafts | Recover unsaved work | On desktop: `~/.tutuaword/autosave/`; on mobile: app sandbox; on web: in-memory only for the session |
| Recent document paths | “Recent files” list | `~/.tutuaword/recent.json` (desktop) or app sandbox (mobile) |
| App settings | Preferences such as autosave interval | `~/.tutuaword/settings.json` (desktop) or app sandbox (mobile) |
| User templates | Templates you save from documents | `~/.tutuaword/templates/` (desktop) or app sandbox (mobile) |
| Digital signatures | Self-attested Ed25519 signatures embedded in documents | Stored inside your document files |

You can delete local Tutuaword data by removing files from the locations above or uninstalling the App.

### 3. Optional cloud AI features

If you configure and use AI-assisted features (rewrite, summarize, chat, smart edit, grammar suggestions via cloud providers, and similar), the App may send **prompts derived from your document or current selection** to:

- **OpenAI** ([Privacy Policy](https://openai.com/policies/privacy-policy))
- **Google Gemini** ([Privacy Policy](https://policies.google.com/privacy))

You provide API keys voluntarily in App settings or via environment variables on desktop (`OPENAI_API_KEY`, `GEMINI_API_KEY`). Keys are used only to authenticate your requests to those providers. **We do not operate a server that receives your API keys or document content.**

Alternatively, you may:

- Use **Always local** routing to avoid cloud providers where local processing is available
- Point AI requests to a **local Llama-compatible endpoint** you control

You are responsible for reviewing each provider’s terms and for not sending sensitive content to a cloud provider unless you accept their data handling practices.

### 4. Local processing (no network)

The following run **on device** without sending document content to Tutuaword or third-party servers:

- Core editing, layout, and rendering
- Local spell-check rules (where implemented locally)
- Self-attested digital signatures
- Password-protected document encryption/decryption (passwords are not transmitted by Tutuaword)

### 5. Sharing through your operating system

When you use Share, export, or “Open with” features, documents are passed to **other apps or services you choose** through your platform’s share sheet or file system. We do not control how those recipients handle your data.

---

## Information we do not collect

In the current release, Tutuaword **does not**:

- Require account registration or collect email addresses for sign-in
- Collect advertising identifiers or perform cross-app tracking (`NSPrivacyTracking` is declared as false on Apple platforms)
- Run product analytics, marketing analytics, or crash-reporting SDKs
- Sync documents to Tutuaword-operated cloud storage
- Access your contacts, photos, microphone, camera, or location

---

## Permissions

The App may request platform permissions necessary for its features:

| Permission | Why it is used |
|------------|----------------|
| **Internet** (Android and network-capable builds) | Optional cloud AI providers when you configure API keys and use AI features |
| **Read external storage** (Android 12 and below) | Open documents from local storage when you choose a file |
| **File access / document picker** (all platforms) | Open, save, and export documents at paths you select |

The App accesses only files and folders you explicitly choose, except for autosave and recent-files metadata stored in the App’s local data directory.

---

## Data retention

- **Documents:** Retained until you delete them from your device or storage.
- **Autosave drafts:** Retained locally until you save, discard, or clear autosave data.
- **Recent files list:** Retained locally until you clear it or uninstall the App.
- **Cloud AI providers:** Retention is governed by the provider you select. Consult OpenAI or Google policies if you use their services.

---

## Security

We design Tutuaword with a local-first security model:

- Document encryption passwords and API keys are not sent to Tutuaword servers
- Release builds use platform-standard TLS when contacting third-party AI APIs
- Macro-enabled documents may be blocked by enterprise policy settings when configured

No method of transmission or storage is completely secure. You are responsible for protecting your device, API keys, and document passwords.

---

## Children’s privacy

Tutuaword is not directed at children under 13 (or the minimum age required in your jurisdiction). We do not knowingly collect personal information from children. If you believe a child has provided personal information through the App, contact us through the store listing and we will take appropriate steps to delete it.

---

## International users

If you use optional cloud AI features, your prompts may be processed in countries where your chosen AI provider operates. By using those features, you acknowledge that transfer. Local-only use avoids sending content to third-party AI services.

---

## Your rights

Depending on where you live, you may have rights to access, correct, delete, or restrict processing of personal information. Because Tutuaword stores most data locally on your device and does not operate user accounts in the current release:

- **Access / deletion:** You can access and delete your documents and local App data directly on your device.
- **Cloud AI data:** Requests relating to content sent to OpenAI or Google must be directed to those providers under their policies.

Residents of the European Economic Area, United Kingdom, and California may have additional statutory rights. Contact **amit.blr76@gmail.com** to exercise rights that apply to data we control.

---

## Changes to this policy

We may update this Privacy Policy from time to time. We will revise the “Last updated” date at the top. Material changes may also be communicated through App release notes or store listing updates. Continued use of the App after changes become effective constitutes acceptance of the revised policy.

---

## Third-party services (reference)

| Service | Used for | Privacy policy |
|---------|----------|----------------|
| OpenAI | Optional cloud AI | https://openai.com/policies/privacy-policy |
| Google (Gemini) | Optional cloud AI | https://policies.google.com/privacy |

Tutuaword is not affiliated with Microsoft, OpenAI, or Google. Word and DOCX are formats supported for compatibility; trademarks belong to their respective owners.

---

## Contact

**Developer:** amit.blr76  
**Application ID:** com.manasai.tutuaword  

For privacy inquiries, email **amit.blr76@gmail.com**.
