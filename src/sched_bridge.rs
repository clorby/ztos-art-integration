use crate::syscall_wrapper::{EINVAL, EFAULT};
use crate::syscall_table::*;

static mut SIM_NSEC: u64 = 1_000_000_000;

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, _a4: u64, _a5: u64) -> i64 {
    match nr {
        SYS_CLOCK_GETTIME    => sys_clock_gettime(a0 as i32, a1),
        SYS_CLOCK_NANOSLEEP  => sys_nanosleep(a2, a3),
        SYS_NANOSLEEP        => sys_nanosleep(a0, a1),
        SYS_GETTIMEOFDAY     => sys_gettimeofday(a0),
        SYS_SCHED_YIELD      => 0,
        SYS_SCHED_SETSCHEDULER => 0,
        SYS_SCHED_GETSCHEDULER => 0,
        SYS_SCHED_SETAFFINITY => 0,
        SYS_SCHED_GETAFFINITY => sys_sched_getaffinity(a1 as usize, a2),
        SYS_GETRLIMIT        => crate::syscall_wrapper::handle_getrlimit_pub(a0, a1),
        SYS_SETRLIMIT        => 0,
        _                    => EINVAL,
    }
}

fn sys_clock_gettime(clockid: i32, ts_ptr: u64) -> i64 {
    if ts_ptr == 0 { return EFAULT; }
    let ns = unsafe { SIM_NSEC += 1_000_000; SIM_NSEC };
    unsafe {
        let ts = ts_ptr as *mut u64;
        *ts = ns / 1_000_000_000;
        *ts.add(1) = ns % 1_000_000_000;
    }
    0
}

fn sys_nanosleep(req_ptr: u64, rem_ptr: u64) -> i64 {
    if req_ptr == 0 { return EFAULT; }
    unsafe {
        let req = req_ptr as *const u64;
        let sec = *req; let nsec = *req.add(1);
        let nops = (sec * 100_000 + nsec / 10_000).min(10_000_000);
        for _ in 0..nops { core::arch::asm!("nop", options(nomem, nostack)); }
        if rem_ptr != 0 { let r = rem_ptr as *mut u64; *r = 0; *r.add(1) = 0; }
    }
    0
}

fn sys_gettimeofday(tv_ptr: u64) -> i64 {
    if tv_ptr == 0 { return 0; }
    let ns = unsafe { SIM_NSEC };
    unsafe {
        let tv = tv_ptr as *mut u64;
        *tv = ns / 1_000_000_000;
        *tv.add(1) = (ns % 1_000_000_000) / 1000;
    }
    0
}

fn sys_sched_getaffinity(cpusetsize: usize, mask_ptr: u64) -> i64 {
    if mask_ptr == 0 { return EFAULT; }
    unsafe {
        let mask = core::slice::from_raw_parts_mut(mask_ptr as *mut u8, cpusetsize.min(8));
        mask[0] = 0x01;
        for b in mask[1..].iter_mut() { *b = 0; }
    }
    0
}
