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
        let id;
        let message;
        let status_code;
        let password_hash = crypto::create_hash(&body.password).map_err(|e| custom_reject(e))?;
        let user_added = database::add_user(&db, &*body.username, &*password_hash)
            .await
            .map_err(|e| custom_reject(e));
        match user_added {
            Ok(()) => {
                id = String::from("RegistrationSuccess");
                message = String::from("Successfully registered.");
                status_code = StatusCode::OK;
            }
            Err(e) => {
                error!("Error registering a user: {:?}", e);
                id = String::from("RegistrationError");
                message = String::from("Error registering user.");
                status_code = StatusCode::BAD_REQUEST;
            }
        }
        Ok(with_status(
            json(&data::Response { id, message }),
            status_code,
        ))
    }

    /*
      Returns auth token for user.
      Takes the hash of the password, this should be hashed by the client.
    */
    pub async fn login(body: data::User, db: SqlitePool) -> WarpResult<impl Reply> {
        let id;
        let message;
        let status_code;
        let user = database::get_user_by_name(&db, &body.username)
            .await
            .map_err(|e| custom_reject(e))?;
        match user {
            Some(u) => {
                if crypto::check_hash(&body.password, &u.password_hash) {
                    id = String::from("LoginSuccess");
                    message = create_auth_token(&body).map_err(|e| custom_reject(e))?;
                    status_code = StatusCode::OK;
                } else {
                    id = String::from("LoginError");
                    message = String::from("Unable to login");
                    status_code = StatusCode::OK;
                }
            }
            None => {
                id = String::from("NoSuchUser");
                message = String::from("The user specified does not exist.");
                status_code = StatusCode::BAD_REQUEST
            }
        }
        Ok(with_status(
            json(&data::Response { id, message }),
            status_code,
        ))
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
        let username: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let password = "AHHAHAHAHHA LAMOAOAOAOOA";
        let register_response = register(&username, &password.to_string(), &filter).await;
        assert_eq!("RegistrationSuccess", register_response.id);
        let login_response = login(&username, &password.to_string(), &filter).await;
        assert_eq!("LoginSuccess", login_response.id)
    }

    async fn login(
        username: &String,
        password: &String,
        filter: &(impl warp::Filter<Extract = (impl Reply,), Error = Infallible> + Clone + 'static),
    ) -> Response {
        // sadge
        let result = warp::test::request()
            .path("/api/login")
            .method("POST")
            .json(&data::User {
                username: String::from(username.clone()),
                password: password.to_string(),
            })
            .reply(filter)
            .await;
        let body_text = String::from_utf8(result.body().to_vec()).unwrap();
        serde_json::from_str::<Response>(&body_text).unwrap()
    }

    async fn register(
        username: &String,
        password: &String,
        filter: &(impl warp::Filter<Extract = (impl Reply,), Error = Infallible> + Clone + 'static),
    ) -> Response {
        let result = warp::test::request()
            .path("/api/register")
            .method("POST")
            .json(&data::User {
                username: String::from(username.clone()),
                password: password.to_string(),
            })
            .reply(filter)
            .await;
        let body_text = String::from_utf8(result.body().to_vec()).unwrap();
        serde_json::from_str::<Response>(&body_text).unwrap()
    }
}
