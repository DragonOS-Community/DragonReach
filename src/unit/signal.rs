use nix::sys::signal::{self, Signal, SigHandler};
use std::sync::atomic::{AtomicBool, Ordering};

pub static SIGCHILD_SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigchld(_: libc::c_int) {
    SIGCHILD_SIGNAL_RECEIVED.store(true, Ordering::SeqCst);
}


pub fn init_signal_handler() {
    unsafe {
        signal::signal(Signal::SIGCHLD, SigHandler::Handler(handle_sigchld))
            .expect("Error setting SIGUSR1 handler");
    }
}
