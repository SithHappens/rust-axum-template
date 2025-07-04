//! Simplistic model layer (with mock-store layer)
//!
//! The model manager is going to be for the state, like database pull.
//! The model controller is going to have CRUD and other methods pair entities.
//! It will also have a store, e.g. SQLx.
//!
//! Design
//! - the model layer normalizes the application's data type structures and access
//! - all application code data access must go through the model layer
//! - the [`ModelManager`] holds the internal states/resources needed by ModelControllers to access data,
//!   e.g. db_pool, S3 client, redis client
//! - ModelControllers (e.g. TaskBmc, ProjectBmc) ipmlement CRUD and other data access methods on a given "entity",
//!   e.g. 'Task', 'Project'
//! - In frameworks like Axum, Tauri, [`ModelManager`] are typically used as App State.
//! - ModelManager are designed to be passed as an argument to all ModelController functions
//!
//! BMC = Backend Model Controller


mod base;
mod error;
mod store;
pub mod task;
pub mod user;

use crate::model::store::{Db, new_db_pool};

pub use self::error::{Error, Result};


#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}


impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager { db })
    }

    /// Returns the sqlx db pool reference (only for the model layer)
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}


/*
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::ctx::Ctx;
// region:    --- Ticket Types

// region:    --- Ticket Types
/// Ticket is like a task, sent to the client
#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
    pub id: u64,
    pub cid: u64,  // creator user id
    pub title: String,
}

/// Payload that is sent for the crate API, deserialized from JSON to REST
#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}

// endregion: --- Ticket Types

// region:    --- Model Controller

/// Will be an application state. Clone will not clone the vector, it will clone the Arc.
#[derive(Clone)]
pub struct ModelController {
    tickets_store: Arc<Mutex<Vec<Option<Ticket>>>>, // will later be a database
}

impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets_store: Arc::default(),
        })
    }

    // CRUD Implementation
    pub async fn create_ticket(&self, ctx: Ctx, ticket_fc: TicketForCreate) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().unwrap();

        let id = store.len() as u64;

        let ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };

        store.push(Some(ticket.clone()));

        Ok(ticket)
    }

    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        let store = self.tickets_store.lock().unwrap();

        // clone the options, and because it's filter_map,
        // everything that has been deleted won't be returned
        let tickets = store.iter().filter_map(|t| t.clone()).collect();

        Ok(tickets)
    }

    pub async fn delete_ticket(&self, _ctx: Ctx, id: u64) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().unwrap();

        // Take the ticket out if found, leave None behind in the Option
        let ticket = store.get_mut(id as usize).and_then(|t| t.take());

        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}

// endregion: --- Model Controller
*/
