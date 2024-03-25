use std::time::Instant;

use crate::model::Model;

pub trait Source {
    /// Load the model using its key. Returns None if no key is set, or if the
    /// key doesn't exist in the database.
    fn get(&self, model: &Model) -> Result<Option<Model>>;

    /// Get a list of models that are related to the specified model. If None is
    /// specified list all models of the specified type.
    fn list(
        &self,
        list_of: &Model,
        related_to: Option<&Model>,
    ) -> Result<Box<dyn Iterator<Item = Model>>>;
}

