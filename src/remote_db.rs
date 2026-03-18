use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    MySQL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub connection_string: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::SQLite,
            connection_string: "qso.db".to_string(),
        }
    }
}

pub struct RemoteDatabase {
    pub config: DatabaseConfig,
    pub is_connected: bool,
}

impl RemoteDatabase {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            is_connected: false,
        }
    }

    pub fn connection_string_example(db_type: &DatabaseType) -> String {
        match db_type {
            DatabaseType::SQLite => "qso.db".to_string(),
            DatabaseType::PostgreSQL => {
                "postgres://user:password@localhost:5432/qsolog".to_string()
            }
            DatabaseType::MySQL => "mysql://user:password@localhost:3306/qsolog".to_string(),
        }
    }
}

impl Default for RemoteDatabase {
    fn default() -> Self {
        Self::new(DatabaseConfig::default())
    }
}
