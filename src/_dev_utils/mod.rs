mod dev_db;

use crate::ctx::Ctx;
use crate::model::task::{Task, TaskBmc, TaskForCreate};
use crate::model::{self, ModelManager};
use tokio::sync::OnceCell;
use tracing::info;


/*
/// Initialize environment for local development.
///
/// (for early development, will be called from main())
pub async fn init_dev() {
    static INIT: OnceCell<()> = OnceCell::const_new();
    INIT.get_or_init(|| async {
        info!("{:<12} - init_dev", "FOR_DEV_ONLY");

        dev_db::init_dev_db().await.unwrap();
    })
    .await;
}


/// Initialize test environment
pub async fn init_test() -> ModelManager {
    static INIT: OnceCell<ModelManager> = OnceCell::const_new();

    let mm = INIT
        .get_or_init(|| async {
            init_dev().await;
            ModelManager::new().await.unwrap()
        })
        .await;

    mm.clone() // fine because the ModelManager is built to be clonable, everything will be below Arc
}
*/


// Because our unit tests fail with Error: Sqlx(PoolTimedOut) randomly, and it seems to be this issue:
// https://github.com/launchbadge/sqlx/issues/2567#issuecomment-2009849261
// we recreate the dev database for every test for now by dropping the static
pub async fn init_dev() {
    info!("{:<12} - init_dev", "FOR_DEV_ONLY");

    dev_db::init_dev_db().await.unwrap();
}


pub async fn init_test() -> ModelManager {
    init_dev().await;
    ModelManager::new().await.unwrap()
}

/// For unit tests
pub async fn seed_tasks(ctx: &Ctx, mm: &ModelManager, titles: &[&str]) -> model::Result<Vec<Task>> {
    let mut tasks = Vec::new();

    for title in titles {
        let id = TaskBmc::create(
            ctx,
            mm,
            TaskForCreate {
                title: title.to_string(),
            },
        )
        .await?;

        let task = TaskBmc::get(ctx, mm, id).await?;

        tasks.push(task);
    }

    Ok(tasks)
}
