//! Constant-rate CPU polling in a background thread (port of `src/rate_poll.*`).

use crate::cpu::{Cpu, CpuError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct Inner {
    stat_path: String,
    period: Duration,
    io_err: Mutex<Option<String>>,
    fmt_err: Mutex<Option<String>>,
    load: Mutex<f32>,
}

#[derive(Clone)]
pub struct RatePoll {
    inner: Arc<Inner>,
}

impl RatePoll {
    pub fn new(period_ms: u64, stat_path: &str) -> Self {
        RatePoll {
            inner: Arc::new(Inner {
                stat_path: stat_path.to_string(),
                period: Duration::from_millis(period_ms),
                io_err: Mutex::new(None),
                fmt_err: Mutex::new(None),
                load: Mutex::new(0.0),
            }),
        }
    }

    /// CPU polling routine; run in a separate thread.
    pub fn run(&self) {
        let mut cpu = Cpu::new(&self.inner.stat_path);

        loop {
            let point = Instant::now() + self.inner.period;

            match cpu.poll() {
                Ok(load) => {
                    *self.inner.load.lock().unwrap() = load;
                }
                Err(CpuError::Io(m)) => {
                    *self.inner.io_err.lock().unwrap() = Some(m);
                    break;
                }
                Err(CpuError::Fmt(m)) => {
                    *self.inner.fmt_err.lock().unwrap() = Some(m);
                    break;
                }
            }

            let now = Instant::now();
            if point > now {
                thread::sleep(point - now);
            }
        }
    }

    pub fn io_err(&self) -> Option<String> {
        self.inner.io_err.lock().unwrap().clone()
    }

    pub fn fmt_err(&self) -> Option<String> {
        self.inner.fmt_err.lock().unwrap().clone()
    }

    pub fn poll(&self) -> f32 {
        *self.inner.load.lock().unwrap()
    }
}
