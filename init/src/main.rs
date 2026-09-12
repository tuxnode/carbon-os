// CarbonOS PID 1: mount the virtual filesystems and supervise an interactive shell.
//
// It stays alive as PID 1 for the whole lifetime of the system: the shell runs as a
// child, so when the shell exits it is simply restarted instead of panicking the kernel.

use std::ffi::CString;
use std::fs::OpenOptions;
use std::os::raw::{c_char, c_int, c_ulong, c_void};
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

extern "C" {
    fn mount(
        source: *const c_char,
        target: *const c_char,
        fstype: *const c_char,
        flags: c_ulong,
        data: *const c_void,
    ) -> c_int;
    fn setsid() -> c_int;
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    fn sethostname(name: *const c_char, len: usize) -> c_int;
}

const TIOCSCTTY: c_ulong = 0x540E;
const EBUSY: i32 = 16;

fn mount_fs(source: &str, target: &str, fstype: &str) {
    let (Ok(src), Ok(tgt), Ok(fs)) = (
        CString::new(source),
        CString::new(target),
        CString::new(fstype),
    ) else {
        return;
    };

    let rc = unsafe { mount(src.as_ptr(), tgt.as_ptr(), fs.as_ptr(), 0, std::ptr::null()) };
    if rc == 0 {
        println!("[init] mounted {fstype} on {target}");
        return;
    }

    // EBUSY means the kernel already mounted it (e.g. devtmpfs on /dev); that is fine.
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() != Some(EBUSY) {
        eprintln!("[init] mount {fstype} on {target} failed: {err}");
    }
}

fn ensure_dir(path: &str) {
    if let Err(e) = std::fs::create_dir_all(path) {
        eprintln!("[init] mkdir {path} failed: {e}");
    }
}

fn set_hostname(name: &str) {
    if let Ok(c) = CString::new(name) {
        unsafe {
            sethostname(c.as_ptr(), name.len());
        }
    }
}

fn banner() {
    println!("========================================================");
    println!("      Welcome to CarbonOS (RISC-V 64 / Musl / Rust)     ");
    println!("        Terminal-First Immutable Linux System           ");
    println!("========================================================");
    println!();
}

fn spawn_shell() -> std::io::Result<std::process::Child> {
    let console = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/console")?;
    let stdout = console.try_clone()?;
    let stderr = console.try_clone()?;

    let mut cmd = Command::new("/bin/sh");
    cmd.stdin(console);
    cmd.stdout(stdout);
    cmd.stderr(stderr);

    // Run the shell in its own session with /dev/console as the controlling terminal.
    unsafe {
        cmd.pre_exec(|| {
            if setsid() == -1 {
                // Already a session leader; not fatal.
            }
            ioctl(0, TIOCSCTTY, 0);
            Ok(())
        });
    }

    cmd.spawn()
}

fn main() {
    for dir in ["/proc", "/sys", "/dev", "/tmp", "/run"] {
        ensure_dir(dir);
    }

    mount_fs("proc", "/proc", "proc");
    mount_fs("sysfs", "/sys", "sysfs");
    mount_fs("devtmpfs", "/dev", "devtmpfs");
    // /dev/pts must be created after devtmpfs is mounted over /dev.
    ensure_dir("/dev/pts");
    mount_fs("devpts", "/dev/pts", "devpts");
    mount_fs("tmpfs", "/tmp", "tmpfs");
    mount_fs("tmpfs", "/run", "tmpfs");

    set_hostname("carbonos");
    banner();

    // PID 1 must never exit, so keep restarting the shell forever.
    loop {
        match spawn_shell() {
            Ok(mut child) => {
                let _ = child.wait();
                println!("[init] shell exited; restarting...");
            }
            Err(e) => {
                eprintln!("[init] failed to spawn /bin/sh: {e}");
            }
        }
        sleep(Duration::from_secs(1));
    }
}
