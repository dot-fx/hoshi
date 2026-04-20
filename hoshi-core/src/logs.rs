use std::collections::VecDeque;
use std::sync::{Arc, OnceLock, RwLock};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
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
static LOG_FILE: OnceLock<Arc<RwLock<File>>> = OnceLock::new();

pub fn new_log_store() -> LogStore {
    Arc::new(RwLock::new(VecDeque::new()))
}

pub fn init_log_file(logs_dir: &PathBuf) {
    if !logs_dir.exists() {
        let _ = std::fs::create_dir_all(logs_dir);
    }

    if let Some(f) = new_log_file(logs_dir) {
        let _ = LOG_FILE.set(f);
    }
}

pub fn new_log_file(logs_dir: &PathBuf) -> Option<Arc<RwLock<File>>> {
    let filename = chrono::Utc::now()
        .format("%Y-%m-%dT%H-%M-%S")
        .to_string()
        + ".log";

    let path = logs_dir.join(filename);

    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => Some(Arc::new(RwLock::new(f))),
        Err(e) => {
            eprintln!("[logs] Failed to open log file {:?}: {}", path, e);
            None
        }
    }
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
        let message = message.trim().to_string();

        let now = chrono::Utc::now();

        // 1. Write to Memory Store (for Live View)
        let entry = LogEntry {
            timestamp: now.timestamp_millis(),
            level: event.metadata().level().to_string(),
            target: event.metadata().target().to_string(),
            message: message.clone(),
        };

        if let Ok(mut lock) = self.store.write() {
            if lock.len() >= self.limit {
                lock.pop_front();
            }
            lock.push_back(entry);
        }

        if let Some(file) = LOG_FILE.get() {
            if let Ok(mut f) = file.write() {
                let line = format!(
                    "[{}] {} {} - {}\n",
                    now.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                    event.metadata().level(),
                    event.metadata().target(),
                    message,
                );
                let _ = f.write_all(line.as_bytes());
                let _ = f.flush();
            }
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