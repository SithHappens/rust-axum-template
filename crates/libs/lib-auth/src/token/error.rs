use serde::Serialize;


pub type Result<T> = core::result::Result<T, Error>;


#[derive(Debug, Serialize)]
pub enum Error {
    HmacFailNewFromSlice,

    InvalidFormat,
    CannotDecodeIdent,
    CannotDecodeExp,
    SignatureNotMatching,
    ExpNotIso,
    Expired,
}


impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
