//! Context is the information you get from the request or anything, that identifies the user.

mod error;

pub use self::error::{Error, Result};


#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: i64,
}


impl Ctx {
    /// Ctx gets the concept of root that the system will use to make its calls.
    ///
    /// This is used when we don't have a user. When we have a user, we can use [`new`].
    pub fn root_ctx() -> Self {
        Ctx { user_id: 0 }
    }

    /// Here the [`user_id`] cannot be 0, because it is reserved for root
    pub fn new(user_id: i64) -> Result<Self> {
        if user_id == 0 {
            Err(Error::CtxCannotNewRootCtx)
        } else {
            Ok(Self { user_id })
        }
    }

    pub fn user_id(&self) -> i64 {
        self.user_id
    }
}
