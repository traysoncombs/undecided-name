use rusqlite::{params, Connection};
use std::path::Path;
use std::fs::File;
use crate::utils::types::BoxResult;

struct StoredUser {
  username: String,
  password_hash: String,
  vault: Vec<String>
}


struct Database {
  conn: rusqlite::Connection,
} 
  impl Database {
    pub fn new(path: &str) -> BoxResult<Database> {
      if !Path::new(path.clone()).exists() {
        File::create(path.clone())?;
      }
      Ok(Database {
        conn: Connection::open(path)?
      })
    }

    pub fn init(&mut self) -> BoxResult<()> {
      self.conn.execute(
          "CREATE TABLE IF NOT EXISTS Users (
            Username CHAR,
            PasswordHash VARCHAR(512),
            Vault LONGTEXT
          )",
          params![]
      )?;
      Ok(())
    }

    pub fn add_user(&mut self, username: &str, password_hash: &str) -> BoxResult<()> { // doesn't check if user already exists
      self.conn.execute("INSERT INTO Users (Username, PasswordHash) VALUES (?1, ?2)", params![username, password_hash])?;
      Ok(())
    }

    pub fn get_user(&mut self, username: &str, password_hash: &str) -> BoxResult<Option<StoredUser>> {
      self.conn.query_row("SELECT * FROM Users WHERE Username=?1 AND PasswordHash=?2", 
          params![username, password_hash], 
          |row| {
            Ok(Some(StoredUser {
              username: row.get(0)?,
              password_hash: row.get(1)?,
              vault: row.get(2)?
            }))
      }).optional()?
    }
  }

#[cfg(test)]
mod database_tests {
  use super::*;
  #[test]
  fn full_test() {
    let db_path = "test_db.db";
    let mut db = Database::new(&db_path).unwrap();
    let init_succ = db.init().unwrap();
    let user_added = db.add_user("penis".as_ref(), "penis".as_ref());


  }
}