use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

use crate::ctx::Ctx;
use crate::model::user::{User, UserBmc};
use crate::model::ModelManager;


type Db = Pool<Postgres>;


// Hardcode to prevent deployed system db update
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

// SQL files
const SQL_RECREATE_DB: &str = "sql/dev-initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev-initial";

const DEMO_PWD: &str = "welcome";


/// Recreate the database
pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_db", "FOR-DEV-ONLY");

    {
        // create the app_db/app_user with the postgers user
        // because root_db has full privilage on the database, we want to limit its scope
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pexec(&root_db, SQL_RECREATE_DB).await?;
    }

    // get sql files
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    // by doing the sort we make sure to have them in order 00-*.sql, 01-*.sql, ...
    paths.sort();

    let app_db = new_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/"); // for windows

            // only take the .sql and skip the SQL_RECREATE_DB
            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                pexec(&app_db, &path).await?;
            }
        }
    }

    // Init model layer
    let mm = ModelManager::new().await?;
    let ctx = Ctx::root_ctx();

    // Set demo1 password
    let demo1_user: User = UserBmc::first_by_username(&ctx, &mm, "demo1")
        .await?
        .unwrap();
    UserBmc::update_pwd(&ctx, &mm, demo1_user.id, DEMO_PWD).await?;
    info!("{:<12} - init_dev_db - set demo1 password", "FOR-DEV-ONLY");

    Ok(())
}


/// Execute one sql file
async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file}", "FOR-DEV-ONLY");

    let content = fs::read_to_string(file)?;

    // FIXME make the split more sql proof
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }

    Ok(())
}


/// Create database pool
async fn new_db_pool(db_connection_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        //.test_before_acquire(false)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_connection_url)
        .await
}
