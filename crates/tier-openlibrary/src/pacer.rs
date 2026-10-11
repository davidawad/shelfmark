//! At most one request per interval, across every thread sharing the
//! pacer.

use std::sync::Mutex;
use std::time::{Duration, Instant};

type Sleeper = Box<dyn Fn(Duration) + Send + Sync>;

/// Spaces requests at least `interval` apart.
pub struct Pacer {
    interval: Duration,
    last: Mutex<Option<Instant>>,
    sleep: Sleeper,
}

impl Pacer {
    pub fn new(interval: Duration) -> Self {
        Pacer::with_sleeper(interval, Box::new(std::thread::sleep))
    }

    /// A pacer that waits through `sleep` (tests record the waits instead
    /// of sleeping; time then does not advance, so every wait after the
    /// first is a full interval).
    pub fn with_sleeper(interval: Duration, sleep: Sleeper) -> Self {
        Pacer {
            interval,
            last: Mutex::new(None),
            sleep,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Block until a request may go out, and claim that slot. Callers
    /// queue on the lock, so concurrent requests stay spaced too.
    pub fn wait(&self) {
        self.wait_at_least(Duration::ZERO);
    }

    /// As [`Pacer::wait`], but at least `extra` after the last request
    /// (retry backoff).
    pub fn wait_at_least(&self, extra: Duration) {
        let mut last = self.last.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(t) = *last {
            let due = self.interval.max(extra);
            let since = t.elapsed();
            if since < due {
                (self.sleep)(due - since);
            }
        }
        *last = Some(Instant::now());
    }
}
