use crate::stuff::config::config;
use crate::Result;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub port: u16,
    pub counter: Arc<AtomicU64>,
    pub work_dir: String,
}

impl AppState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            port: config().WORKER_PORT,
            counter: Arc::new(AtomicU64::new(config().START_COUNTER)),
            work_dir: config().WORK_DIR.clone(),
        })
    }
}