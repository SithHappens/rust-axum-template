use crate::web;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use lib_auth::{pwd, token};
use lib_core::model;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};
use std::sync::Arc;
use tracing::debug;


pub type Result<T> = core::result::Result<T, Error>;


#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, From)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // Login
    LoginFailUsernameNotFound,
    LoginFailUserHasNoPwd {
        user_id: i64,
    },
    LoginFailPwdNotMaching {
        user_id: i64,
    },

    // Context external error
    CtxExt(web::mw_auth::CtxExtError),

    // Modules
    #[from]
    Model(model::Error),
    #[from]
    Pwd(pwd::Error),
    #[from]
    Token(token::Error),
    #[from]
    Rpc(lib_rpc::Error),

    // External Modules
    #[from]
    SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!("{:<12} - model::Error {self:?}", "INTO_RES");

        // Create a placeholder Axum reponse.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the reponse.
        response.extensions_mut().insert(Arc::new(self));

        response
    }
}


impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}


impl std::error::Error for Error {}


// region:    --- Client Error

/// From the root error to the http status code and ClientError
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use web::Error::*;

        #[allow(unreachable_patterns)]
        match self {
            // Login
            LoginFailUsernameNotFound
            | LoginFailUserHasNoPwd { .. }
            | LoginFailPwdNotMaching { .. } => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // Auth
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // Model
            Model(model::Error::EntityNotFound { entity, id }) => (
                StatusCode::BAD_REQUEST,
                ClientError::ENTITY_NOT_FOUND { entity, id: *id },
            ),

            // Fallback: The lazy path is the safe path. If we don't specify the client error,
            // don't leak any information.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}


#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    SERVICE_ERROR,
}

// endregion: --- Client Error
