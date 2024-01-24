use std::error::Error;
use std::fmt;
use serde::ser::StdError;
use crate::log_handle;

#[derive(Debug)]
pub struct UnwrapLogError<'a> {
    msg: &'a str,
}

impl fmt::Display for UnwrapLogError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UnwrapLogError: {}", self.msg)
    }
}

impl Error for UnwrapLogError<'_> {}

pub trait UnwrapLog<T> {
    fn unwrap_log(self, msg: &str) -> Result<T, UnwrapLogError>;
}

impl<T: Default> UnwrapLog<T> for Option<T> {
    fn unwrap_log(self, msg: &str) -> Result<T, UnwrapLogError> {
        self.map_or_else(|| {
            log_handle!("{msg}");
            Err(UnwrapLogError { msg })
        }, |t| Ok(t))
    }
}

impl<T: Default, E: StdError> UnwrapLog<T> for Result<T, E> {
    fn unwrap_log(self, msg: &str) -> Result<T, UnwrapLogError> {
        match self {
            Ok(t) => Ok(t),
            Err(why) => {
                log_handle!("{msg}: {why}");
                Err(UnwrapLogError { msg })
            }
        }
    }
}