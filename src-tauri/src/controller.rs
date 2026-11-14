use crate::power::Power;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub awake: bool,
    pub display_may_sleep: bool,
    pub click_through: bool,
}

/// Single source of truth for whether the machine is kept awake.
/// The power assertion is always derived from `state`.
pub struct Controller<P: Power> {
    power: P,
    state: State,
}

impl<P: Power> Controller<P> {
    /// Starts awake (the reason the app was opened).
    pub fn new(power: P, display_may_sleep: bool, click_through: bool) -> Self {
        let mut c = Controller {
            power,
            state: State { awake: false, display_may_sleep, click_through },
        };
        c.set_awake(true);
        c
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_awake(&mut self, awake: bool) -> State {
        if awake == self.state.awake {
            return self.state;
        }
        if awake {
            // Never report "lit" when the OS refused the assertion.
            self.state.awake = self.power.acquire(!self.state.display_may_sleep).is_ok();
        } else {
            self.power.release();
            self.state.awake = false;
        }
        self.state
    }

    pub fn toggle(&mut self) -> State {
        self.set_awake(!self.state.awake)
    }

    pub fn set_display_may_sleep(&mut self, v: bool) -> State {
        if v == self.state.display_may_sleep {
            return self.state;
        }
        self.state.display_may_sleep = v;
        // Power::acquire swaps the held assertion with no gap.
        if self.state.awake && self.power.acquire(!v).is_err() {
            self.power.release();
            self.state.awake = false;
        }
        self.state
    }

    pub fn set_click_through(&mut self, v: bool) -> State {
        self.state.click_through = v;
        self.state
    }
}

impl<P: Power> Drop for Controller<P> {
    fn drop(&mut self) {
        if self.state.awake {
            self.power.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct FakePower {
        log: Arc<Mutex<Vec<String>>>,
        fail: Arc<Mutex<bool>>,
    }

    impl Power for FakePower {
        fn acquire(&mut self, display: bool) -> Result<(), String> {
            if *self.fail.lock().unwrap() {
                return Err("nope".into());
            }
            self.log.lock().unwrap().push(format!("acquire({display})"));
            Ok(())
        }
        fn release(&mut self) {
            self.log.lock().unwrap().push("release".into());
        }
    }

    impl FakePower {
        fn log(&self) -> Vec<String> {
            self.log.lock().unwrap().clone()
        }
    }

    #[test]
    fn new_starts_awake_and_acquires() {
        let p = FakePower::default();
        let c = Controller::new(p.clone(), false, false);
        assert_eq!(p.log(), vec!["acquire(true)"]);
        assert!(c.state().awake);
    }

    #[test]
    fn toggle_off_releases() {
        let p = FakePower::default();
        let mut c = Controller::new(p.clone(), false, false);
        let s = c.toggle();
        assert!(!s.awake);
        assert_eq!(p.log().last().unwrap(), "release");
    }

    #[test]
    fn set_awake_same_value_is_noop() {
        let p = FakePower::default();
        let mut c = Controller::new(p.clone(), false, false);
        c.set_awake(true);
        assert_eq!(p.log(), vec!["acquire(true)"]);
    }

    #[test]
    fn rapid_toggle_balances_power_calls() {
        let p = FakePower::default();
        let mut c = Controller::new(p.clone(), false, false);
        for _ in 0..101 {
            c.toggle();
        }
        assert!(!c.state().awake);
        let log = p.log();
        let acq = log.iter().filter(|l| l.starts_with("acquire")).count();
        let rel = log.iter().filter(|l| *l == "release").count();
        assert_eq!(acq, rel);
    }

    #[test]
    fn display_flag_change_reacquires_before_release() {
        let p = FakePower::default();
        let mut c = Controller::new(p.clone(), false, false);
        let before = p.log().len();
        let s = c.set_display_may_sleep(true);
        assert!(s.display_may_sleep && s.awake);
        assert_eq!(p.log()[before..].to_vec(), vec!["acquire(false)"]);

        c.set_awake(false);
        let before = p.log().len();
        c.set_display_may_sleep(false);
        c.set_display_may_sleep(true);
        assert_eq!(p.log().len(), before);
        c.set_awake(true);
        assert_eq!(p.log().last().unwrap(), "acquire(false)");
    }

    #[test]
    fn drop_releases_power() {
        let p = FakePower::default();
        drop(Controller::new(p.clone(), false, false));
        assert_eq!(p.log().last().unwrap(), "release");

        let p = FakePower::default();
        let mut c = Controller::new(p.clone(), false, false);
        c.set_awake(false);
        let n = p.log().len();
        drop(c);
        assert_eq!(p.log().len(), n);
    }

    #[test]
    fn acquire_failure_reports_asleep() {
        let p = FakePower::default();
        let mut c = Controller::new(p.clone(), false, false);
        c.set_awake(false);
        *p.fail.lock().unwrap() = true;
        let s = c.set_awake(true);
        assert!(!s.awake);
    }

    #[test]
    fn click_through_is_tracked() {
        let p = FakePower::default();
        let mut c = Controller::new(p.clone(), false, true);
        assert!(c.state().click_through);
        assert!(!c.set_click_through(false).click_through);
    }
}
