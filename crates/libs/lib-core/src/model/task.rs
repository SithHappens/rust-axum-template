//! Design
//! - structs are views on database tables, can be only a subset of what you have in the database,
//!   e.g. in [`TaskForCreate`] we don't want people to be able to reset the id of a task.

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::{ModelManager, Result, base};
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsBool, OpValsInt64, OpValsString};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;


// sqlx::FromRow is for reading. sqlb::Fields is for writing and updating and allows for creation of sql using the
// builder pattern
#[derive(Debug, Clone, FromRow, Serialize, Fields)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

/// For when we create a task
#[derive(Deserialize, Fields)]
pub struct TaskForCreate {
    pub title: String,
}

/// For when we update a task
#[derive(Default, Deserialize, Fields)]
pub struct TaskForUpdate {
    pub title: Option<String>,
    pub done: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TaskFilter {
    id: Option<OpValsInt64>,
    title: Option<OpValsString>,
    done: Option<OpValsBool>,
}


/// Task Backend Model Controller
pub struct TaskBmc;


impl DbBmc for TaskBmc {
    const TABLE: &'static str = "task";
}


impl TaskBmc {
    /// The Context is decoupled from the framework (Axum/Tauri/...). The model layer doesn't need to know how
    /// the request was made, only that it can always expect a Context with all information needed to make sure
    /// the action can be performed.
    pub async fn create(ctx: &Ctx, mm: &ModelManager, task: TaskForCreate) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, task).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    /// Each filter in filters is chained via OR.
    /// Each property inside each filter is chained via AND.
    pub async fn list(
        ctx: &Ctx,
        mm: &ModelManager,
        filters: Option<Vec<TaskFilter>>,
        list_options: Option<ListOptions>,
    ) -> Result<Vec<Task>> {
        base::list::<Self, _, _>(ctx, mm, filters, list_options).await
    }

    pub async fn update(ctx: &Ctx, mm: &ModelManager, id: i64, task: TaskForUpdate) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, task).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}


#[cfg(test)]
mod tests {
    #![allow(unused)]
    use super::*;
    use crate::_dev_utils;
    use crate::model::Error;
    use anyhow::{Ok, Result};
    use serde_json::json;
    use serial_test::serial;

    /// We run these tests in serial because of the database connection
    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        // Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok title";

        // Create task
        let task_c = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBmc::create(&ctx, &mm, task_c).await?;

        // Check if task exists in the database
        let task = TaskBmc::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        // Cleanup: Delete task from database
        TaskBmc::delete(&ctx, &mm, id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()> {
        // Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        // Execute
        let res = TaskBmc::get(&ctx, &mm, fx_id).await;

        // Check
        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100,
                })
            ),
            "EntityNotFound not matching: {:?}",
            res
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_all_ok() -> Result<()> {
        // Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &["test_list_all_ok - task 01", "test_list_all_ok - task 02"];
        _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        // Execute
        let tasks = TaskBmc::list(&ctx, &mm, None, None).await?;

        // Check
        let tasks: Vec<Task> = tasks
            .into_iter()
            .filter(|t| t.title.starts_with("test_list_all_ok - task"))
            .collect();

        assert_eq!(tasks.len(), 2, "number of seeded tasks");

        // Clean
        for task in tasks.iter() {
            TaskBmc::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_by_filter_ok() -> Result<()> {
        // Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &[
            "test_list_by_filter_ok - task 01.a",
            "test_list_by_filter_ok - task 01.b",
            "test_list_by_filter_ok - task 02.a",
            "test_list_by_filter_ok - task 02.b",
            "test_list_by_filter_ok - task 03",
        ];
        _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        // Execute
        let filters: Vec<TaskFilter> = serde_json::from_value(json!([
            {
                "title": {
                    "$endsWith": ".a",
                    "$containsAny": ["01", "02"],
                },
            },
            {
                "title": { "$contains": "03" }
            }
        ]))?;
        let list_options: ListOptions = serde_json::from_value(json!({
            "order_bys": "!id",
        }))?;
        let tasks = TaskBmc::list(&ctx, &mm, Some(filters), Some(list_options)).await?;

        // Check
        //println!("{:#?}", tasks);
        assert_eq!(tasks.len(), 3);
        assert!(tasks[0].title.ends_with("03"));
        assert!(tasks[1].title.ends_with("02.a"));
        assert!(tasks[2].title.ends_with("01.a"));

        // Clean
        let tasks = TaskBmc::list(
            &ctx,
            &mm,
            Some(serde_json::from_value(json!([{
                "title": { "$startsWith": "test_list_by_filter_ok" }
            }]))?),
            None,
        )
        .await?;
        assert_eq!(tasks.len(), 5);

        for task in tasks.iter() {
            TaskBmc::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_update_ok() -> Result<()> {
        // Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_update_ok - task 01";
        let fx_title_new = "test_update_ok - task 01 - new";
        let fx_task = _dev_utils::seed_tasks(&ctx, &mm, &[fx_title])
            .await?
            .remove(0); // pop task 0

        // Execute
        TaskBmc::update(
            &ctx,
            &mm,
            fx_task.id,
            TaskForUpdate {
                title: Some(fx_title_new.to_string()),
                ..Default::default()
            },
        )
        .await;

        // Check
        let task = TaskBmc::get(&ctx, &mm, fx_task.id).await?;
        assert_eq!(task.title, fx_title_new);

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_delete_err_not_found() -> Result<()> {
        // Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        // Execute
        let res = TaskBmc::delete(&ctx, &mm, fx_id).await;

        // Check
        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100,
                })
            ),
            "EntityNotFound not matching: {:?}",
            res
        );

        Ok(())
    }
}
