use serde::ser::StdError;
use crate::log_handle;

pub trait UnwrapLog<T> {
    fn unwrap_log(self, msg: &str) -> T;
}

impl<T: Default> UnwrapLog<T> for Option<T> {
    fn unwrap_log(self, msg: &str) -> T {
        self.map_or_else(|| {
                log_handle!("{msg}");
                Default::default()
            }, |t| t)
    }
}

impl<T: Default, E: StdError> UnwrapLog<T> for Result<T, E> {
    fn unwrap_log(self, msg: &str) -> T {
        self.unwrap_or_else(|why| {
            log_handle!("{msg}: {why}");
            Default::default()
        })
    }
}