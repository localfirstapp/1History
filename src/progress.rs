use indicatif::ProgressBar;
use std::sync::{Arc, Mutex};

pub trait ProgressCollector {
    fn inc(&self, delta: u64);
    fn finish(&self);
}

pub struct TUICollector {
    pb: ProgressBar,
}

impl TUICollector {
    pub fn new(len: u64) -> Self {
        Self {
            pb: ProgressBar::new(len),
        }
    }
}

impl ProgressCollector for TUICollector {
    fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }

    fn finish(&self) {
        self.pb.finish();
    }
}

pub struct LogCollector {
    lines: Arc<Mutex<Vec<String>>>,
    label: String,
    total: u64,
    done: Arc<Mutex<u64>>,
}

impl LogCollector {
    pub fn new(label: String, total: u64, lines: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            lines,
            label,
            total,
            done: Arc::new(Mutex::new(0)),
        }
    }
}

impl ProgressCollector for LogCollector {
    fn inc(&self, delta: u64) {
        let mut d = self.done.lock().unwrap();
        *d += delta;
        if (*d).is_multiple_of(500) || *d == self.total {
            self.lines
                .lock()
                .unwrap()
                .push(format!("[{}] {}/{}", self.label, d, self.total));
        }
    }
    fn finish(&self) {
        let d = *self.done.lock().unwrap();
        self.lines
            .lock()
            .unwrap()
            .push(format!("[{}] done ({} records)", self.label, d));
    }
}
