use std::num::ParseIntError;

use serde::ser::StdError;
use thiserror::Error;

pub type UnwrapResult<T> = Result<T, UnwrapErrors>;

#[derive(Error, Debug)]
pub enum UnwrapErrors {
    #[error("Value was None: {0}")]
    DebugNone(String),
    
    #[error("Value was None")]
    NoneError,

    #[error(transparent)]
    Surreal(#[from] surrealdb::Error),

    #[error(transparent)]
    ParseInt(#[from] ParseIntError),

    #[error(transparent)]
    Serenity(#[from] serenity::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Tokio(#[from] tokio::io::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),
}

pub trait IntoUnwrapResult<T> {
    fn into_result(self) -> UnwrapResult<T>;
}

impl<T> IntoUnwrapResult<T> for Option<T> {
    fn into_result(self) -> UnwrapResult<T> {
        self.ok_or(UnwrapErrors::NoneError)
    }
}

pub trait UnwrapLog<T> {
    fn unwrap_log(self, caller: String) -> UnwrapResult<T>;
}

impl<T> UnwrapLog<T> for Option<T> {
    fn unwrap_log(self, caller: String) -> UnwrapResult<T> {
        self.map_or_else(move || {
            Err(UnwrapErrors::DebugNone(caller))
        }, move |t| Ok(t))
    }
}

impl<T, E> UnwrapLog<T> for Result<T, E>
    where
        T: Default,
        E: StdError,
{
    fn unwrap_log(self, caller: String) -> UnwrapResult<T> {
        match self {
            Ok(t) => Ok(t),
            Err(why) => {
                println!("Error: {why}");
                Err(UnwrapErrors::DebugNone(caller))
            }
        }
    }
}