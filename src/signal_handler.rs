use nix::sys::signal;
use nix::sys::signal::{SigAction, SigHandler, SaFlags, SigSet, Signal};

pub fn setup_signal_handlers() {
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigint),
        SaFlags::empty(),
        SigSet::empty(),
    );
    unsafe {
        signal::sigaction(Signal::SIGINT, &sig_action).unwrap();
    }
}

extern "C" fn handle_sigint(_signal: i32) {
    // Handle Ctrl+C (SIGINT)
    println!("\nReceived Ctrl+C. Type 'exit' to quit.");
}
