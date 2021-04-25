use crate::utils::errors::CustomErrors;
use crate::utils::types::CustomResult;
use serde_json::{from_str, to_string};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct StoredUser {
    pub username: String,
    pub password_hash: String,
    pub vault: Vec<String>,
}

pub async fn new_user_db(path: &str) -> CustomResult<SqlitePool> {
    if !Path::new(path.clone()).exists() {
        File::create(path.clone()).map_err(CustomErrors::FileError)?;
    }
    let pool = SqlitePool::connect(format!("sqlite://{}", path).as_str())
        .await
        .map_err(CustomErrors::DBInitError)?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Users (
        Username CHAR,
        PasswordHash VARCHAR(512),
        Vault LONGTEXT,
        UNIQUE(Username)
      )",
    )
    .execute(&pool)
    .await
    .map_err(CustomErrors::QueryError)?;
    Ok(pool)
}

pub async fn add_user(pool: &SqlitePool, username: &str, password_hash: &str) -> CustomResult<()> {
    // doesn't check if user already exists
    sqlx::query("INSERT INTO Users (Username, PasswordHash, Vault) VALUES ($1, $2, $3)")
        .bind(username)
        .bind(password_hash)
        .bind(to_string(&Vec::<String>::new()).map_err(CustomErrors::VaultEncodeError)?)
        .execute(pool)
        .await
        .map_err(CustomErrors::RegisterError)?;
    Ok(())
}

pub async fn user_exists(pool: &SqlitePool, username: &str) -> CustomResult<bool> {
    Ok(sqlx::query("SELECT Username FROM Users WHERE Username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(CustomErrors::QueryError)?
        .is_some())
}

pub async fn get_user(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
) -> CustomResult<Option<StoredUser>> {
    // nasty
    sqlx::query("SELECT * FROM Users WHERE Username = ? AND PasswordHash = ?")
        .bind(username)
        .bind(password_hash)
        .fetch_optional(pool)
        .await
        .map_err(CustomErrors::QueryError)?
        .map_or(Ok(None), |row| {
            let vault: String = row.try_get(2).map_err(CustomErrors::RowDecodeError)?;
            Ok(Some(StoredUser {
                username: row.try_get(0).map_err(CustomErrors::RowDecodeError)?,
                password_hash: row.try_get(1).map_err(CustomErrors::RowDecodeError)?,
                vault: from_str(vault.as_ref()).map_err(CustomErrors::VaultDecodeError)?,
            }))
        })
}

pub async fn get_user_by_name(
    pool: &SqlitePool,
    username: &str,
) -> CustomResult<Option<StoredUser>> {
    // nasty
    sqlx::query("SELECT * FROM Users WHERE Username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(CustomErrors::QueryError)?
        .map_or(Ok(None), |row| {
            let vault: String = row.try_get(2).map_err(CustomErrors::RowDecodeError)?;
            Ok(Some(StoredUser {
                username: row.try_get(0).map_err(CustomErrors::RowDecodeError)?,
                password_hash: row.try_get(1).map_err(CustomErrors::RowDecodeError)?,
                vault: from_str(vault.as_ref()).map_err(CustomErrors::VaultDecodeError)?,
            }))
        })
}

#[cfg(test)]
mod database_tests {
    use super::*;
    #[tokio::test(flavor = "multi_thread")]
    async fn full_test() {
        let db_path = "test_db.db";
        let db = new_user_db(&db_path).await.unwrap();
        let user_added = add_user(&db, "test", "test").await;
        assert!(true, "{}", user_added.is_ok());
        let user_exists = user_exists(&db, "test").await;
        println!("{:?}", &user_exists);
        assert!(true, "{}", user_exists.unwrap());
        let user = get_user(&db, "test", "test").await;
        println!("User: {:?}", &user.as_ref().unwrap().as_ref().unwrap());
        assert!(true, "{}", user.as_ref().unwrap().is_some());
    }
}
