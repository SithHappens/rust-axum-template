use crate::model::store;
use derive_more::From;
use lib_auth::pwd;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};


pub type Result<T> = core::result::Result<T, Error>;


#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
    EntityNotFound {
        entity: &'static str,
        id: i64,
    },
    ListLimitOverMax {
        max: i64,
        actual: i64,
    },

    // Modules
    #[from]
    Pwd(pwd::Error),
    #[from]
    Store(store::Error),

    // External
    #[from]
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),

    #[from]
    SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),

    #[from]
    ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] modql::filter::IntoSeaError),
}


impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{self:?}")
    }
}


impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Store(e) => Some(e),
            Error::Sqlx(e) => Some(e),
            _ => None,
        }
    }
}
