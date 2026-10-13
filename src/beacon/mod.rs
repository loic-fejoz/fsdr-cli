use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Beacon Tracker monitoring carrier frequency offset (e.g. QO-100 BPSK beacon)
/// and maintaining `delta_f_beacon` with holdover and TX safety interlock.
#[derive(Clone)]
pub struct BeaconTracker {
    enabled: bool,
    max_drift_hz: i64,
    holdover_on_loss: bool,
    delta_f: Arc<AtomicI64>,
    locked: Arc<AtomicBool>,
    last_lock_time: Arc<std::sync::RwLock<Instant>>,
}

impl BeaconTracker {
    pub fn new(enabled: bool, max_drift_hz: i64, holdover_on_loss: bool) -> Self {
        Self {
            enabled,
            max_drift_hz,
            holdover_on_loss,
            delta_f: Arc::new(AtomicI64::new(0)),
            locked: Arc::new(AtomicBool::new(true)),
            last_lock_time: Arc::new(std::sync::RwLock::new(Instant::now())),
        }
    }

    /// Returns the current estimated beacon frequency offset in Hz
    pub fn delta_f(&self) -> i64 {
        self.delta_f.load(Ordering::Relaxed)
    }

    /// Returns whether the beacon is currently locked
    pub fn is_locked(&self) -> bool {
        if !self.enabled {
            return true;
        }
        self.locked.load(Ordering::Relaxed)
    }

    /// TX Interlock: check if transmission is authorized based on beacon status
    pub fn is_tx_allowed(&self) -> bool {
        if !self.enabled {
            return true;
        }
        if self.is_locked() {
            true
        } else if self.holdover_on_loss {
            // Check holdover timeout (e.g. 10 seconds holdover)
            if let Ok(last) = self.last_lock_time.read() {
                last.elapsed() < Duration::from_secs(10)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Update tracked frequency offset
    pub fn update_offset(&self, offset_hz: i64, is_signal_present: bool) {
        if !self.enabled {
            return;
        }
        if is_signal_present && offset_hz.abs() <= self.max_drift_hz {
            self.delta_f.store(offset_hz, Ordering::Relaxed);
            self.locked.store(true, Ordering::Relaxed);
            if let Ok(mut last) = self.last_lock_time.write() {
                *last = Instant::now();
            }
        } else {
            self.locked.store(false, Ordering::Relaxed);
        }
    }
}
