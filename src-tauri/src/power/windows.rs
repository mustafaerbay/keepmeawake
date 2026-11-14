use super::Power;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use windows_sys::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

enum Cmd {
    Acquire(bool, Sender<Result<(), String>>),
    Release(Sender<()>),
    Shutdown,
}

/// SetThreadExecutionState is per-thread, so every call happens on one
/// dedicated thread that lives as long as this value.
pub struct WindowsPower {
    tx: Sender<Cmd>,
    thread: Option<JoinHandle<()>>,
}

fn worker(rx: Receiver<Cmd>) {
    for cmd in rx {
        match cmd {
            Cmd::Acquire(display, reply) => {
                let mut flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;
                if display {
                    flags |= ES_DISPLAY_REQUIRED;
                }
                // Replacing the flags in one call: no gap.
                let prev = unsafe { SetThreadExecutionState(flags) };
                let _ = reply.send(if prev == 0 {
                    Err("SetThreadExecutionState failed".into())
                } else {
                    Ok(())
                });
            }
            Cmd::Release(done) => {
                unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
                let _ = done.send(());
            }
            Cmd::Shutdown => {
                unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
                break;
            }
        }
    }
}

impl WindowsPower {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let thread = std::thread::Builder::new()
            .name("keepmeawake-power".into())
            .spawn(move || worker(rx))
            .expect("spawn power thread");
        WindowsPower { tx, thread: Some(thread) }
    }
}

impl Power for WindowsPower {
    fn acquire(&mut self, display: bool) -> Result<(), String> {
        let (reply, rx) = channel();
        self.tx.send(Cmd::Acquire(display, reply)).map_err(|e| e.to_string())?;
        rx.recv().map_err(|e| e.to_string())?
    }

    fn release(&mut self) {
        let (done, rx) = channel();
        if self.tx.send(Cmd::Release(done)).is_ok() {
            let _ = rx.recv();
        }
    }
}

impl Drop for WindowsPower {
    fn drop(&mut self) {
        let _ = self.tx.send(Cmd::Shutdown);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
