use std::fmt;
use std::num::ParseIntError;
use serde::ser::StdError;
use thiserror::Error;
use crate::log_handle;

pub type UnwrapResult<T> = Result<T, UnwrapErrors>;

#[derive(Error, Debug)]
pub enum UnwrapErrors {
    #[error(transparent)]
    Unwrap(#[from] UnwrapLogError),

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
}

#[derive(Error, Debug)]
pub struct UnwrapLogError {
    pub msg: &'static str,
}

impl fmt::Display for UnwrapLogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UnwrapLogError: {}", self.msg)
    }
}

pub trait UnwrapLog<T> {
    fn unwrap_log(self, msg: &'static str, module: &str, line: u32) -> Result<T, UnwrapLogError>;
}

impl<T> UnwrapLog<T> for Option<T> {
    fn unwrap_log(self, msg: &'static str, module: &str, line: u32) -> Result<T, UnwrapLogError> {
        self.map_or_else(move || {
            log_handle!("{msg} : Caller: `{module}` Line {line}");
            Err(UnwrapLogError { msg })
        }, move |t| Ok(t))
    }
}

impl<T, E> UnwrapLog<T> for Result<T, E>
    where
        T: Default,
        E: StdError,
{
    fn unwrap_log(self, msg: &'static str, module: &str, line: u32) -> Result<T, UnwrapLogError> {
        match self {
            Ok(t) => Ok(t),
            Err(why) => {
                log_handle!("{msg}: {why} : `{module}` Line {line}");
                Err(UnwrapLogError { msg })
            }
        }
    }
}

#[macro_export]
macro_rules! unwrap_log {
    ($expr:expr, $msg:expr) => {
        {
            use $crate::utils::misc::debug::UnwrapLogError;
            use $crate::log_handle;

            match $expr {
                Some(val) => val,
                None => {
                    log_handle!("{} : Caller: `{}` Line {}", $msg, file!(), line!());
                    return Err(Box::new(UnwrapLogError { msg: $msg }));
                }
            }
        }
    };
    ($expr:expr, $msg:expr) => {
        {
            use $crate::utils::misc::debug::UnwrapLogError;
            use $crate::log_handle;

            match $expr {
                Ok(val) => val,
                Err(why) => {
                    log_handle!("{}: {} : `{}` Line {}", $msg, why, file!(), line!());
                    return Err(UnwrapLogError { msg: $msg });
                }
            }
        }
    };
}