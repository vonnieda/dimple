use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct DimageRef {
    pub key: Option<String>,
    pub model_key: String,
    pub dimage_key: String,
}

