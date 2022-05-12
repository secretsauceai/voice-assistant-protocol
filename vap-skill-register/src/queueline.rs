use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::Notify;

/// This is a special kind of barrier. It is designed so that one kind of task waits
/// for another kind of task to happen. For example, clients waiting for a server
/// in the same process.
/// 
/// Waiting tasks use `wait` while the task that signals everything is ready will
/// call `open`, once this is is done all the waiting tasks will be called, also
/// any further call to `wait` will return immediately.
#[derive(Clone)]
pub struct QueueLine {
    inner: Arc<QueueLineImpl>
}

struct QueueLineImpl {
    closed: AtomicBool,
    notifier: Notify,
}



impl QueueLine {
    pub fn new() -> Self {
        Self{ inner:
            Arc::new(QueueLineImpl {
                closed: AtomicBool::new(true),
                notifier: Notify::new(),
            })
        }
    }
}

impl Default for QueueLine {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueLine {
    /// Wait for the queue to be open.
    pub async fn wait(&self) {
        if self.inner.closed.load(std::sync::atomic::Ordering::SeqCst) {
            self.inner.notifier.notified().await;
        }
    }

    /// Signal that the queue is open.
    pub fn open(&self) {
        self.inner.closed.store(false, std::sync::atomic::Ordering::SeqCst);
        self.inner.notifier.notify_waiters();
    }
}

