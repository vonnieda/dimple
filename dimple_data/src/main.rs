pub mod data_store;
pub mod notifier;

use std::{error::Error, sync::{Arc, Mutex}, thread, time::Duration};

use data_store::{DataStore, DataStoreError, DataStoreEvent, DataStoreModel};
use notifier::Notifier;
use r2d2::{CustomizeConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Transaction;
use ulid::Generator;
use uuid::Uuid;

pub struct SqliteDataStore {
    pool: Arc<Pool<SqliteConnectionManager>>,
    ulid_gen: Arc<Mutex<Generator>>,
    notifier: Arc<Notifier<DataStoreEvent>>,
}

impl SqliteDataStore {
    pub fn open(path: &str) -> Result<SqliteDataStore, Box<dyn Error>> {
        Self::open_with_manager(r2d2_sqlite::SqliteConnectionManager::file(path))
    }

    pub fn open_memory() -> Result<SqliteDataStore, Box<dyn Error>> {
        Self::open_with_manager(r2d2_sqlite::SqliteConnectionManager::memory())
    }

    fn open_with_manager(manager: SqliteConnectionManager) -> Result<SqliteDataStore, Box<dyn Error>> {
        let pool = r2d2::Pool::builder()
            .connection_customizer(Box::new(SqliteDataStoreConnectionCustomizer{}))
            .build(manager)?;

        let ds = SqliteDataStore {
            pool: Arc::new(pool),
            ulid_gen: Arc::new(Mutex::new(Generator::new())),
            notifier: Arc::new(Notifier::new()),
        };

        ds.initialize()?;

        Ok(ds)
    }

    fn initialize(&self) -> Result<(), Box<dyn Error>> {
        self.conn().execute("CREATE TABLE IF NOT EXISTS __Sync_ModelLastModified (
            model_key TEXT NOT NULL, 
            last_modified TEXT NOT NULL,
            PRIMARY KEY (model_key, last_modified)
        );", ())?;
        // self.conn().execute("CREATE TABLE IF NOT EXISTS __SyncState (
        //     actor_id TEXT NOT NULL, 
        //     newest_change_seen_ulid TEXT NOT NULL
        // );", ())?;
        Ok(())
    }

    fn set_modified(&self, key: &str, txn: &Transaction) -> Result<String, Box<dyn Error>> {
        let last_modified = self.ulid();
        txn.execute("INSERT OR REPLACE INTO __Sync_ModelLastModified (model_key, last_modified) VALUES (?1, ?2);", 
            (key, &last_modified))?;
        Ok(last_modified)
    }

    pub fn conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.pool.get().unwrap()
    }

    pub fn ulid(&self) -> String {
        self.ulid_gen.lock().unwrap().generate().unwrap().to_string()
    }
}

impl DataStore for SqliteDataStore {
    fn save<Model: DataStoreModel>(&self, mut model: Model) -> Result<Model, Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let txn = conn.transaction()?;
        let event = if model.key().is_none() {
            model.set_key(Some(Uuid::new_v4().to_string()));
            model.insert(&txn)?;
            DataStoreEvent::Created
        }
        else {
            model.update(&txn)?;
            DataStoreEvent::Updated
        };
        self.set_modified(model.key().unwrap().as_str(), &txn)?;
        txn.commit()?;
        self.notifier.notify(event);
        let model = self.get(model.key().unwrap().as_str())?;
        Ok(model.unwrap())
    }

    fn get<Model: DataStoreModel>(&self, key: &str) -> Result<Option<Model>, Box<dyn Error>> {
        // let sql = format!("SELECT * FROM {} WHERE key = ?1", Model::type_name());
        // self.conn().query_row(&sql, (key,), 
        //     |row| Ok(Model::from_row(row))).optional().unwrap()
        todo!()
    }

    fn list<Model: DataStoreModel>(&self) -> Result<Vec<Model>, Box<dyn Error>> {
        todo!()
    }

    fn query<Model: DataStoreModel>(&self, sql: &str) -> Result<Vec<Model>, Box<dyn Error>> {
        todo!()
    }

    fn delete<Model: DataStoreModel>(&self, key: &str) -> Result<Option<Model>, Box<dyn Error>> {
        todo!()
    }

    fn observe<Model: DataStoreModel>(&self, model: Model, callback: impl FnMut(DataStoreEvent) -> () + 'static) {
        self.notifier.observe(callback);
    }

    fn unobserve<Model: DataStoreModel>(&self, model: Model, callback: impl FnMut(DataStoreEvent) -> ()) {
        todo!()
    }

    fn changes_since(&self, ulid: &str) -> Result<(), DataStoreError> {
        todo!()
    }
}

#[derive(Debug)]
struct SqliteDataStoreConnectionCustomizer;
impl CustomizeConnection<rusqlite::Connection, rusqlite::Error> for SqliteDataStoreConnectionCustomizer {
    fn on_acquire(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;                
        Ok(())
    }
}

#[derive(Default, Debug)]
struct Artist {
    key: Option<String>,
    name: Option<String>,
}

impl DataStoreModel for Artist {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }
    
    fn insert(&self, txn: &Transaction) -> Result<(), Box<dyn Error>> {
        let _ = txn.execute("INSERT INTO Artist (key, name) VALUES (?1, ?2);", 
            (&self.key, &self.name))?;
        Ok(())
    }
    
    fn update(&self, txn: &Transaction) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    
    fn type_name() -> String {
        "Artist".to_string()
    }
}

fn main() -> Result<(), Box<dyn Error>> {    
    let ds = SqliteDataStore::open("test.db")?;
    ds.conn().execute("CREATE TABLE IF NOT EXISTS Artist (key TEXT NOT NULL PRIMARY KEY, name TEXT);", ())?;
    ds.observe(Artist::default(), move |event| {
        dbg!(event);
    });
    let artist = ds.save(Artist {
        name: Some("Humphrey Bogart".to_string()),
        ..Default::default()
    })?;
    thread::sleep(Duration::from_secs(2));
    dbg!(artist);
    let artists = ds.list::<Artist>()?;
    dbg!(artists);
    thread::sleep(Duration::from_secs(2));
    Ok(())
}
