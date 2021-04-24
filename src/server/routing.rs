use crate::utils::errors;
use sqlx::SqlitePool;
use std::convert::Infallible;
use warp::*;

pub fn get_routes(
    db: SqlitePool,

) -> impl warp::Filter<Extract = (impl Reply,), Error = Infallible> + Clone {
    let api = warp::path("api");

    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 64))
        .and(filters::with_db(db.clone()))
        .and_then(handlers::register);

    let login = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::body::content_length_limit(1024 * 64))
        .and(filters::with_db(db.clone()))
        .and_then(handlers::login);

    api.and(register.or(login))
        .recover(errors::handle_rejection)
}

pub mod handlers {
    use crate::{
        server::database,
        utils::{crypto, data, types::WarpResult},
    };
    use sqlx::SqlitePool;
    use warp::http::StatusCode;
    use warp::reply::{json, with_status};
    use warp::{reject::custom as custom_reject, Reply};
    use log::error;

    /*
      Returns true or false depending on whether or not the user was successfully registered.
      Password should be a hash of the actual password.
    */
    pub async fn register(body: data::User, db: SqlitePool) -> WarpResult<impl Reply> {
        let password_hash = crypto::create_hash(&body.password).map_err(|e| custom_reject(e))?;
        let user_added = database::add_user(&db, &*body.username, &*password_hash)
            .await
            .map_err(|e| custom_reject(e));
        match user_added {
            Ok(()) => Ok(with_status(
                json(&data::Response {
                    id: "RegistrationSuccess".to_string(),
                    message: "Successfully registered.".to_string(),
                }),
                StatusCode::OK,
            )),
            Err(e) => {
                error!("Error registering a user: {:?}", e);
                Ok(with_status(
                    json(&data::Response {
                        id: "RegistrationError".to_string(),
                        message: "Error registering user".to_string(),
                    }),
                    StatusCode::BAD_REQUEST,
                ))
            }
        }
    }

    /*
      Returns auth token for user.
      Takes the hash of the password, this should be hashed by the client.
    */
    pub async fn login(body: data::User, db: SqlitePool) -> WarpResult<impl Reply> {
        Ok(json(&data::Response {
            id: "NotImplemented".to_string(),
            message: "Not currently implemented".to_string(),
        }))
    }
}

pub mod filters {
    use sqlx::SqlitePool;
    use std::convert::Infallible;
    use warp::Filter;

    pub fn with_db(
        db: SqlitePool,
    ) -> impl Filter<Extract = (SqlitePool,), Error = Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

#[cfg(test)]
mod filter_tests {
    use super::*;
    use crate::server::database;
    use crate::utils::data;
    use crate::utils::data::Response;
    use rand::{distributions::Alphanumeric, Rng};
    use warp;

    #[tokio::test(flavor = "multi_thread")]
    async fn full_test() {
        let db = database::new_user_db("test_db.db").await.unwrap();
        let filter = get_routes(db.clone());
        let result = warp::test::request()
            .path("/api/register")
            .method("POST")
            .json(&data::User {
                username: rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect(),
                password: "password".to_string(),
            })
            .reply(&filter)
            .await;
        let body_text = String::from_utf8(result.body().to_vec()).unwrap();
        println!("{:?}", &body_text);
        let response: Response = serde_json::from_str(&body_text).unwrap();
        println!("{:?}", response.id);
        assert_eq!("RegistrationSuccess", response.id);
    }
}
