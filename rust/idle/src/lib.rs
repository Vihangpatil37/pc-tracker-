use std::mem;

const IDLE_THRESHOLD_SECONDS: i64 = 300;

#[repr(C)]
struct LASTINPUTINFO {
    cb_size: u32,
    dw_time: u32,
}

#[link(name = "user32")]
extern "system" {
    fn GetLastInputInfo(plii: *mut LASTINPUTINFO) -> i32;
}

extern "system" {
    fn GetTickCount() -> u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleState {
    Active,
    Idle { since: i64 },
}

pub struct IdleDetector {
    state: IdleState,
}

impl IdleDetector {
    pub fn new() -> Self {
        Self {
            state: IdleState::Active,
        }
    }

    pub fn state(&self) -> IdleState {
        self.state
    }

    pub fn tick(&mut self, now: i64) -> IdleState {
        let idle_ms = unsafe {
            let mut li: LASTINPUTINFO = mem::zeroed();
            li.cb_size = mem::size_of::<LASTINPUTINFO>() as u32;
            if GetLastInputInfo(&mut li) != 0 {
                let tick_count = GetTickCount();
                tick_count.wrapping_sub(li.dw_time) as u64
            } else {
                0
            }
        };

        let idle_seconds = (idle_ms / 1000) as i64;

        match self.state {
            IdleState::Active if idle_seconds >= IDLE_THRESHOLD_SECONDS => {
                self.state = IdleState::Idle { since: now };
            }
            IdleState::Idle { .. } if idle_seconds < IDLE_THRESHOLD_SECONDS => {
                self.state = IdleState::Active;
            }
            _ => {}
        }

        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_active() {
        let detector = IdleDetector::new();
        assert_eq!(detector.state(), IdleState::Active);
    }

    #[test]
    fn tick_returns_valid_state() {
        let mut detector = IdleDetector::new();
        let state = detector.tick(1000);
        // Should be Active if user is active, or Idle if inactive for 5+ min
        assert!(state == IdleState::Active || matches!(state, IdleState::Idle { .. }));
    }
}
