use crate::models::Contact;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyPoolOptions;
use sqlx::Row;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::SQLite,
            connection_string: "qso.db".to_string(),
            max_connections: 5,
        }
    }
}

#[derive(Clone)]
pub struct RemoteDatabase {
    pub config: DatabaseConfig,
    pool: Option<Arc<Mutex<sqlx::AnyPool>>>,
    is_connected: bool,
}

impl RemoteDatabase {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            pool: None,
            is_connected: false,
        }
    }

    pub async fn connect(&mut self) -> Result<bool, String> {
        let options = AnyPoolOptions::new()
            .max_connections(self.config.max_connections);

        let pool = options
            .connect(&self.config.connection_string)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        self.pool = Some(Arc::new(Mutex::new(pool)));
        self.is_connected = true;
        Ok(true)
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        if let Some(pool) = self.pool.take() {
            let pool = pool.lock().await;
            pool.close().await;
        }
        self.is_connected = false;
        Ok(())
    }

    pub async fn test_connection(&self) -> Result<bool, String> {
        let options = AnyPoolOptions::new()
            .max_connections(1);

        let pool = options
            .connect(&self.config.connection_string)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        let result = sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Query failed: {}", e))?;

        pool.close().await;
        
        Ok(result.get::<i32, _>(0) == 1)
    }

    pub fn connection_string_example(db_type: &DatabaseType) -> String {
        match db_type {
            DatabaseType::SQLite => "sqlite:qso.db".to_string(),
            DatabaseType::PostgreSQL => {
                "postgres://user:password@localhost:5432/qsolog".to_string()
            }
            DatabaseType::MySQL => "mysql://user:password@localhost:3306/qsolog".to_string(),
        }
    }

    pub async fn create_table_if_not_exists(&self) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let pool = pool.lock().await;

        let create_table_sql = match self.config.db_type {
            DatabaseType::SQLite | DatabaseType::PostgreSQL => {
                "CREATE TABLE IF NOT EXISTS contacts (
                    id SERIAL PRIMARY KEY,
                    call_sign TEXT NOT NULL,
                    name TEXT,
                    qth TEXT,
                    frequency REAL,
                    band TEXT,
                    mode TEXT,
                    rst_sent TEXT,
                    rst_recv TEXT,
                    notes TEXT,
                    qso_date TEXT,
                    qso_time TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )"
            }
            DatabaseType::MySQL => {
                "CREATE TABLE IF NOT EXISTS contacts (
                    id INT AUTO_INCREMENT PRIMARY KEY,
                    call_sign VARCHAR(20) NOT NULL,
                    name VARCHAR(100),
                    qth VARCHAR(100),
                    frequency DOUBLE,
                    band VARCHAR(20),
                    mode VARCHAR(20),
                    rst_sent VARCHAR(10),
                    rst_recv VARCHAR(10),
                    notes TEXT,
                    qso_date VARCHAR(20),
                    qso_time VARCHAR(10),
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )"
            }
        };

        sqlx::query(create_table_sql)
            .execute(&**&pool)
            .await
            .map_err(|e| format!("Failed to create table: {}", e))?;

        Ok(())
    }

    fn row_to_contact(row: &sqlx::any::AnyRow) -> Contact {
        Contact {
            id: row.try_get::<i64, _>(0).ok().map(|id| id as i64),
            call_sign: row.try_get::<&str, _>(1).unwrap_or_default().to_string(),
            name: row.try_get::<&str, _>(2).unwrap_or_default().to_string(),
            qth: row.try_get::<&str, _>(3).unwrap_or_default().to_string(),
            frequency: row.try_get::<f64, _>(4).unwrap_or(0.0),
            band: row.try_get::<&str, _>(5).unwrap_or_default().to_string(),
            mode: row.try_get::<&str, _>(6).unwrap_or_default().to_string(),
            rst_sent: row.try_get::<&str, _>(7).unwrap_or_default().to_string(),
            rst_recv: row.try_get::<&str, _>(8).unwrap_or_default().to_string(),
            notes: row.try_get::<&str, _>(9).unwrap_or_default().to_string(),
            qso_date: row.try_get::<&str, _>(10).unwrap_or_default().to_string(),
            qso_time: row.try_get::<&str, _>(11).unwrap_or_default().to_string(),
            created_at: row.try_get::<&str, _>(12).ok().map(|s| s.to_string()),
        }
    }

    pub async fn insert_contact(&self, contact: &Contact) -> Result<i64, String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let pool = pool.lock().await;

        let result = sqlx::query(
            "INSERT INTO contacts (call_sign, name, qth, frequency, band, mode, rst_sent, rst_recv, notes, qso_date, qso_time)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(&contact.call_sign)
        .bind(&contact.name)
        .bind(&contact.qth)
        .bind(contact.frequency)
        .bind(&contact.band)
        .bind(&contact.mode)
        .bind(&contact.rst_sent)
        .bind(&contact.rst_recv)
        .bind(&contact.notes)
        .bind(&contact.qso_date)
        .bind(&contact.qso_time)
        .execute(&**&pool)
        .await
        .map_err(|e| format!("Insert failed: {}", e))?;

        Ok(result.last_insert_id().unwrap_or(0) as i64)
    }

    pub async fn get_all_contacts(&self) -> Result<Vec<Contact>, String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let pool = pool.lock().await;

        let rows = sqlx::query(
            "SELECT id, call_sign, name, qth, frequency, band, mode, rst_sent, rst_recv, notes, qso_date, qso_time, created_at
             FROM contacts ORDER BY id DESC"
        )
        .fetch_all(&**&pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        let contacts: Vec<Contact> = rows.iter().map(|row| Self::row_to_contact(row)).collect();

        Ok(contacts)
    }

    pub async fn search_contacts(&self, query: &str) -> Result<Vec<Contact>, String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let pool = pool.lock().await;

        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query(
            "SELECT id, call_sign, name, qth, frequency, band, mode, rst_sent, rst_recv, notes, qso_date, qso_time, created_at
             FROM contacts WHERE call_sign LIKE $1 OR name LIKE $1 OR qth LIKE $1 ORDER BY id DESC"
        )
        .bind(&search_pattern)
        .fetch_all(&**&pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        let contacts: Vec<Contact> = rows.iter().map(|row| Self::row_to_contact(row)).collect();

        Ok(contacts)
    }

    pub async fn delete_contact(&self, id: i64) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let pool = pool.lock().await;

        sqlx::query("DELETE FROM contacts WHERE id = $1")
            .bind(id)
            .execute(&**&pool)
            .await
            .map_err(|e| format!("Delete failed: {}", e))?;

        Ok(())
    }

    pub async fn update_contact(&self, contact: &Contact) -> Result<(), String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let pool = pool.lock().await;

        let id = contact.id.ok_or("Contact ID required for update")?;

        sqlx::query(
            "UPDATE contacts SET call_sign = $1, name = $2, qth = $3, frequency = $4, band = $5, mode = $6, rst_sent = $7, rst_recv = $8, notes = $9, qso_date = $10, qso_time = $11 WHERE id = $12"
        )
        .bind(&contact.call_sign)
        .bind(&contact.name)
        .bind(&contact.qth)
        .bind(contact.frequency)
        .bind(&contact.band)
        .bind(&contact.mode)
        .bind(&contact.rst_sent)
        .bind(&contact.rst_recv)
        .bind(&contact.notes)
        .bind(&contact.qso_date)
        .bind(&contact.qso_time)
        .bind(id)
        .execute(&**&pool)
        .await
        .map_err(|e| format!("Update failed: {}", e))?;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
}

impl Default for RemoteDatabase {
    fn default() -> Self {
        Self::new(DatabaseConfig::default())
    }
}
