//! Plugin host FFI (F26.S3) — bridges Flutter to wasmtime sandbox.

use parking_lot::Mutex;
use std::sync::OnceLock;
use tw_plugin::{PluginError, PluginManager};

static PLUGIN_HOST: OnceLock<Mutex<Option<PluginManager>>> = OnceLock::new();

fn plugin_host() -> &'static Mutex<Option<PluginManager>> {
    PLUGIN_HOST.get_or_init(|| Mutex::new(PluginManager::new().ok()))
}

pub fn plugin_list_json() -> Result<String, PluginError> {
    let guard = plugin_host().lock();
    let Some(manager) = guard.as_ref() else {
        return Err(PluginError::Message("plugin host unavailable".into()));
    };
    manager.list_json()
}

pub fn plugin_install_sample(grant_edit: bool) -> Result<(), PluginError> {
    let mut guard = plugin_host().lock();
    let manager = guard
        .as_mut()
        .ok_or_else(|| PluginError::Message("plugin host unavailable".into()))?;
    manager.install_sample_edit_plugin(grant_edit)
}

pub fn plugin_invoke(id: &str, session: &mut tw_edit::EditSession) -> Result<i32, PluginError> {
    // Compile/cache lookup under the lock, then drop it before WASM runs.
    let prepared = {
        let guard = plugin_host().lock();
        let Some(manager) = guard.as_ref() else {
            return Err(PluginError::Message("plugin host unavailable".into()));
        };
        manager.prepare_invoke(id)?
    };
    prepared.run(session)
}
