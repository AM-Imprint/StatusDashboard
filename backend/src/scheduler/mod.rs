use sqlx::SqlitePool;
use tokio::sync::broadcast;
use crate::{state::SchedulerHandles, ws::messages::WsMessage};

pub mod worker;

pub async fn start_all(
    db: &SqlitePool,
    tx: broadcast::Sender<WsMessage>,
    handles: &SchedulerHandles,
) {
    let services = sqlx::query!(r#"SELECT id as "id!" FROM services WHERE enabled = 1"#)
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut map = handles.lock().await;
    for svc in services {
        let handle = tokio::spawn(worker::run_service_loop(
            svc.id.clone(),
            db.clone(),
            tx.clone(),
        ));
        map.insert(svc.id, handle);
    }
}

pub async fn spawn_service(
    service_id: String,
    db: &SqlitePool,
    tx: broadcast::Sender<WsMessage>,
    handles: &SchedulerHandles,
) {
    let mut map = handles.lock().await;
    if let Some(old) = map.remove(&service_id) {
        old.abort();
    }
    let handle = tokio::spawn(worker::run_service_loop(
        service_id.clone(),
        db.clone(),
        tx,
    ));
    map.insert(service_id, handle);
}

pub async fn abort_service(service_id: &str, handles: &SchedulerHandles) {
    let mut map = handles.lock().await;
    if let Some(handle) = map.remove(service_id) {
        handle.abort();
    }
}
