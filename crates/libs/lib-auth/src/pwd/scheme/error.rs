use serde::Serialize;
use std::fmt::Display;


pub type Result<T> = core::result::Result<T, Error>;


#[derive(Debug, Serialize)]
pub enum Error {
    Key,
    Salt,
    Hash,
    PwdValidate,
    SchemeNotFound(String),
}


impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
