use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::user::{User, UserBmc};


type Db = Pool<Postgres>;


// Hardcode to prevent deployed system db update
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

// SQL files
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql\\dev-initial";

const DEMO_PWD: &str = "welcome";


/// Recreate the database
pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_db", "FOR-DEV-ONLY");

    // Get the sql dir
    let current_dir = std::env::current_dir().unwrap();
    let v: Vec<_> = current_dir.components().collect();
    let path_comp = v.get(v.len().wrapping_sub(3));
    let base_dir = if Some(true) == path_comp.map(|c| c.as_os_str() == "crates") {
        v[..v.len() - 3].iter().collect::<PathBuf>()
    } else {
        current_dir.clone()
    };
    let sql_dir = base_dir.join(SQL_DIR);

    // create the app_db/app_user with the postgers user
    {
        // because root_db has full privilage on the database, we want to limit its scope
        let sql_recreate_db_file = sql_dir.join(SQL_RECREATE_DB_FILE_NAME);
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pexec(&root_db, &sql_recreate_db_file).await?;
    }

    // get sql files
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    // by doing the sort we make sure to have them in order 00-*.sql, 01-*.sql, ...
    paths.sort();

    let app_db = new_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        let path_str = path.to_string_lossy();

        // only take the .sql and skip the SQL_RECREATE_DB
        if path_str.ends_with(".sql") && !path_str.ends_with(SQL_RECREATE_DB_FILE_NAME) {
            pexec(&app_db, &path).await?;
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
async fn pexec(db: &Db, file: &Path) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file:?}", "FOR-DEV-ONLY");

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
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_connection_url)
        .await
}
