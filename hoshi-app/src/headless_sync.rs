// headless_sync.rs
use std::sync::{Arc, Condvar, Mutex};
use std::collections::HashMap;

pub struct HeadlessSlot {
    pub inner: Mutex<Option<String>>,
    pub cvar:  Condvar,
}

impl HeadlessSlot {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(None),
            cvar:  Condvar::new(),
        })
    }

    pub fn resolve(&self, payload: String) {
        let mut guard = self.inner.lock().unwrap();
        *guard = Some(payload);
        self.cvar.notify_one();
    }

    pub fn wait_timeout(&self, duration: std::time::Duration) -> Option<String> {
        let guard = self.inner.lock().unwrap();
        let (mut guard, timed_out) = self.cvar
            .wait_timeout_while(guard, duration, |v| v.is_none())
            .unwrap();
        if timed_out.timed_out() { None } else { guard.take() }
    }
}

static REGISTRY: std::sync::OnceLock<Mutex<HashMap<String, Arc<HeadlessSlot>>>> =
    std::sync::OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, Arc<HeadlessSlot>>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register_slot(label: String, slot: Arc<HeadlessSlot>) {
    registry().lock().unwrap().insert(label, slot);
}

pub fn unregister_slot(label: &str) {
    registry().lock().unwrap().remove(label);
}

pub fn resolve_slot(label: &str, payload: String) -> bool {
    if let Some(slot) = registry().lock().unwrap().get(label) {
        slot.resolve(payload);
        true
    } else {
        false
    }
}