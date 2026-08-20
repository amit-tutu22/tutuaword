//! iOS stub for plugin host FFI — wasmtime does not build for iOS.

#[derive(Debug)]
pub struct PluginError(String);

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn plugin_list_json() -> Result<String, PluginError> {
    Ok("[]".into())
}

pub fn plugin_install_sample(_grant_edit: bool) -> Result<(), PluginError> {
    Err(PluginError(
        "plugins are not available on iOS (wasmtime unsupported)".into(),
    ))
}

#[allow(dead_code)]
pub fn plugin_invoke(
    _id: &str,
    _session: &mut tw_edit::EditSession,
) -> Result<i32, PluginError> {
    Err(PluginError(
        "plugins are not available on iOS (wasmtime unsupported)".into(),
    ))
}
