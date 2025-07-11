mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_rpc;
pub mod routes_static;

pub use self::error::{ClientError, Error, Result};
use lib_auth::token::generate_web_token;
use tower_cookies::{Cookie, Cookies};


pub const AUTH_TOKEN: &str = "auth-token";


fn set_token_cookie(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
    let token = generate_web_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    cookie.set_http_only(true); // Increase security by preventing JavaScript access
    cookie.set_path("/"); // default path is the URI path of the request ('/api/login')

    cookies.add(cookie);

    Ok(())
}


fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
    let mut cookie = Cookie::from(AUTH_TOKEN);
    cookie.set_path("/");
    cookies.remove(cookie);

    Ok(())
}
