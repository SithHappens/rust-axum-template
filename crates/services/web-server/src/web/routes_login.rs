use crate::web::{self, Error, Result, remove_token_cookie};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use lib_auth::pwd;
use lib_auth::pwd::ContentToHash;
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use lib_core::model::user::{UserBmc, UserForLogin};
use serde::Deserialize;
use serde_json::{Value, json};
use tower_cookies::{Cookie, Cookies};
use tracing::debug;


pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login_handler))
        .route("/api/logoff", post(api_logoff_handler))
        .with_state(mm)
}


async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_login_handler", "HANDLER");

    let LoginPayload {
        username,
        password: pwd_clear,
    } = payload;
    let root_ctx = Ctx::root_ctx();

    // get the user
    // IMPORTANT: we do not want to log the username anywhere. The reason is that sometimes users enter passwords
    // as usernames by mistake. So when it is not found, we do not want to capture it.
    let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;
    let user_id = user.id;

    // Validate the password
    let Some(pwd) = user.pwd else {
        return Err(Error::LoginFailUserHasNoPwd { user_id });
    };

    pwd::validate_pwd(
        &ContentToHash {
            content: pwd_clear.clone(),
            salt: user.pwd_salt,
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMaching { user_id })?;

    web::set_token_cookie(&cookies, &user.username, &user.token_salt.to_string())?;

    // Create the success body.
    let body = Json(json!(
        {
            "result": {
                "success": true
            }
        }
    ));

    Ok(body)
}


async fn api_logoff_handler(
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_logoff_handler", "HANDLER");
    let should_logoff = payload.logoff;

    if should_logoff {
        remove_token_cookie(&cookies)?;
    }

    // Create the success body
    let body = Json(json!({ "result": {
        "logged_off": should_logoff
    } }));

    Ok(body)
}


#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}


#[derive(Debug, Deserialize)]
struct LogoffPayload {
    logoff: bool,
}
