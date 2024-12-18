use std::sync::{Arc, RwLock};

use rusqlite::{Connection, OptionalExtension, Params};
use uuid::Uuid;

use crate::{model::Model, notifier::Notifier};

use threadpool::ThreadPool;


#[derive(Clone)]
pub struct ModelStore {
    _database_path: String,
    conn: Arc<RwLock<Connection>>,
    notifier: Notifier<String>,
    threadpool: ThreadPool,
}

impl ModelStore {
    pub fn open(database_path: &str) -> Self {

        let conn = Connection::open(database_path).unwrap();

        Self {
            _database_path: database_path.to_string(),
            conn: Arc::new(RwLock::new(conn)),
            notifier: Notifier::new(),
            threadpool: ThreadPool::new(1),
        }
    }

    pub fn on_change(&self, l: Box<dyn Fn(&String) + Send>) {
        self.notifier.on_notify(l);
    }

    fn emit_change(&self, _type_name: &str, key: &str) {
        let notifier = self.notifier.clone();
        let key = key.to_string();
        self.threadpool.execute(move || {
            notifier.notify(&key);
        });
    }

    pub fn save<T: Model>(&self, obj: &T) -> T {
        let key = obj.key().or_else(|| Some(Uuid::new_v4().to_string()));
        let mut obj = obj.clone();
        obj.set_key(key.clone());
        obj.upsert(&self.conn().read().unwrap());
        let new: T = self.get(&key.unwrap()).unwrap();
        self.emit_change(&obj.type_name(), &obj.key().unwrap());
        new
    }

    pub fn get<T: Model>(&self, key: &str) -> Option<T> {
        let sql = format!("SELECT * FROM {} WHERE key = ?1", T::default().type_name());
        self.conn().read().unwrap().query_row(&sql, (key,), 
            |row| Ok(T::from_row(row))).optional().unwrap()
    }

    pub fn list<T: Model>(&self) -> Vec<T> {
        let sql = format!("SELECT * FROM {}", T::default().type_name());
        self.conn().read().unwrap().prepare(&sql).unwrap()
            .query_map((), |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect()
    }

    pub fn query<T: Model, P: Params>(&self, sql: &str, params: P) -> Vec<T> {
        let conn = self.conn();
        let result = conn.read().unwrap().prepare(&sql).unwrap()
            .query_map(params, |row| Ok(T::from_row(row))).unwrap()
            .map(|m| m.unwrap())
            .collect();
        result
    }

    fn conn(&self) -> Arc<RwLock<Connection>> {
        self.conn.clone()
    }
}

#[cfg(test)]
mod tests {
    use dimple_core_macro::ModelSupport;
    use rusqlite_migration::{Migrations, M};

    use super::ModelStore;

    #[test]
    fn it_works() {
        let store = ModelStore::open("file:32dd72e2-3a8e-42ea-8e69-17136b60dfe9?mode=memory&cache=shared");
        if let Ok(mut conn) = store.conn().write() {
            let migrations = Migrations::new(vec![
                M::up("CREATE TABLE TestModel (key TEXT PRIMARY KEY, name TEXT);"),
                M::up("ALTER TABLE TestModel ADD COLUMN summary TEXT;"),
            ]);
            migrations.to_latest(&mut conn).unwrap();
        }
        let model = store.save(&TestModel {
            summary: Some("Oh Hi Mark".to_string()),
            ..Default::default()
        });
        assert!(model.key.is_some());
        assert!(model.summary.unwrap() == "Oh Hi Mark");
    }

    #[derive(Debug, Clone, Default, ModelSupport)]
    struct TestModel {
        pub key: Option<String>,
        pub name: Option<String>,
        pub summary: Option<String>,
    }
}
