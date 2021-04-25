use log;
use serde::Serialize;
use sqlx;
use std::convert::Infallible;
use thiserror::Error;
use warp::http::StatusCode;
use warp::reply::{json, with_status};

#[derive(Error, Debug)]
pub enum CustomErrors {
    #[error("Error executing query: {0}")]
    QueryError(sqlx::Error),
    #[error("Error while working with a file: {0}")]
    FileError(std::io::Error),
    #[error("Error connecting to database: {0}")]
    DBInitError(sqlx::Error),
    #[error("Error decoding row, this is a problem: {0}")]
    RowDecodeError(sqlx::Error),
    #[error("Error decoding users vault, shit: {0}")]
    VaultDecodeError(serde_json::Error),
    #[error("Error encoding users vault, nope: {0}")]
    VaultEncodeError(serde_json::Error),
    #[error("Error registering user: {0}")]
    RegisterError(sqlx::Error),
    #[error("Error encoding jwt token: {0}")]
    JWTEncodingError(jsonwebtoken::errors::Error),
    #[error("Unexpected error")]
    UnexpectedError,
}

impl warp::reject::Reject for CustomErrors {}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(e) = err.find::<CustomErrors>() {
        match e {
            CustomErrors::QueryError(_) => {
                log::error!("Query error: {:?}", err);
                code = StatusCode::BAD_REQUEST;
                message = "Could not execute request";
            }
            _ => {
                log::error!("unhandled application error: {:?}", err);
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "Internal Server Error";
            }
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        log::error!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    Ok(with_status(
        json(&ErrorResponse {
            message: message.into(),
        }),
        code,
    ))
}
