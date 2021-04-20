use std::path::Path;
use std::fs::File;
use crate::utils::types::BoxResult;
use serde_json::{from_str, to_string};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

#[derive(Debug)]
struct StoredUser {
  username: String,
  password_hash: String,
  vault: Vec<String>
}

struct Database {
  conn: SqlitePool
}
  impl Database {
    pub async fn new(path: &str) -> BoxResult<Database> {
      if !Path::new(path.clone()).exists() {
        File::create(path.clone())?;
      }
      Ok(Database {
        conn: SqlitePool::connect(format!("sqlite://{}", path).as_str()).await?
      })
    }

    pub async fn init(&self) -> BoxResult<()> {
      sqlx::query(
        "CREATE TABLE IF NOT EXISTS Users (
            Username CHAR,
            PasswordHash VARCHAR(512),
            Vault LONGTEXT,
            UNIQUE(Username)
          )").execute(&self.conn).await?;
      Ok(())
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> BoxResult<()> { // doesn't check if user already exists
      sqlx::query("INSERT INTO Users (Username, PasswordHash, Vault) VALUES ($1, $2, $3)")
          .bind(username)
          .bind(password_hash)
          .bind(to_string(&Vec::<String>::new())?)
          .execute(&self.conn)
          .await?;
      Ok(())
    }

    pub async fn user_exists(&self, username: &str) -> BoxResult<bool> {
      let row = sqlx::query("SELECT Username FROM Users WHERE Username = ?")
          .bind(username)
          .fetch_optional(&self.conn)
          .await?;
      match row {
        Some(_) => Ok(true),
        None => Ok(false)
      }
    }

    pub async fn get_user(&self, username: &str, password_hash: &str) -> BoxResult<Option<StoredUser>> { // nasty
      let row = sqlx::query("SELECT * FROM Users WHERE Username = ? AND PasswordHash = ?")
          .bind(username)
          .bind(password_hash)
          .fetch_optional(&self.conn)
          .await?;
      match row {
        Some(row) => {
          let vault : String = row.try_get(2)?;
          Ok(
            Some(
              StoredUser {
                username: row.try_get(0)?,
                password_hash: row.try_get(1)?,
                vault: match from_str(vault.as_ref()) {
                  Ok(v) => v,
                  Err(e) => panic!("Yikes, error parsing a users vault, you should prbably add better error handling for this: {}", e)
                }
              }  
            )
          )
        },
        None => Ok(None)
      }
    }
  }

#[cfg(test)]
mod database_tests {
  use super::*;
  #[tokio::test(flavor = "multi_thread")]
  async fn full_test() {
    let db_path = "test_db.db";
    let  db = Database::new(&db_path).await.unwrap();
    let _init_succ = &db.init().await.unwrap();
    let _user_added = &db.add_user("penis", "penis").await;
    let user_exists = &db.user_exists("penis").await;
    println!("{:?}",user_exists);
    let user = &db.get_user("penis", "penis").await;
    println!("User: {:?}", user.as_ref().unwrap().as_ref().unwrap());
  }
}