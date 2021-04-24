use crate::utils::errors::CustomErrors;
use std::error::Error;
use warp;

pub type BoxResult<T> = Result<T, Box<dyn Error>>;
pub type CustomResult<T> = Result<T, CustomErrors>;
pub type WarpResult<T> = std::result::Result<T, warp::Rejection>;
