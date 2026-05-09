// ZTOS Syscall Wrapper – entry point chiamato da bionic patchato.
//
// Quando bionic esegue una syscall, invece di `svc #0` chiama:
//   bl ztos_syscall_wrapper
// con registro x0=nr, x1..x6=argomenti.
//
// Questa funzione:
//   1. Legge il contesto dell'app corrente (app_id, domain_id)
//   2. Controlla le capability nel kernel ZTOS
//   3. Delega all'handler appropriato (memory_bridge, io_bridge, etc.)
//   4. Ritorna il risultato in x0 (come farebbe il kernel Linux)

#![allow(non_snake_case)]

use crate::kernel::capability::has_capability;
use crate::art::syscall_table::{lookup, SyscallCat, SyscallDesc};
use crate::art::memory_bridge;
use crate::art::io_bridge;
use crate::art::thread_bridge;
use crate::art::net_bridge;
use crate::art::sched_bridge;

// ── Errno Linux ───────────────────────────────────────────────────────────
pub const EPERM:   i64 = -1;
pub const ENOENT:  i64 = -2;
pub const EINTR:   i64 = -4;
pub const EBADF:   i64 = -9;
pub const ENOMEM:  i64 = -12;
pub const EACCES:  i64 = -13;
pub const EFAULT:  i64 = -14;
pub const EINVAL:  i64 = -22;
pub const ENOSYS:  i64 = -38;

// ── Contesto app corrente ─────────────────────────────────────────────────
// In un sistema multi-threaded reale, questi sarebbero per-thread (TLS).
// In ZTOS V0.7 usiamo variabili globali protette dalla natura single-threaded.

static mut CURRENT_APP_ID:    u32 = 0;
static mut CURRENT_DOMAIN_ID: u32 = 0;

/// Imposta il contesto dell'app corrente (chiamato da package_manager).
#[no_mangle]
pub extern "C" fn ztos_set_app_context(app_id: u32, domain_id: u32) {
    unsafe {
        CURRENT_APP_ID    = app_id;
        CURRENT_DOMAIN_ID = domain_id;
    }
}

pub fn current_app_id()    -> u32 { unsafe { CURRENT_APP_ID } }
pub fn current_domain_id() -> u32 { unsafe { CURRENT_DOMAIN_ID } }

// ── Entry point principale ─────────────────────────────────────────────────
/// Chiamato da bionic patchato invece di `svc #0`.
/// Firma: (nr, a0, a1, a2, a3, a4, a5) → return_value
///
/// L'assembly bionic riorganizza i registri prima di chiamare questa funzione:
///   mov x0, x8    (syscall number)
///   mov x1, x0_orig  (first arg)
///   ... etc
#[no_mangle]
pub extern "C" fn ztos_syscall_wrapper(
    nr: u64,
    a0: u64, a1: u64, a2: u64,
    a3: u64, a4: u64, a5: u64,
) -> i64 {
    let app_id    = current_app_id();
    let domain_id = current_domain_id();

    // Lookup del descrittore
    let desc = match lookup(nr) {
        Some(d) => d,
        None    => return handle_unknown(nr),
    };

    // Verifica capability (se richiesta)
    if desc.cap_needed != 0 {
        if !check_cap(app_id, domain_id, desc) {
            return log_denied(desc.name, app_id);
        }
    }

    // Dispatch per categoria
    match desc.category {
        SyscallCat::Memory   => memory_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Io       => io_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Thread   => thread_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Signal   => thread_bridge::handle_signal(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Sched    => sched_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Net      => net_bridge::handle(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::ProcInfo => handle_proc_info(nr, a0, a1, a2, a3, a4, a5),
        SyscallCat::Debug    => handle_debug(nr, app_id),
        SyscallCat::Noop     => handle_noop(nr),
        SyscallCat::Stub     => handle_stub(nr, a0, a1, a2, a3, a4, a5),
    }
}

// ── Controllo capability ──────────────────────────────────────────────────

fn check_cap(app_id: u32, domain_id: u32, desc: &SyscallDesc) -> bool {
    use crate::kernel::capability::*;
    let needed = desc.cap_needed;

    // Controllo 1: capability generiche dell'app
    if needed & CAP_APP_STORAGE != 0 {
        let ep = APP_STORAGE_BASE + app_id;
        if !has_capability(app_id, ep, CAP_APP_STORAGE) { return false; }
    }
    if needed & (CAP_NET_SEND | CAP_NET_RECV) != 0 {
        if !has_capability(app_id, NETWORK_ENDPOINT_ID, needed & (CAP_NET_SEND|CAP_NET_RECV)) {
            return false;
        }
        // Controllo 2: kill switch VPN
        if crate::net::vpn::kill_switch_active(domain_id) { return false; }
    }
    if needed & CAP_THREAD_CREATE != 0 {
        if !has_capability(app_id, DISPLAY_ENDPOINT_ID, CAP_THREAD_CREATE) { return false; }
    }
    if needed & CAP_GUI_RENDER != 0 {
        if !has_capability(app_id, DISPLAY_ENDPOINT_ID, CAP_GUI_RENDER) { return false; }
    }
    if needed & CAP_DEBUG != 0 {
        // Debug solo se cap esplicita (mai concessa in produzione)
        if !has_capability(app_id, DISPLAY_ENDPOINT_ID, CAP_DEBUG) { return false; }
    }
    if needed & CAP_SIGNAL != 0 {
        // Segnali verso altri processi: verifica dominio
        // Per segnali interni al processo: sempre permessi
    }
    true
}

// ── Handler specializzati ─────────────────────────────────────────────────

fn handle_proc_info(nr: u64, _a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64) -> i64 {
    use crate::art::syscall_table::*;
    match nr {
        SYS_GETPID    => current_app_id() as i64,
        SYS_GETTID    => (current_app_id() * 100 + 1) as i64, // thread ID simulato
        SYS_GETPPID   => 1i64,       // init (zygote stub)
        SYS_GETUID    => 10000i64,   // UID app Android (u0_a0)
        SYS_GETEUID   => 10000i64,
        SYS_GETGID    => 10000i64,
        SYS_GETEGID   => 10000i64,
        SYS_GETCPU    => 0i64,       // CPU 0
        SYS_GETRLIMIT => handle_getrlimit(_a0, _a1),
        SYS_SETRLIMIT => 0i64,       // accettato (stub)
        SYS_PRCTL     => handle_prctl(_a0, _a1, _a2, _a3, _a4),
        SYS_UNAME     => handle_uname(_a0),
        _             => EINVAL,
    }
}

fn handle_getrlimit(resource: u64, rlim_ptr: u64) -> i64 {
    // Ritorna limiti plausibili per un processo Android
    if rlim_ptr == 0 { return EFAULT; }
    unsafe {
        let ptr = rlim_ptr as *mut u64;
        match resource {
            7  => { *ptr = 8*1024*1024; *ptr.add(1) = 8*1024*1024; }  // RLIMIT_STACK: 8MB
            8  => { *ptr = u64::MAX;    *ptr.add(1) = u64::MAX; }       // RLIMIT_AS: unlimited
            0  => { *ptr = 1024;        *ptr.add(1) = 4096; }           // RLIMIT_CPU
            7  => { *ptr = 65536;       *ptr.add(1) = 65536; }          // RLIMIT_NOFILE
            _  => { *ptr = u64::MAX;    *ptr.add(1) = u64::MAX; }
        }
    }
    0
}

fn handle_prctl(option: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64) -> i64 {
    match option {
        15 => 0,  // PR_SET_NAME: nome thread, ignorato
        16 => 0,  // PR_GET_NAME
        36 => 0,  // PR_SET_SECCOMP: ignorato (nessuna seccomp in ZTOS)
        38 => 0,  // PR_SET_NO_NEW_PRIVS
        1  => 0,  // PR_SET_PDEATHSIG
        22 => 0,  // PR_SET_MM
        _  => 0,  // accetta tutto per compatibilità
    }
}

fn handle_uname(buf_ptr: u64) -> i64 {
    // Riempie struct utsname con dati ZTOS
    if buf_ptr == 0 { return EFAULT; }
    unsafe {
        let ptr = buf_ptr as *mut u8;
        // sysname (65 bytes)
        copy_str(ptr, "Linux");
        // nodename (65 bytes, offset 65)
        copy_str(ptr.add(65), "ztos-device");
        // release (65 bytes, offset 130)
        copy_str(ptr.add(130), "6.1.0-ztos");
        // version (65 bytes, offset 195)
        copy_str(ptr.add(195), "#1 ZTOS SMP");
        // machine (65 bytes, offset 260)
        copy_str(ptr.add(260), "aarch64");
    }
    0
}

fn handle_debug(nr: u64, app_id: u32) -> i64 {
    use crate::kernel::uart::print_str;
    print_str("[ZTOS] DEBUG syscall denied for app ");
    let _ = app_id;
    EPERM
}

fn handle_noop(nr: u64) -> i64 {
    // Syscall che non hanno effetto in ZTOS (sched_yield, membarrier, etc.)
    0
}

fn handle_stub(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    use crate::art::syscall_table::*;
    match nr {
        SYS_GETRANDOM => {
            // Genera byte pseudo-casuali dalla TEE key
            let buf_ptr = a0 as *mut u8;
            let len     = a1 as usize;
            if buf_ptr.is_null() { return EFAULT; }
            let key_handle = crate::kernel::tee::generate_key();
            unsafe {
                for i in 0..len.min(256) {
                    *buf_ptr.add(i) = ((key_handle.wrapping_mul(0x9e37 + i as u32))
                        ^ (i as u32 * 0x1f)) as u8;
                }
            }
            len as i64
        }
        SYS_SECCOMP => 0,  // ignora seccomp, ZTOS ha il proprio enforcement
        _ => ENOSYS,
    }
}

fn handle_unknown(nr: u64) -> i64 {
    use crate::kernel::uart::print_str;
    print_str("[ZTOS] Unknown syscall: ");
    print_nr(nr);
    print_str(" - returning ENOSYS\n");
    ENOSYS
}

fn log_denied(name: &str, app_id: u32) -> i64 {
    use crate::kernel::uart::print_str;
    print_str("[ZTOS] Syscall ");
    print_str(name);
    print_str(" denied for app_id=");
    print_u32(app_id);
    print_str(" (capability check failed)\n");
    EPERM
}

// ── Utility ───────────────────────────────────────────────────────────────

unsafe fn copy_str(dst: *mut u8, s: &str) {
    let b = s.as_bytes();
    for (i, &byte) in b.iter().enumerate() {
        *dst.add(i) = byte;
    }
    *dst.add(b.len()) = 0;
}

fn print_nr(n: u64) {
    use crate::kernel::uart::print_str;
    let mut buf = [0u8; 20]; let mut i = 20usize; let mut v = n;
    if v == 0 { print_str("0"); return; }
    while v > 0 { i -= 1; buf[i] = b'0' + (v%10) as u8; v /= 10; }
    for &b in &buf[i..] { let a=[b]; if let Ok(s)=core::str::from_utf8(&a){print_str(s);} }
}

fn print_u32(n: u32) { print_nr(n as u64); }
