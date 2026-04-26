use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::sync::atomic::{AtomicBool, Ordering};

/// Set when SIGINT is received in the shell process. Polled by the main loop.
pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn setup_signal_handlers() {
    let action = SigAction::new(
        SigHandler::Handler(handle_sigint),
        SaFlags::empty(),
        SigSet::empty(),
    );
    // SAFETY: installing a process-wide handler at startup before any threads spawn.
    unsafe {
        signal::sigaction(Signal::SIGINT, &action).expect("failed to install SIGINT handler");
    }
}

/// Returns true exactly once if SIGINT was received since the last check.
pub fn take_interrupt() -> bool {
    INTERRUPTED.swap(false, Ordering::SeqCst)
}

extern "C" fn handle_sigint(_sig: i32) {
    // Async-signal-safe: only an atomic store and a write(2) call.
    INTERRUPTED.store(true, Ordering::SeqCst);
    const MSG: &[u8] = b"\n";
    // SAFETY: write(2) is async-signal-safe.
    unsafe {
        let _ = libc::write(libc::STDERR_FILENO, MSG.as_ptr() as *const _, MSG.len());
    }
}
