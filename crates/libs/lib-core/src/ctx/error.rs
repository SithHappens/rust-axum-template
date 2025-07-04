use serde::Serialize;


pub type Result<T> = core::result::Result<T, Error>;


#[derive(Debug, Serialize)]
pub enum Error {
    CtxCannotNewRootCtx,
}


impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{self:?}")
    }
}


impl std::error::Error for Error {}
