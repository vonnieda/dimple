use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct LinkRef {
    pub key: Option<String>,
    pub model_key: String,
    pub link_key: String,
}
