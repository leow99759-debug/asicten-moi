//! Timers and reminders (SPEC §10.3): jobs fire on a background thread into a channel.

use std::collections::BTreeMap;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Job {
    /// Run a command by id (`Timer.then_command`).
    RunCommand(String),
    /// Say + toast this text.
    Remind(String),
}

enum Ctl {
    Add(u64, Instant, Job),
    Cancel(u64),
    CancelAll,
}

/// Handle; dropping it stops the timer thread.
pub struct Scheduler {
    tx: Sender<Ctl>,
    next_id: u64,
}

impl Scheduler {
    /// Fired jobs arrive on the returned receiver.
    pub fn start() -> (Self, Receiver<Job>) {
        let (tx, rx) = mpsc::channel::<Ctl>();
        let (fired_tx, fired_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("jarvis-timers".into())
            .spawn(move || run(rx, fired_tx))
            .map_err(|e| tracing::error!(%e, "timer thread"))
            .ok();
        (Self { tx, next_id: 1 }, fired_rx)
    }

    pub fn add(&mut self, after: Duration, job: Job) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let _ = self.tx.send(Ctl::Add(id, Instant::now() + after, job));
        id
    }

    pub fn cancel(&self, id: u64) {
        let _ = self.tx.send(Ctl::Cancel(id));
    }

    pub fn cancel_all(&self) {
        let _ = self.tx.send(Ctl::CancelAll);
    }
}

fn run(rx: Receiver<Ctl>, fired: Sender<Job>) {
    let mut jobs: BTreeMap<u64, (Instant, Job)> = BTreeMap::new();
    loop {
        let now = Instant::now();
        let due: Vec<u64> = jobs
            .iter()
            .filter(|(_, (t, _))| *t <= now)
            .map(|(id, _)| *id)
            .collect();
        for id in due {
            if let Some((_, job)) = jobs.remove(&id) {
                if fired.send(job).is_err() {
                    return;
                }
            }
        }
        let wait = jobs
            .values()
            .map(|(t, _)| t.saturating_duration_since(now))
            .min()
            .unwrap_or(Duration::from_secs(3600));
        match rx.recv_timeout(wait) {
            Ok(Ctl::Add(id, at, job)) => {
                jobs.insert(id, (at, job));
            }
            Ok(Ctl::Cancel(id)) => {
                jobs.remove(&id);
            }
            Ok(Ctl::CancelAll) => jobs.clear(),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_in_order_and_cancels() {
        let (mut s, rx) = Scheduler::start();
        s.add(Duration::from_millis(120), Job::Remind("второй".into()));
        let c = s.add(Duration::from_millis(60), Job::RunCommand("отменён".into()));
        s.add(Duration::from_millis(30), Job::Remind("первый".into()));
        s.cancel(c);
        let t = Duration::from_secs(2);
        assert_eq!(rx.recv_timeout(t), Ok(Job::Remind("первый".into())));
        assert_eq!(rx.recv_timeout(t), Ok(Job::Remind("второй".into())));
        assert!(rx.recv_timeout(Duration::from_millis(150)).is_err());
    }
}
