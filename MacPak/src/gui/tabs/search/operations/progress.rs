//! Shared progress state for search operations

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Maximum results for fulltext search
pub const MAX_RESULTS: usize = 50000;

/// Shared progress state for search operations (thread-safe)
#[derive(Default)]
pub struct SharedSearchProgress {
    current: AtomicUsize,
    total: AtomicUsize,
    message: Mutex<String>,
    active: AtomicBool,
}

impl SharedSearchProgress {
    pub fn set(&self, current: usize, total: usize, message: String) {
        self.current.store(current, Ordering::SeqCst);
        self.total.store(total, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = message;
        }
    }

    pub fn get(&self) -> (usize, usize, String) {
        let msg = self.message.lock().map(|m| m.clone()).unwrap_or_default();
        (
            self.current.load(Ordering::SeqCst),
            self.total.load(Ordering::SeqCst),
            msg,
        )
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.current.store(0, Ordering::SeqCst);
        self.total.store(0, Ordering::SeqCst);
        if let Ok(mut msg) = self.message.lock() {
            *msg = String::new();
        }
    }
}

lazy_static::lazy_static! {
    pub static ref SEARCH_PROGRESS: Arc<SharedSearchProgress> = Arc::new(SharedSearchProgress::default());
    /// Track whether we've already attempted to auto-load the cached index
    pub static ref INDEX_AUTO_LOADED: AtomicBool = AtomicBool::new(false);
}
