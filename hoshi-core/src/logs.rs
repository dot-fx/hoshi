use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::fmt::Write;
use serde::{Deserialize, Serialize};
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing::field::{Field, Visit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: String,
    pub target: String,
    pub message: String,
}

pub type LogStore = Arc<RwLock<VecDeque<LogEntry>>>;

pub fn new_log_store() -> LogStore {
    Arc::new(RwLock::new(VecDeque::new()))
}

pub struct MemoryLogLayer {
    pub store: LogStore,
    pub limit: usize,
}

impl<S: Subscriber> Layer<S> for MemoryLogLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut message = String::new();
        let mut visitor = LogVisitor(&mut message);
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: event.metadata().level().to_string(),
            target: event.metadata().target().to_string(),
            message: message.trim().to_string(),
        };

        if let Ok(mut lock) = self.store.write() {
            if lock.len() >= self.limit {
                lock.pop_front();
            }
            lock.push_back(entry);
        }
    }
}

struct LogVisitor<'a>(&'a mut String);

impl<'a> Visit for LogVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.0, "{:?} ", value);
        } else {
            let _ = write!(self.0, "{}={:?} ", field.name(), value);
        }
    }
}