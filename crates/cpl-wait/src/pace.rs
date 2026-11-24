//! The one rate limiter: a judge's server sees at most one request per
//! interval, as from a person clicking through its pages. Waiting out the
//! interval is this module's one sleep.

use std::time::{Duration, Instant};

pub struct Pace {
    every: Duration,
    last: Option<Instant>,
}

impl Pace {
    pub const fn new(every: Duration) -> Pace {
        Pace { every, last: None }
    }

    /// Returns at once the first time, then once `every` has passed since
    /// the previous return.
    pub fn wait(&mut self) {
        if let Some(last) = self.last {
            std::thread::sleep(self.every.saturating_sub(last.elapsed()));
        }
        self.last = Some(Instant::now());
    }
}

#[cfg(test)]
#[test]
fn the_second_call_waits_out_the_interval() {
    let mut pace = Pace::new(Duration::from_millis(30));
    let start = Instant::now();
    pace.wait();
    assert!(start.elapsed() < Duration::from_millis(30));
    pace.wait();
    assert!(start.elapsed() >= Duration::from_millis(30));
}
