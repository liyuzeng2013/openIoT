use rusqlite::{Connection, Result};
use shared::types::*;

pub struct Database {
    path: String,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let db = Self { path: path.to_string() };
        db.init_tables()?;
        Ok(db)
    }

    fn conn(&self) -> Connection {
        Connection::open(&self.path).unwrap()
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS devices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                device_id TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                device_type TEXT NOT NULL,
                protocol TEXT NOT NULL,
                token TEXT NOT NULL,
                online INTEGER NOT NULL DEFAULT 0,
                last_seen TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS device_states (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL,
                state TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_user(&self, username: &str, password_hash: &str) -> Result<i64> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO users (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![username, password_hash, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, created_at FROM users WHERE username = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn create_device(&self, device: &Device) -> Result<i64> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            r#"INSERT INTO devices 
            (user_id, device_id, name, device_type, protocol, token, online, last_seen, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            rusqlite::params![
                device.user_id,
                device.device_id,
                device.name,
                serde_json::to_string(&device.device_type).unwrap(),
                serde_json::to_string(&device.protocol).unwrap(),
                device.token,
                device.online as i32,
                now,
                now,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_user_devices(&self, user_id: i64) -> Result<Vec<Device>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, device_id, name, device_type, protocol, token, online, last_seen, created_at 
             FROM devices WHERE user_id = ?1",
        )?;
        let devices = stmt.query_map(rusqlite::params![user_id], |row| {
            let device_type_str: String = row.get(4)?;
            let protocol_str: String = row.get(5)?;
            Ok(Device {
                id: row.get(0)?,
                user_id: row.get(1)?,
                device_id: row.get(2)?,
                name: row.get(3)?,
                device_type: serde_json::from_str(&device_type_str).unwrap(),
                protocol: serde_json::from_str(&protocol_str).unwrap(),
                token: row.get(6)?,
                online: row.get::<_, i32>(7)? != 0,
                last_seen: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        Ok(devices.collect::<Result<Vec<_>>>()?)
    }

    pub fn get_device_by_id(&self, device_id: &str) -> Result<Option<Device>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, device_id, name, device_type, protocol, token, online, last_seen, created_at 
             FROM devices WHERE device_id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![device_id], |row| {
            let device_type_str: String = row.get(4)?;
            let protocol_str: String = row.get(5)?;
            Ok(Device {
                id: row.get(0)?,
                user_id: row.get(1)?,
                device_id: row.get(2)?,
                name: row.get(3)?,
                device_type: serde_json::from_str(&device_type_str).unwrap(),
                protocol: serde_json::from_str(&protocol_str).unwrap(),
                token: row.get(6)?,
                online: row.get::<_, i32>(7)? != 0,
                last_seen: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_device_online(&self, device_id: &str, online: bool) -> Result<()> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE devices SET online = ?1, last_seen = ?2 WHERE device_id = ?3",
            rusqlite::params![online as i32, now, device_id],
        )?;
        Ok(())
    }

    pub fn delete_device(&self, device_id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM devices WHERE device_id = ?1", rusqlite::params![device_id])?;
        Ok(())
    }

    pub fn save_device_state(&self, device_id: &str, state: &serde_json::Value) -> Result<()> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO device_states (device_id, state, timestamp) VALUES (?1, ?2, ?3)",
            rusqlite::params![device_id, serde_json::to_string(state).unwrap(), now],
        )?;
        Ok(())
    }
}
