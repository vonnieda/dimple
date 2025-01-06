use dimple_core_macro::ModelSupport;

#[derive(Debug, Clone, Default, PartialEq, ModelSupport)]
pub struct GenreRef {
    pub key: Option<String>,
    pub model_key: String,
    pub genre_key: String,
}

