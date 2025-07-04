use derive_more::From;
use lib_core::model;


pub type Result<T> = core::result::Result<T, Error>;
//pub type Error = Box<dyn std::error::Error>;


#[derive(Debug, From)]
pub enum Error {
    #[from]
    Model(model::Error),
}


impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{self:?}")
    }
}


impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Model(e) => Some(e),
            _ => None,
        }
    }
}
