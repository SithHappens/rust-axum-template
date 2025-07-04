mod error;

use std::time::Duration;

pub use self::error::{Error, Result};

use crate::core_config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub type Db = Pool<Postgres>;


/// Create new database pool
pub async fn new_db_pool() -> Result<Db> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&core_config().DB_URL)
        .await
        .map_err(|err| Error::FailToCreatePool(err.to_string()))
}
