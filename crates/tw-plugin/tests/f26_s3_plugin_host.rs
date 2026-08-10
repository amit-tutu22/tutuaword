//! F26.S3 — wasmtime sandbox + capability gates (unit / integration / stress).

use tw_edit::{Command, EditSession};
use tw_plugin::{
    Capability, PluginContext, PluginError, PluginManager, PluginManifest, SandboxHostState,
    WasmSandbox, SAMPLE_EDIT_PLUGIN_WAT, SAMPLE_READ_PLUGIN_WAT,
};

#[test]
fn u_f26_s3_capability_denied_without_grant() {
    let mut mgr = PluginManager::new().unwrap();
    mgr.install_sample_edit_plugin(false).unwrap();
    let mut session = EditSession::new();
    let err = mgr
        .invoke("com.tutuaword.sample.edit", &mut session)
        .unwrap_err();
    assert!(matches!(
        err,
        PluginError::CapabilityDenied(Capability::DocumentEdit)
    ));
    // Document unchanged.
    assert!(session.document.paragraph_at(0, 0).unwrap().runs[0]
        .text()
        .is_empty()
        || !session
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .runs
            .iter()
            .any(|r| r.text().contains("Hello from plugin")));
}

#[test]
fn u_f26_s3_wasm_edit_with_grant() {
    let mut mgr = PluginManager::new().unwrap();
    mgr.install_sample_edit_plugin(true).unwrap();
    let mut session = EditSession::new();
    let code = mgr
        .invoke("com.tutuaword.sample.edit", &mut session)
        .unwrap();
    assert_eq!(code, 0);
    let text: String = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .map(|r| r.text().to_string())
        .collect();
    assert!(
        text.contains("Hello from plugin"),
        "plugin should insert text: {text:?}"
    );
}

#[test]
fn u_f26_s3_host_require_gate() {
    let ctx = PluginContext {
        manifest: PluginManifest {
            id: "t".into(),
            name: "t".into(),
            version: "0".into(),
            capabilities: vec![Capability::DocumentRead],
        },
        granted: vec![Capability::DocumentRead],
    };
    let mut state = SandboxHostState::new(ctx, EditSession::new());
    assert!(state.require(Capability::DocumentRead).is_ok());
    assert!(matches!(
        state.require(Capability::DocumentEdit),
        Err(PluginError::CapabilityDenied(Capability::DocumentEdit))
    ));
    assert!(matches!(
        state.require(Capability::Network),
        Err(PluginError::CapabilityDenied(Capability::Network))
    ));
}

#[test]
fn i_f26_s3_read_plugin_paragraph_count() {
    let sandbox = WasmSandbox::new().unwrap();
    let ctx = PluginContext {
        manifest: PluginManifest {
            id: "read".into(),
            name: "Read".into(),
            version: "1".into(),
            capabilities: vec![Capability::DocumentRead],
        },
        granted: vec![Capability::DocumentRead],
    };
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "hi".into(),
        })
        .unwrap();
    let state = SandboxHostState::new(ctx, session);
    let (count, state) = sandbox
        .invoke(SAMPLE_READ_PLUGIN_WAT.as_bytes(), state, "run", 100_000)
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(state.session.document.paragraph_at(0, 0).unwrap().runs[0].text(), "hi");
}

#[test]
fn i_f26_s3_lifecycle_enable_disable() {
    let mut mgr = PluginManager::new().unwrap();
    mgr.install(
        PluginManifest {
            id: "com.example.p".into(),
            name: "Example".into(),
            version: "1.0.0".into(),
            capabilities: vec![Capability::DocumentRead],
        },
        SAMPLE_READ_PLUGIN_WAT.as_bytes().to_vec(),
        &[Capability::DocumentRead],
    )
    .unwrap();
    assert_eq!(mgr.plugin_count(), 1);
    mgr.disable("com.example.p").unwrap();
    let mut session = EditSession::new();
    let err = mgr.invoke("com.example.p", &mut session).unwrap_err();
    assert!(err.to_string().contains("disabled"));
    mgr.enable("com.example.p").unwrap();
    let code = mgr.invoke("com.example.p", &mut session).unwrap();
    assert_eq!(code, 1);
    let json = mgr.list_json().unwrap();
    assert!(json.contains("com.example.p"));
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f26_s3_plugin_invoke_churn() {
    let mut mgr = PluginManager::new().unwrap();
    mgr.install_sample_edit_plugin(true).unwrap();
    for i in 0..200 {
        let mut session = EditSession::new();
        mgr.invoke("com.tutuaword.sample.edit", &mut session)
            .unwrap_or_else(|e| panic!("iter {i}: {e}"));
        let text: String = session
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .runs
            .iter()
            .map(|r| r.text().to_string())
            .collect();
        assert!(text.contains("Hello from plugin"), "iter {i}: {text}");
    }
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f26_s3_sandbox_compile_churn() {
    let sandbox = WasmSandbox::new().unwrap();
    for i in 0..100 {
        let ctx = PluginContext {
            manifest: PluginManifest {
                id: format!("p{i}"),
                name: "n".into(),
                version: "1".into(),
                capabilities: vec![Capability::DocumentRead],
            },
            granted: vec![Capability::DocumentRead],
        };
        let state = SandboxHostState::new(ctx, EditSession::new());
        let (n, _) = sandbox
            .invoke(SAMPLE_READ_PLUGIN_WAT.as_bytes(), state, "run", 50_000)
            .unwrap();
        assert_eq!(n, 1);
    }
}

// Keep SAMPLE_EDIT referenced for discoverability in unit tests.
#[test]
fn u_f26_s3_sample_wat_non_empty() {
    assert!(SAMPLE_EDIT_PLUGIN_WAT.contains("insert_hello"));
    assert!(SAMPLE_READ_PLUGIN_WAT.contains("get_paragraph_count"));
}
