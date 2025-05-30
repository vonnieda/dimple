use rusqlite::Row;

use super::{Diff, FromRow, LibraryModel, Model};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChangeLog {
    pub key: Option<String>,
    pub actor: String,
    pub timestamp: String,
    pub model: String,
    pub model_key: String,
    pub op: String,
    pub field: Option<String>,
    pub value: Option<String>,
}

impl FromRow for ChangeLog {
    fn from_row(row: &Row) -> Self {
        Self {
            key: row.get("key").unwrap(),
            actor: row.get("actor").unwrap(),
            timestamp: row.get("timestamp").unwrap(),
            model: row.get("model").unwrap(),
            model_key: row.get("model_key").unwrap(),
            op: row.get("op").unwrap(),
            field: row.get("field").unwrap(),
            value: row.get("value").unwrap(),
        }    
    }
}

impl Diff for ChangeLog {
    fn diff(&self, _other: &Self) -> Vec<ChangeLog> where Self: Sized {
        todo!()
    }

    fn apply_diff(&mut self, _diff: &[ChangeLog]) {
        todo!()
    }
}

impl Model for ChangeLog {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn set_key(&mut self, key: Option<String>) {
        self.key = key.clone();
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn type_name(&self) -> String {
        "ChangeLog".to_string()
    }
}

impl LibraryModel for ChangeLog {
    fn upsert(&self, conn: &rusqlite::Connection) {
        conn.execute("INSERT OR REPLACE INTO ChangeLog 
            (key, actor, timestamp, model, model_key, op, field, value) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (&self.key, &self.actor, &self.timestamp, &self.model, 
                &self.model_key, &self.op, &self.field, &self.value)).unwrap();
    }
    
    fn insert(&self, conn: &rusqlite::Connection) {
        todo!()
    }
    
    fn update(&self, conn: &rusqlite::Connection) {
        todo!()
    }    

    
}