mod error;

pub use self::error::{Error, Result};

use crate::config::auth_config;
use hmac::{Hmac, Mac};
use lib_utils::b64::{b64u_decode_to_string, b64u_encode};
use lib_utils::time::{now_utc, now_utc_plus_sec_str, parse_utc};
use sha2::Sha512;
use std::fmt::Display;
use std::str::FromStr;


// region:    --- Token Type

/// String format: `ident_b64u.exp_b64u.sign_b64u`
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Token {
    /// Identifier, username for example, base64url encoded
    pub ident: String,
    /// Expiration date in Rfc3339, base64url encoded
    pub exp: String,
    /// Signature, base64url encoded
    pub sign_b64u: String,
}


impl FromStr for Token {
    type Err = Error;

    fn from_str(token_str: &str) -> Result<Self> {
        let splits: Vec<&str> = token_str.split('.').collect();
        if splits.len() != 3 {
            return Err(Error::InvalidFormat);
        }
        let (ident_b64u, exp_b64u, sign_b64u) = (splits[0], splits[1], splits[2]);

        Ok(Self {
            ident: b64u_decode_to_string(ident_b64u).map_err(|_| Error::CannotDecodeIdent)?,
            exp: b64u_decode_to_string(exp_b64u).map_err(|_| Error::CannotDecodeExp)?,
            sign_b64u: sign_b64u.to_string(),
        })
    }
}


impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            b64u_encode(&self.ident),
            b64u_encode(&self.exp),
            self.sign_b64u
        )
    }
}

// endregion: --- Token Type


// region:    --- Web Token and Gen Validation

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
    let config = &auth_config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(origin_token: &Token, salt: &str) -> Result<()> {
    let config = &auth_config();
    _validate_token_sign_and_exp(origin_token, salt, &config.TOKEN_KEY)
}

// endregion: --- Web Token and Gen Validation


// region:    --- (private) Token Gen and Validation

fn _generate_token(ident: &str, duration_sec: f64, salt: &str, key: &[u8]) -> Result<Token> {
    // create first two components
    let ident = ident.to_string();
    let exp = now_utc_plus_sec_str(duration_sec);

    // sign first two components
    let sign_b64u = _token_sign_into_b64u(&ident, &exp, salt, key)?;

    Ok(Token {
        ident,
        exp,
        sign_b64u,
    })
}

/// Returns Err if validatetion fails.
fn _validate_token_sign_and_exp(origin_token: &Token, salt: &str, key: &[u8]) -> Result<()> {
    // Validate signature
    let new_sign_b64u = _token_sign_into_b64u(&origin_token.ident, &origin_token.exp, salt, key)?;

    // Compare the new signature with the original one
    if new_sign_b64u != origin_token.sign_b64u {
        return Err(Error::SignatureNotMatching);
    }

    // Validate expiration
    let origin_exp = parse_utc(&origin_token.exp).map_err(|_| Error::ExpNotIso)?;
    let now = now_utc();

    if origin_exp < now {
        return Err(Error::Expired);
    }

    Ok(())
}

/// Create token signature from token parts and salt.
fn _token_sign_into_b64u(ident: &str, exp: &str, salt: &str, key: &[u8]) -> Result<String> {
    let content = format!("{}.{}", b64u_encode(ident), b64u_encode(exp));

    // Create a HMAC-SHA-512 from key
    let mut hmac_sha512 =
        Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::HmacFailNewFromSlice)?;

    // Add content
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // Finalize and base64url encode
    let hmac_result = hmac_sha512.finalize();
    let result_bytes = hmac_result.into_bytes();
    let result = b64u_encode(result_bytes);

    Ok(result)
}

// endregion: --- (private) Token Gen and Validation


// region:    --- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_token_display_ok() -> Result<()> {
        // Fixture
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyNS0wNy0wMVQxMjowMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            ident: "fx-ident-01".to_string(),
            exp: "2025-07-01T12:00:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // Check
        //println!("Token: {}", fx_token); // to generate the fx_token_str in the first place
        assert_eq!(fx_token.to_string(), fx_token_str);

        Ok(())
    }

    #[test]
    fn test_token_from_str_ok() -> Result<()> {
        // Fixture
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyNS0wNy0wMVQxMjowMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            ident: "fx-ident-01".to_string(),
            exp: "2025-07-01T12:00:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // Execute
        let token: Token = fx_token_str.parse()?;

        // Check
        assert_eq!(token, fx_token);

        Ok(())
    }

    #[test]
    fn test_validate_web_token_ok() -> Result<()> {
        // Setup and Fixtures
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.02; // 20 ms
        let token_key = &auth_config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // Execute
        thread::sleep(Duration::from_millis(10));
        let res = validate_web_token(&fx_token, fx_salt);

        // Check
        res?;

        Ok(())
    }

    /// Test if token expires properly.
    #[test]
    fn test_validate_web_token_err_expired() -> Result<()> {
        // Setup and Fixtures
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.01; // 10 ms
        let token_key = &auth_config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // Execute
        thread::sleep(Duration::from_millis(20));
        let res = validate_web_token(&fx_token, fx_salt);

        // Check
        assert!(
            matches!(res, Err(Error::Expired)),
            "Should have matched `Err(Error::TokenExpired)` but was {res:?}"
        );

        Ok(())
    }
}

// endregion: --- Tests
