use crate::models::Contact;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| PathBuf::from("qso.db"));
        let conn = Connection::open(&path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS contacts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
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
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn insert_contact(&self, contact: &Contact) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO contacts (call_sign, name, qth, frequency, band, mode, rst_sent, rst_recv, notes, qso_date, qso_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                contact.call_sign,
                contact.name,
                contact.qth,
                contact.frequency,
                contact.band,
                contact.mode,
                contact.rst_sent,
                contact.rst_recv,
                contact.notes,
                contact.qso_date,
                contact.qso_time,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_contacts(&self) -> Result<Vec<Contact>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, call_sign, name, qth, frequency, band, mode, rst_sent, rst_recv, notes, qso_date, qso_time, created_at
             FROM contacts ORDER BY id DESC"
        )?;

        let contacts = stmt
            .query_map([], |row| {
                Ok(Contact {
                    id: Some(row.get(0)?),
                    call_sign: row.get(1)?,
                    name: row.get(2)?,
                    qth: row.get(3)?,
                    frequency: row.get(4)?,
                    band: row.get(5)?,
                    mode: row.get(6)?,
                    rst_sent: row.get(7)?,
                    rst_recv: row.get(8)?,
                    notes: row.get(9)?,
                    qso_date: row.get(10)?,
                    qso_time: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(contacts)
    }

    pub fn search_contacts(&self, query: &str) -> Result<Vec<Contact>> {
        let conn = self.conn.lock().unwrap();
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, call_sign, name, qth, frequency, band, mode, rst_sent, rst_recv, notes, qso_date, qso_time, created_at
             FROM contacts WHERE call_sign LIKE ?1 OR name LIKE ?1 OR qth LIKE ?1 ORDER BY id DESC"
        )?;

        let contacts = stmt
            .query_map([&search_pattern], |row| {
                Ok(Contact {
                    id: Some(row.get(0)?),
                    call_sign: row.get(1)?,
                    name: row.get(2)?,
                    qth: row.get(3)?,
                    frequency: row.get(4)?,
                    band: row.get(5)?,
                    mode: row.get(6)?,
                    rst_sent: row.get(7)?,
                    rst_recv: row.get(8)?,
                    notes: row.get(9)?,
                    qso_date: row.get(10)?,
                    qso_time: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(contacts)
    }

    pub fn delete_contact(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM contacts WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update_contact(&self, contact: &Contact) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE contacts SET call_sign = ?1, name = ?2, qth = ?3, frequency = ?4, band = ?5, mode = ?6, rst_sent = ?7, rst_recv = ?8, notes = ?9, qso_date = ?10, qso_time = ?11 WHERE id = ?12",
            params![
                contact.call_sign,
                contact.name,
                contact.qth,
                contact.frequency,
                contact.band,
                contact.mode,
                contact.rst_sent,
                contact.rst_recv,
                contact.notes,
                contact.qso_date,
                contact.qso_time,
                contact.id,
            ],
        )?;
        Ok(())
    }
}
