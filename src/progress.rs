use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

/// Simple progress message
#[derive(Clone, Debug)]
pub struct ProgressMessage {
    pub message: String,
}

/// Global progress tracker - stores channels for each session
pub type ProgressStore = Arc<RwLock<HashMap<String, mpsc::Sender<ProgressMessage>>>>;

/// Create a new progress store
pub fn create_progress_store() -> ProgressStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Send a progress update for a session
pub async fn send_progress(store: &ProgressStore, session_id: &str, message: String) {
    let store_read = store.read().await;
    if let Some(sender) = store_read.get(session_id) {
        // Try to send, but don't block if channel is full or closed
        let _ = sender.try_send(ProgressMessage { message });
    }
}

/// Register a new session and get the receiver
pub async fn register_session(
    store: &ProgressStore,
    session_id: String,
) -> mpsc::Receiver<ProgressMessage> {
    let (tx, rx) = mpsc::channel(100); // Buffer up to 100 messages
    let mut store_write = store.write().await;
    store_write.insert(session_id, tx);
    rx
}

/// Unregister a session (cleanup)
pub async fn unregister_session(store: &ProgressStore, session_id: &str) {
    let mut store_write = store.write().await;
    store_write.remove(session_id);
}
