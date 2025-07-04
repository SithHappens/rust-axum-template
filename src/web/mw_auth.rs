//! Middleware for authentication

use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForAuth};
use crate::model::ModelManager;
use crate::token::{validate_web_token, Token};
use crate::web::{set_token_cookie, Error, Result, AUTH_TOKEN};
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::RequestPartsExt;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

/// Just checks there is no error in the context.
#[allow(dead_code)]
pub async fn mw_ctx_require(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    debug!("{:<12} - mw_require_context - {:?}", "MIDDLEWARE", ctx);

    ctx?;

    Ok(next.run(req).await)
}


/// Resolves and creates the context, updates and validates the auth token, then put it in the
/// request extension
pub async fn mw_ctx_resolver(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolver", "MIDDLEWARE");

    // This should not fail
    let ctx_ext_result = _ctx_resolve(mm, &cookies).await;

    // If we have an error, we want to remove it from the cookies. We don't want to invalidate
    // the same token over and over.
    if ctx_ext_result.is_err() && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from("AUTH_TOKEN"));
    }

    // Store the CtxExtResult in the request extension for Ctx extractor
    req.extensions_mut().insert(ctx_ext_result);

    Ok(next.run(req).await)
}


async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    // Get token string
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    // Parse token
    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    // Get UserForAuth
    let user: UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
        .await
        .map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    // Validate token
    validate_web_token(&token, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::FailValidate)?;

    // Update token
    set_token_cookie(cookies, &user.username, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

    // Create CtxExtResult
    Ctx::new(user.id).map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}


// region:    --- Ctx Extractor

/// Axum extractor for Ctx
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!("{:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)
    }
}

// endregion: --- Ctx Extractor

type CtxExtResult = core::result::Result<Ctx, CtxExtError>;


#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
