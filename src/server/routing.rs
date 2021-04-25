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
    use crate::server::database::StoredUser;
    use crate::utils::crypto::create_auth_token;
    use crate::{
        server::database,
        utils::{crypto, data, types::WarpResult},
    };
    use log::error;
    use sqlx::SqlitePool;
    use warp::http::StatusCode;
    use warp::reply::{json, with_status};
    use warp::{reject::custom as custom_reject, Reply};

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
            Ok(()) => {
                Ok(StatusCode::CREATED)
            }
            Err(e) => {
                error!("Error registering a user: {:?}", e);
                Ok(StatusCode::BAD_REQUEST)
            }
        }
    }

    /*
      Returns auth token for user.
      Takes the hash of the password, this should be hashed by the client.
    */
    pub async fn login(body: data::User, db: SqlitePool) -> WarpResult<Box<dyn Reply>> {
        let user = database::get_user_by_name(&db, &body.username)
            .await
            .map_err(|e| custom_reject(e))?;
        match user {
            Some(u) => {
                if crypto::check_hash(&body.password, &u.password_hash) {
                    Ok(Box::new(with_status(
                        create_auth_token(&body).map_err(|e| custom_reject(e))?,
                        StatusCode::OK,
                    )))
                } else {
                    Ok(Box::new(StatusCode::BAD_REQUEST))
                }
            }
            None => Ok(Box::new(StatusCode::BAD_REQUEST)),
        }
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
    use rand::{distributions::Alphanumeric, Rng};
    use warp;
    use warp::hyper::body::Bytes;
    use warp::http::StatusCode;

    #[tokio::test(flavor = "multi_thread")]
    async fn full_test() {
        let db = database::new_user_db("test_db.db").await.unwrap();
        let filter = get_routes(db.clone());
        let username: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let password = "AHHAHAHAHHA LAMOAOAOAOOA";
        let register_response = register(&username, &password.to_string(), &filter).await;
        assert_eq!(StatusCode::CREATED, register_response.status());
        let login_response = login(&username, &password.to_string(), &filter).await;
        assert_eq!(StatusCode::OK, login_response.status())
    }

    async fn login(
        username: &String,
        password: &String,
        filter: &(impl warp::Filter<Extract = (impl Reply,), Error = Infallible> + Clone + 'static),
    ) -> warp::http::Response<Bytes> {
        // sadge
        warp::test::request()
            .path("/api/login")
            .method("POST")
            .json(&data::User {
                username: String::from(username.clone()),
                password: password.to_string(),
            })
            .reply(filter)
            .await
    }

    async fn register(
        username: &String,
        password: &String,
        filter: &(impl warp::Filter<Extract = (impl Reply,), Error = Infallible> + Clone + 'static),
    ) -> warp::http::Response<Bytes> {
        warp::test::request()
            .path("/api/register")
            .method("POST")
            .json(&data::User {
                username: String::from(username.clone()),
                password: password.to_string(),
            })
            .reply(filter)
            .await
    }
}
