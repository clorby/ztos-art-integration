#![no_std]
#![allow(
    dead_code, unused_variables, unused_imports,
    non_snake_case, unused_assignments,
    static_mut_refs, unreachable_patterns,
)]

// Modulo stubs kernel ZTOS (dipendenze locali alla libreria compat)
mod kernel_stubs;
use kernel_stubs::*;

// Bridge per categoria syscall
pub mod syscall_table;
pub mod syscall_wrapper;
pub mod memory_bridge;
pub mod io_bridge;
pub mod thread_bridge;
pub mod net_bridge;
pub mod sched_bridge;

// Re-export dei simboli C richiesti da bionic/ART
pub use syscall_wrapper::{
    ztos_syscall_wrapper,
    ztos_set_app_context,
    ztos_get_current_app_id,
    ztos_get_current_domain_id,
};

// ── Wrappers C per ogni categoria ────────────────────────────────────────

#[no_mangle]
pub extern "C" fn ztos_mmap(
    addr: u64, length: u64, prot: u32, flags: u32, fd: i32, offset: u64,
) -> u64 {
    memory_bridge::handle(
        syscall_table::SYS_MMAP, addr, length, prot as u64, flags as u64, fd as u64, offset,
    ) as u64
}

#[no_mangle]
pub extern "C" fn ztos_munmap(addr: u64, length: u64) -> i32 {
    memory_bridge::handle(syscall_table::SYS_MUNMAP, addr, length, 0, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_mprotect(addr: u64, length: u64, prot: u32) -> i32 {
    memory_bridge::handle(syscall_table::SYS_MPROTECT, addr, length, prot as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_madvise(addr: u64, length: u64, advice: i32) -> i32 {
    memory_bridge::handle(syscall_table::SYS_MADVISE, addr, length, advice as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_memfd_create(name: *const u8, flags: u32) -> i32 {
    memory_bridge::handle(syscall_table::SYS_MEMFD_CREATE, name as u64, flags as u64, 0, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_ftruncate(fd: i32, length: u64) -> i32 {
    io_bridge::ftruncate(fd, length) as i32
}

#[no_mangle]
pub extern "C" fn ztos_mprotect_exec(addr: u64, length: u64) -> i32 {
    let app_id = ztos_get_current_app_id();
    memory_bridge::handle(syscall_table::SYS_MPROTECT, addr, length, 5, 0, 0, 0) as i32 // PROT_READ|EXEC=5
}

#[no_mangle]
pub extern "C" fn ztos_openat_compat(path: *const u8, flags: u32, mode: u32) -> i32 {
    io_bridge::handle(
        syscall_table::SYS_OPENAT,
        0xFFFF_FF9Cu64, // AT_FDCWD = -100
        path as u64, flags as u64, mode as u64, 0, 0,
    ) as i32
}

#[no_mangle]
pub extern "C" fn ztos_read(fd: i32, buf: *mut u8, len: u64) -> i64 {
    io_bridge::handle(syscall_table::SYS_READ, fd as u64, buf as u64, len, 0, 0, 0)
}

#[no_mangle]
pub extern "C" fn ztos_write(fd: i32, buf: *const u8, len: u64) -> i64 {
    io_bridge::handle(syscall_table::SYS_WRITE, fd as u64, buf as u64, len, 0, 0, 0)
}

#[no_mangle]
pub extern "C" fn ztos_close(fd: i32) -> i32 {
    io_bridge::handle(syscall_table::SYS_CLOSE, fd as u64, 0, 0, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_fstat(fd: i32, stat_buf: *mut u8) -> i32 {
    io_bridge::handle(syscall_table::SYS_FSTAT, fd as u64, stat_buf as u64, 0, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_stat(path: *const u8, stat_buf: *mut u8) -> i32 {
    io_bridge::handle(syscall_table::SYS_NEWFSTATAT, 0xFFFF_FF9Cu64, path as u64, stat_buf as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_lseek(fd: i32, offset: i64, whence: i32) -> i64 {
    io_bridge::handle(syscall_table::SYS_LSEEK, fd as u64, offset as u64, whence as u64, 0, 0, 0)
}

#[no_mangle]
pub extern "C" fn ztos_socket(domain: i32, stype: i32, proto: i32) -> i32 {
    net_bridge::handle(syscall_table::SYS_SOCKET, domain as u64, stype as u64, proto as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_connect(fd: i32, addr: *const u8, len: u32) -> i32 {
    net_bridge::handle(syscall_table::SYS_CONNECT, fd as u64, addr as u64, len as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_send(fd: i32, buf: *const u8, len: u64, flags: i32) -> i64 {
    net_bridge::handle(syscall_table::SYS_SENDTO, fd as u64, buf as u64, len, flags as u64, 0, 0)
}

#[no_mangle]
pub extern "C" fn ztos_recv(fd: i32, buf: *mut u8, len: u64, flags: i32) -> i64 {
    net_bridge::handle(syscall_table::SYS_RECVFROM, fd as u64, buf as u64, len, flags as u64, 0, 0)
}

#[no_mangle]
pub extern "C" fn ztos_bind(fd: i32, addr: *const u8, len: u32) -> i32 {
    net_bridge::handle(syscall_table::SYS_BIND, fd as u64, addr as u64, len as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_accept(fd: i32, addr: *mut u8, addrlen: *mut u32) -> i32 {
    net_bridge::handle(syscall_table::SYS_ACCEPT, fd as u64, addr as u64, addrlen as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_getaddrinfo_stub(
    node: *const u8, service: *const u8,
    hints: *const u8, res: *mut *mut u8,
) -> i32 {
    // EAI_FAIL = 4 (nessuna risoluzione DNS in V0.7 senza resolver)
    4
}

// ── Panic handler ─────────────────────────────────────────────────────────
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfe", options(nomem, nostack)); }
    }
}
