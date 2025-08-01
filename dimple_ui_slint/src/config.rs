use anyhow::Result;
use dimple_db::{db::{Migrations, M}, Db};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Config {
    pub db: Db,
}

impl Config {
    pub fn new(db: Db) -> Result<Self> {
        let migrations = Migrations::new(vec![
            M::up("CREATE TABLE ConfigValue (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                value TEXT
            );"),
        ]);
        db.migrate(&migrations)?;
        Ok(Config {
            db,
        })
    }

    fn get_value(&self, key: &str) -> Option<String> {
        let values: Vec<ConfigValue> = self.db.query("SELECT * FROM ConfigValue WHERE key = ?", (key,)).unwrap();
        values.into_iter().next().map(|v| v.value)?
    }

    fn set_value(&self, key: &str, value: Option<String>) {
        self.db.transaction(|tx| {
            let v = tx.query::<ConfigValue, _>("SELECT * FROM ConfigValue WHERE key = ?", (key,))?.into_iter().next();
            if let Some(mut v) = v {
                v.value = value;
                tx.save(&v)?;
            }
            else {
                tx.save(&ConfigValue {
                    key: key.to_string(),
                    value,
                    ..Default::default()
                })?;
            }
            Ok(())
        }).unwrap();
    }

    pub fn offline(&self) -> bool {
        self.get_value("offline") == Some("true".to_string())
    }

    pub fn set_offline(&self, value: bool) {
        self.set_value("offline", if value {
            Some("true".to_string())
        }
        else {
            Some("false".to_string())
        })
    }

    pub fn sidebar_open(&self) -> bool {
        self.get_value("sidebar_open") == Some("true".to_string())
    }

    pub fn set_sidebar_open(&self, value: bool) {
        self.set_value("sidebar_open", if value {
            Some("true".to_string())
        }
        else {
            Some("false".to_string())
        })
    }

    pub fn debug(&self) -> bool {
        self.get_value("debug") == Some("true".to_string())
    }

    pub fn set_debug(&self, value: bool) {
        self.set_value("debug", if value {
            Some("true".to_string())
        }
        else {
            Some("false".to_string())
        })
    }

    pub fn s3_endpoint(&self) -> Option<String> {
        self.get_value("s3_endpoint")
    }

    pub fn set_s3_endpoint(&self, value: Option<String>) {
        self.set_value("s3_endpoint", value)
    }


    pub fn s3_region(&self) -> Option<String> {
        self.get_value("s3_region")
    }

    pub fn set_s3_region(&self, value: Option<String>) {
        self.set_value("s3_region", value)
    }

    
    pub fn s3_bucket(&self) -> Option<String> {
        self.get_value("s3_bucket")
    }

    pub fn set_s3_bucket(&self, value: Option<String>) {
        self.set_value("s3_bucket", value)
    }

    
    pub fn s3_access_key(&self) -> Option<String> {
        self.get_value("s3_access_key")
    }

    pub fn set_s3_access_key(&self, value: Option<String>) {
        self.set_value("s3_access_key", value)
    }

    
    pub fn s3_secret_key(&self) -> Option<String> {
        self.get_value("s3_secret_key")
    }

    pub fn set_s3_secret_key(&self, value: Option<String>) {
        self.set_value("s3_secret_key", value)
    }

    
    pub fn s3_prefix(&self) -> Option<String> {
        self.get_value("s3_prefix")
    }

    pub fn set_s3_prefix(&self, value: Option<String>) {
        self.set_value("s3_prefix", value)
    }
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
struct ConfigValue {
    id: String,
    key: String,
    value: Option<String>,
}

#[cfg(test)]
mod tests {
    use dimple_db::Db;

    use crate::config::Config;

    #[test]
    fn basics() -> anyhow::Result<()> {
        let config = Config::new(Db::open_memory()?)?;
        assert_eq!(config.offline(), false);
        config.set_offline(true);
        assert_eq!(config.offline(), true);
        Ok(())
    }
}