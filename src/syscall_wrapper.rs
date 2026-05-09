// ZTOS Syscall Wrapper – entry point chiamato da bionic patchato.
// Versione standalone per libztos_compat.a (usa kernel_stubs invece del kernel ZTOS).

use crate::kernel_stubs::{has_capability, CAP_NET_SEND, CAP_NET_RECV,
    NETWORK_ENDPOINT_ID, APP_STORAGE_BASE, CAP_APP_STORAGE,
    CAP_DEBUG, DISPLAY_ENDPOINT_ID, CAP_THREAD_CREATE, CAP_GUI_RENDER,
    vpn_kill_switch_active};
use crate::syscall_table::{lookup, SyscallCat};

pub const EPERM:  i64 = -1;
pub const ENOENT: i64 = -2;
pub const EINTR:  i64 = -4;
pub const EBADF:  i64 = -9;
pub const ENOMEM: i64 = -12;
pub const EACCES: i64 = -13;
pub const EFAULT: i64 = -14;
pub const EINVAL: i64 = -22;
pub const ENOSYS: i64 = -38;

static mut CURRENT_APP_ID:    u32 = 0;
static mut CURRENT_DOMAIN_ID: u32 = 0;

#[no_mangle]
pub extern "C" fn ztos_set_app_context(app_id: u32, domain_id: u32) {
    unsafe { CURRENT_APP_ID = app_id; CURRENT_DOMAIN_ID = domain_id; }
}

#[no_mangle]
pub extern "C" fn ztos_get_current_app_id()    -> u32 { unsafe { CURRENT_APP_ID } }

#[no_mangle]
pub extern "C" fn ztos_get_current_domain_id() -> u32 { unsafe { CURRENT_DOMAIN_ID } }

pub fn current_app_id()    -> u32 { unsafe { CURRENT_APP_ID } }
pub fn current_domain_id() -> u32 { unsafe { CURRENT_DOMAIN_ID } }

#[no_mangle]
pub extern "C" fn ztos_syscall_wrapper(
    nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64,
) -> i64 {
    let app_id    = current_app_id();
    let domain_id = current_domain_id();

    let desc = match lookup(nr) {
        Some(d) => d,
        None    => return ENOSYS,
    };

    // Capability check
    if desc.cap_needed != 0 && !check_cap(app_id, domain_id, desc.cap_needed, nr) {
        return EPERM;
    }

    match desc.category {
        SyscallCat::Memory   => crate::memory_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Io       => crate::io_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Thread   => crate::thread_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Signal   => crate::thread_bridge::handle_signal(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Sched    => crate::sched_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Net      => crate::net_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::ProcInfo => handle_proc_info(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Debug    => EPERM,
        SyscallCat::Noop     => 0,
        SyscallCat::Stub     => handle_stub(nr, a0, a1, a2, a3, a4, a5),
    }
}

fn check_cap(app_id: u32, domain_id: u32, needed: u32, nr: u64) -> bool {
    if needed & (CAP_NET_SEND | CAP_NET_RECV) != 0 {
        if !has_capability(app_id, NETWORK_ENDPOINT_ID, needed & (CAP_NET_SEND | CAP_NET_RECV)) {
            return false;
        }
        if vpn_kill_switch_active(domain_id) { return false; }
    }
    if needed & CAP_APP_STORAGE != 0 {
        let ep = APP_STORAGE_BASE + app_id;
        if !has_capability(app_id, ep, CAP_APP_STORAGE) { return false; }
    }
    if needed & CAP_THREAD_CREATE != 0 {
        if !has_capability(app_id, DISPLAY_ENDPOINT_ID, CAP_THREAD_CREATE) { return false; }
    }
    if needed & CAP_DEBUG != 0 { return false; }
    true
}

fn handle_proc_info(nr: u64, a0: u64, a1: u64, a2: u64, _a3: u64, _a4: u64, _a5: u64) -> i64 {
    use crate::syscall_table::*;
    match nr {
        SYS_GETPID    => current_app_id() as i64,
        SYS_GETTID    => (current_app_id() * 100 + 1) as i64,
        SYS_GETPPID   => 1,
        SYS_GETUID    => 10000,
        SYS_GETEUID   => 10000,
        SYS_GETGID    => 10000,
        SYS_GETEGID   => 10000,
        SYS_GETCPU    => 0,
        SYS_GETRLIMIT => handle_getrlimit_pub(a0, a1),
        SYS_SETRLIMIT => 0,
        SYS_PRCTL     => handle_prctl_pub(a0, a1, a2, 0, 0),
        SYS_UNAME     => handle_uname(a0),
        _             => EINVAL,
    }
}

pub fn handle_getrlimit_pub(resource: u64, rlim_ptr: u64) -> i64 {
    if rlim_ptr == 0 { return EFAULT; }
    unsafe {
        let ptr = rlim_ptr as *mut u64;
        match resource {
            7  => { *ptr = 8*1024*1024; *ptr.add(1) = 8*1024*1024; }
            8  => { *ptr = u64::MAX;    *ptr.add(1) = u64::MAX; }
            _  => { *ptr = u64::MAX;    *ptr.add(1) = u64::MAX; }
        }
    }
    0
}

pub fn handle_prctl_pub(option: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64) -> i64 {
    match option {
        15 | 16 | 36 | 38 | 1 | 22 => 0,
        _  => 0,
    }
}

fn handle_uname(buf_ptr: u64) -> i64 {
    if buf_ptr == 0 { return EFAULT; }
    unsafe {
        let p = buf_ptr as *mut u8;
        copy_str_to(p,        b"Linux");
        copy_str_to(p.add(65), b"ztos-device");
        copy_str_to(p.add(130),b"6.1.0-ztos");
        copy_str_to(p.add(195),b"#1 ZTOS SMP");
        copy_str_to(p.add(260),b"aarch64");
    }
    0
}

fn handle_stub(nr: u64, a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> i64 {
    use crate::syscall_table::SYS_GETRANDOM;
    if nr == SYS_GETRANDOM {
        let buf = a0 as *mut u8;
        let len = a1 as usize;
        if buf.is_null() { return EFAULT; }
        unsafe {
            for i in 0..len.min(256) {
                *buf.add(i) = ((i as u32).wrapping_mul(0x9e37) ^ 0x5a5a) as u8;
            }
        }
        return len as i64;
    }
    0
}

unsafe fn copy_str_to(dst: *mut u8, src: &[u8]) {
    for (i, &b) in src.iter().enumerate() { *dst.add(i) = b; }
    *dst.add(src.len()) = 0;
}
