use std::{collections::HashMap, sync::Arc};
use tokio::{sync::{broadcast, Mutex}, task::JoinHandle};
use sqlx::SqlitePool;
use crate::ws::messages::WsMessage;

pub type SchedulerHandles = Arc<Mutex<HashMap<String, JoinHandle<()>>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub tx: broadcast::Sender<WsMessage>,
    pub scheduler_handles: SchedulerHandles,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            db,
            tx,
            scheduler_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
