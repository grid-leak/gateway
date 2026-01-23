use std::sync::{Arc, Mutex};

use tracing::info;

#[derive(Clone)]
pub struct GatewayContext {
    pub counter: Arc<Mutex<u64>>,
}

impl GatewayContext {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn append(&self, value: u64) {
        let mut counter = self.counter.lock().unwrap();
        *counter += value;
        info!("counter: {}", *counter);
    }
}
