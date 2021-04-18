//use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::fs::File;
use crate::utils::types::BoxResult;
use serde_json::{from_str, to_string};
use sqlx::Connection;

#[derive(Debug)]
struct StoredUser {
  username: String,
  password_hash: String,
  vault: Vec<String>
}

struct Database {
  conn: sqlx::Connection,
}
  impl Database {
    async pub fn new(path: &str) -> BoxResult<Database> {
      if !Path::new(path.clone()).exists() {
        File::create(path.clone())?;
      }
      Ok(Database {
        conn: SqliteConnection::open("sqlite://"+path).await?
      })
    }

    async pub fn init(self) -> BoxResult<()> {
      sqlx::query(
        "CREATE TABLE IF NOT EXISTS Users (
            Username CHAR,
            PasswordHash VARCHAR(512),
            Vault LONGTEXT,
            UNIQUE(Username)
          )").execute(&self.conn).await?;
      Ok(())
    }

    async pub fn add_user(self, username: &str, password_hash: &str) -> BoxResult<()> { // doesn't check if user already exists
      sqlx::query("INSERT INTO Users (Username, PasswordHash, Vault) VALUES ($1, $2, $3)", params![username, password_hash, to_string(&Vec::<String>::new())?])
          .bind(username)
          .bind(password_hash)
          .bind(to_string(&Vec::<String>::new())?)
          .execute(&self.conn)
          .await?;
      Ok(())
    }

    async pub fn get_user(self, username: &str, password_hash: &str) -> BoxResult<Option<StoredUser>> { // nasty
      let row = sqlx::query("SELECT * FROM Users WHERE Username = ? AND PasswordHash = ?")
          .bind(username)
          .bind(password_hash)
          .fetch_optional(&self.conn);
      match row {
        Some(row) => {
          let vault : String = row.get(2)?;
          Ok(
            Some(
              StoredUser {
                username: row.get(0)?,
                password_hash: row.get(1)?,
                vault: match from_str(vault.as_ref()) {
                  Some(v) => v,
                  Err(e) => panic!("Yikes, error parsing a users vault, you should prbably add better error handling for this: {}", e);
                }
              }  
            )
          )
        },
        None => Ok(None)
      }
      /*Ok(
        self.conn.query_row("SELECT * FROM Users WHERE Username=?1 AND PasswordHash=?2",
          params![username, password_hash],
          |row| {
            let vault : String = row.get(2)?;  // need this as
            Ok(StoredUser {
              username: row.get(0)?,
              password_hash: row.get(1)?,
              vault: match from_str(vault.as_ref()) {
                Ok(v) => v,
                Err(e) => panic!("Yikes, error parsing a users vault, you should prbably add better error handling for this: {}", e)  // error parsing vault, i.e. not good
              }
            })
        }).optional()?
      )*/
    }
  }

#[cfg(test)]
mod database_tests {
  use super::*;
  #[tokio::test]
  fn full_test() {
    let db_path = "test_db.db";
    let mut db = Database::new(&db_path).await.unwrap();
    let _init_succ = db.init().await.unwrap();
    let _user_added = db.add_user("penis", "penis").await;
    let user_added2 = db.add_user.await;
    println!("{:?}",user_added2);
    let user = db.get_user("penis", "penis").await;
    println!("User: {:?}", user.unwrap().unwrap());
  }
}