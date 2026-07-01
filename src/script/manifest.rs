#[derive(Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub on_enable_ref: Option<usize>,
    pub on_disable_ref: Option<usize>,
}
