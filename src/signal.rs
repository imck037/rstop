use crate::SignalOption;

pub fn send_signal(pid: i32, signal: i32) {
    unsafe {
        libc::kill(pid, signal);
    }
}
pub const SIGNALS: &[SignalOption] = &[
    SignalOption {
        name: "SIGTERM",
        value: libc::SIGTERM,
    },
    SignalOption {
        name: "SIGKILL",
        value: libc::SIGKILL,
    },
    SignalOption {
        name: "SIGSTOP",
        value: libc::SIGSTOP,
    },
    SignalOption {
        name: "SIGCONT",
        value: libc::SIGCONT,
    },
];
