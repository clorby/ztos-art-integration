// Sched bridge: clock_gettime, nanosleep, sched_*, getrlimit, getrandom.

use crate::art::syscall_wrapper::{EINVAL, EFAULT};
use crate::art::syscall_table::*;

// Orologio ZTOS simulato (incrementa ad ogni chiamata)
static mut SIM_NSEC: u64 = 1_000_000_000; // parte da 1 secondo

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    match nr {
        SYS_CLOCK_GETTIME   => sys_clock_gettime(a0 as i32, a1),
        SYS_CLOCK_NANOSLEEP => sys_clock_nanosleep(a0 as i32, a1 as i32, a2, a3),
        SYS_NANOSLEEP       => sys_nanosleep(a0, a1),
        SYS_GETTIMEOFDAY    => sys_gettimeofday(a0, a1),
        SYS_SCHED_YIELD     => 0,
        SYS_SCHED_SETSCHEDULER => 0,
        SYS_SCHED_GETSCHEDULER => 0,
        SYS_SCHED_SETAFFINITY  => 0,
        SYS_SCHED_GETAFFINITY  => sys_sched_getaffinity(a0, a1 as usize, a2),
        SYS_GETRLIMIT          => crate::art::syscall_wrapper::handle_getrlimit_pub(a0, a1),
        SYS_SETRLIMIT          => 0,
        _                      => EINVAL,
    }
}

// CLOCK_REALTIME=0, CLOCK_MONOTONIC=1, CLOCK_PROCESS_CPUTIME_ID=2, CLOCK_THREAD_CPUTIME_ID=3
fn sys_clock_gettime(clockid: i32, ts_ptr: u64) -> i64 {
    if ts_ptr == 0 { return EFAULT; }
    let ns = unsafe {
        SIM_NSEC += 1_000_000; // incrementa di 1ms ad ogni chiamata
        SIM_NSEC
    };
    unsafe {
        let ts = ts_ptr as *mut u64;
        *ts       = ns / 1_000_000_000;   // tv_sec
        *ts.add(1)= ns % 1_000_000_000;   // tv_nsec
    }
    0
}

fn sys_nanosleep(req_ptr: u64, rem_ptr: u64) -> i64 {
    if req_ptr == 0 { return EFAULT; }
    // Leggi durata e "dormi" con nop loop
    unsafe {
        let req = req_ptr as *const u64;
        let sec  = *req;
        let nsec = *req.add(1);
        // In QEMU: ~500M nop/sec. Per 1ms = 500K nop
        let nops = (sec * 500_000 + nsec / 2000).min(50_000_000);
        for _ in 0..nops {
            core::arch::asm!("nop", options(nomem, nostack));
        }
        if rem_ptr != 0 {
            let rem = rem_ptr as *mut u64;
            *rem       = 0;
            *rem.add(1)= 0;
        }
    }
    0
}

fn sys_clock_nanosleep(clockid: i32, flags: i32, req_ptr: u64, rem_ptr: u64) -> i64 {
    sys_nanosleep(req_ptr, rem_ptr)
}

fn sys_gettimeofday(tv_ptr: u64, tz_ptr: u64) -> i64 {
    if tv_ptr == 0 { return 0; }
    let ns = unsafe { SIM_NSEC };
    unsafe {
        let tv = tv_ptr as *mut u64;
        *tv       = ns / 1_000_000_000;            // tv_sec
        *tv.add(1)= (ns % 1_000_000_000) / 1000;  // tv_usec
    }
    0
}

fn sys_sched_getaffinity(pid: u64, cpusetsize: usize, mask_ptr: u64) -> i64 {
    if mask_ptr == 0 { return EFAULT; }
    // Tutti i processi affini alla CPU 0 in ZTOS
    unsafe {
        let mask = core::slice::from_raw_parts_mut(mask_ptr as *mut u8, cpusetsize.min(8));
        mask[0] = 0x01; // solo CPU 0
        for b in mask[1..].iter_mut() { *b = 0; }
    }
    0
}
