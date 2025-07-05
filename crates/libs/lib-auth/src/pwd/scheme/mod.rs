mod error;
mod scheme_01;
mod scheme_02;

pub use self::error::{Error, Result};

use crate::pwd::ContentToHash;


pub const DEFAULT_SCHEME: &str = "02";


pub trait Scheme {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String>;
    fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: &str) -> Result<()>;
}


#[derive(Debug)]
pub enum SchemeStatus {
    /// The password uses the latest scheme. All good.
    Ok,
    /// The password uses an old scheme.
    Outdated,
}


/// Get the currently used scheme
pub fn get_scheme(scheme_name: &str) -> Result<Box<dyn Scheme>> {
    match scheme_name {
        "01" => Ok(Box::new(scheme_01::Scheme01)),
        "02" => Ok(Box::new(scheme_02::Scheme02)),
        _ => Err(Error::SchemeNotFound(scheme_name.to_string())),
    }
}
