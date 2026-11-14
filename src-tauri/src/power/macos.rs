use super::Power;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};

const ASSERTION_NAME: &str = "keepmeawake: Don is smoking";
const LEVEL_ON: u32 = 255; // kIOPMAssertionLevelOn

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOPMAssertionCreateWithName(
        assertion_type: CFStringRef,
        level: u32,
        name: CFStringRef,
        id: *mut u32,
    ) -> i32;
    fn IOPMAssertionRelease(id: u32) -> i32;
}

pub struct MacPower {
    held: Vec<u32>,
}

impl MacPower {
    pub fn new() -> Self {
        MacPower { held: Vec::new() }
    }

    fn create(kind: &str) -> Result<u32, String> {
        let kind = CFString::new(kind);
        let name = CFString::new(ASSERTION_NAME);
        let mut id = 0u32;
        let rc = unsafe {
            IOPMAssertionCreateWithName(
                kind.as_concrete_TypeRef(),
                LEVEL_ON,
                name.as_concrete_TypeRef(),
                &mut id,
            )
        };
        if rc == 0 { Ok(id) } else { Err(format!("IOPMAssertionCreateWithName failed: {rc:#x}")) }
    }

    fn release_ids(ids: Vec<u32>) {
        for id in ids {
            unsafe { IOPMAssertionRelease(id) };
        }
    }
}

impl Power for MacPower {
    fn acquire(&mut self, display: bool) -> Result<(), String> {
        let mut new = vec![Self::create("PreventUserIdleSystemSleep")?];
        if display {
            match Self::create("PreventUserIdleDisplaySleep") {
                Ok(id) => new.push(id),
                Err(e) => {
                    Self::release_ids(new);
                    return Err(e);
                }
            }
        }
        // New ones exist before the old ones go: no gap.
        let old = std::mem::replace(&mut self.held, new);
        Self::release_ids(old);
        Ok(())
    }

    fn release(&mut self) {
        Self::release_ids(std::mem::take(&mut self.held));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn our_assertions() -> Vec<String> {
        let out = Command::new("pmset").args(["-g", "assertions"]).output().unwrap();
        let pid = std::process::id().to_string();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.contains("keepmeawake") && l.contains(&format!("pid {pid}(")))
            .map(|l| l.to_string())
            .collect()
    }

    #[test]
    fn acquire_and_release_show_in_pmset() {
        let mut p = MacPower::new();
        p.acquire(true).unwrap();
        let a = our_assertions();
        assert!(a.iter().any(|l| l.contains("PreventUserIdleSystemSleep")), "{a:?}");
        assert!(a.iter().any(|l| l.contains("PreventUserIdleDisplaySleep")), "{a:?}");

        p.acquire(false).unwrap();
        let a = our_assertions();
        assert!(a.iter().any(|l| l.contains("PreventUserIdleSystemSleep")), "{a:?}");
        assert!(!a.iter().any(|l| l.contains("PreventUserIdleDisplaySleep")), "{a:?}");

        p.release();
        assert!(our_assertions().is_empty());
    }
}
