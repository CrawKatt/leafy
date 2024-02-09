use anyhow::{anyhow, Error};
use std::fmt;
use serde::ser::StdError;
use thiserror::Error;
use crate::log_handle;

#[derive(Error, Debug)]
pub enum UnwrapErrors {
    #[error(transparent)]
    Unwrap(#[from] UnwrapLogError),

    #[error(transparent)]
    Surreal(#[from] surrealdb::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
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

//impl Error for UnwrapLogError<'_> {}

pub trait UnwrapLog<T> {
    fn unwrap_log(self, msg: &'static str, module: &str, line: u32) -> Result<T, Error>;
}

impl<T> UnwrapLog<T> for Option<T>
    where
        T: Default,
{
    fn unwrap_log(self, msg: &'static str, module: &str, line: u32) -> Result<T, Error> {
        self.map_or_else(move || {
            log_handle!("{msg} : Caller: `{module}` Line {line}");
            Err(anyhow!(UnwrapLogError { msg }))
        }, move |t| Ok(t))
    }
}

impl<T, E> UnwrapLog<T> for Result<T, E>
    where
        T: Default,
        E: StdError,
{
    fn unwrap_log(self, msg: &'static str, module: &str, line: u32) -> Result<T, Error> {
        match self {
            Ok(t) => Ok(t),
            Err(why) => {
                log_handle!("{msg}: {why} : `{module}` Line {line}");
                Err(anyhow!(UnwrapLogError { msg }))
            }
        }
    }
}

#[macro_export]
macro_rules! unwrap_log {
    ($expr:expr, $msg:expr) => {
        {
            use $crate::utils::debug::UnwrapLogError;
            use $crate::log_handle;
            use anyhow::anyhow;

            match $expr {
                Some(val) => val,
                None => {
                    log_handle!("{} : Caller: `{}` Line {}", $msg, file!(), line!());
                    return Err(anyhow!(UnwrapLogError { msg: $msg }).into());
                }
            }
        }
    };
    ($expr:expr, $msg:expr) => {
        {
            use $crate::utils::debug::UnwrapLogError;
            use $crate::log_handle;
            use anyhow::anyhow;

            match $expr {
                Ok(val) => val,
                Err(why) => {
                    log_handle!("{}: {} : `{}` Line {}", $msg, why, file!(), line!());
                    return Err(anyhow!(UnwrapLogError { msg: $msg }).into());
                }
            }
        }
    };
}