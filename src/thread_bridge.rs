use crate::syscall_wrapper::{EPERM, EINVAL, EFAULT};
use crate::syscall_table::*;

const MAX_THREADS: usize = 64;
const MAX_FUTEX:   usize = 256;

#[derive(Copy, Clone)]
struct ThreadEntry { tid: u32, stack: u64, tls: u64, ctid: u64, app_id: u32, valid: bool }
impl ThreadEntry { const fn empty() -> Self { Self{tid:0,stack:0,tls:0,ctid:0,app_id:0,valid:false} } }

#[derive(Copy, Clone)]
struct SigAction { handler: u64, flags: u64, valid: bool }
impl SigAction { const fn empty() -> Self { Self{handler:0,flags:0,valid:false} } }

static mut THREAD_TABLE: [ThreadEntry; MAX_THREADS] = [ThreadEntry::empty(); MAX_THREADS];
static mut NEXT_TID:     u32 = 100;
static mut SIG_TABLE:    [SigAction; 65] = [SigAction::empty(); 65];
static mut SIG_MASK:     u64 = 0;
static mut ALT_STACK:    u64 = 0;
static mut ALT_STACK_SZ: usize = 0;

const CLONE_THREAD: u64 = 0x0001_0000;
const FUTEX_WAIT:   i32 = 0;
const FUTEX_WAKE:   i32 = 1;
const FUTEX_WAIT_BITSET: i32 = 9;
const FUTEX_WAKE_BITSET: i32 = 10;
const FUTEX_PRIVATE:     i32 = 128;

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    match nr {
        SYS_CLONE     => sys_clone(a0, a1, a2, a3, a4),
        SYS_FUTEX     => sys_futex(a0, a1 as i32, a2 as i32, a3, a4, a5 as i32),
        SYS_EVENTFD2  => { static mut N: i32 = 500; unsafe { let f=N; N+=1; f as i64 } },
        SYS_EXIT       => sys_exit(a0 as i32),
        SYS_EXIT_GROUP => sys_exit(a0 as i32),
        SYS_WAIT4     => -10, // ECHILD
        SYS_PRCTL     => crate::syscall_wrapper::handle_prctl_pub(a0, a1, a2, 0, 0),
        _             => EINVAL,
    }
}

pub fn handle_signal(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, _a4: u64, _a5: u64) -> i64 {
    match nr {
        SYS_RT_SIGACTION   => sys_rt_sigaction(a0 as i32, a1, a2),
        SYS_RT_SIGPROCMASK => sys_rt_sigprocmask(a0 as i32, a1, a2),
        SYS_RT_SIGSUSPEND  => 0,
        SYS_SIGALTSTACK    => sys_sigaltstack(a0, a1),
        SYS_TGKILL => sys_tgkill(a2 as i32),
        SYS_TKILL  => sys_tgkill(a1 as i32),
        SYS_KILL   => sys_tgkill(a1 as i32),
        _                  => EINVAL,
    }
}

fn sys_clone(flags: u64, stack: u64, parent_tid: u64, child_tid: u64, tls: u64) -> i64 {
    let tid = unsafe { let t = NEXT_TID; NEXT_TID += 1; t };
    let entry = ThreadEntry {
        tid, stack, tls, ctid: child_tid,
        app_id: crate::syscall_wrapper::current_app_id(), valid: true,
    };
    unsafe {
        for slot in THREAD_TABLE.iter_mut() {
            if !slot.valid { *slot = entry; break; }
        }
        if child_tid != 0 && flags & CLONE_THREAD != 0 { *(child_tid as *mut u32) = tid; }
        if parent_tid != 0 { *(parent_tid as *mut u32) = tid; }
    }
    tid as i64
}

fn sys_futex(uaddr: u64, op: i32, val: i32, _timeout: u64, _uaddr2: u64, _val3: i32) -> i64 {
    let real_op = op & !FUTEX_PRIVATE;
    match real_op {
        FUTEX_WAIT | FUTEX_WAIT_BITSET => {
            let cur = unsafe { core::ptr::read_volatile(uaddr as *const i32) };
            if cur != val { return -11; } // EAGAIN
            0
        }
        FUTEX_WAKE | FUTEX_WAKE_BITSET => val.min(1) as i64,
        _ => 0,
    }
}

fn sys_exit(_status: i32) -> i64 {
    let app_id = crate::syscall_wrapper::current_app_id();
    crate::kernel_stubs::revoke_all_for_task(app_id);
    loop { unsafe { core::arch::asm!("wfe", options(nomem, nostack)); } }
}

fn sys_rt_sigaction(signum: i32, act_ptr: u64, oldact_ptr: u64) -> i64 {
    if signum < 1 || signum > 64 { return EINVAL; }
    unsafe {
        if oldact_ptr != 0 && SIG_TABLE[signum as usize].valid {
            let o = oldact_ptr as *mut u64;
            *o = SIG_TABLE[signum as usize].handler;
            *o.add(1) = SIG_TABLE[signum as usize].flags;
        }
        if act_ptr != 0 {
            let a = act_ptr as *const u64;
            SIG_TABLE[signum as usize] = SigAction { handler: *a, flags: *a.add(1), valid: true };
        }
    }
    0
}

fn sys_rt_sigprocmask(how: i32, set_ptr: u64, oldset_ptr: u64) -> i64 {
    unsafe {
        if oldset_ptr != 0 { *(oldset_ptr as *mut u64) = SIG_MASK; }
        if set_ptr != 0 {
            let new = *(set_ptr as *const u64);
            match how { 0 => SIG_MASK|=new, 1 => SIG_MASK&=!new, 2 => SIG_MASK=new, _ => return EINVAL }
        }
    }
    0
}

fn sys_sigaltstack(ss_ptr: u64, oss_ptr: u64) -> i64 {
    unsafe {
        if oss_ptr != 0 {
            let o = oss_ptr as *mut u64;
            *o = ALT_STACK; *o.add(1) = if ALT_STACK==0 {2} else {0}; *o.add(2) = ALT_STACK_SZ as u64;
        }
        if ss_ptr != 0 {
            let s = ss_ptr as *const u64;
            ALT_STACK = *s; ALT_STACK_SZ = *s.add(2) as usize;
        }
    }
    0
}

fn sys_tgkill(sig: i32) -> i64 { 0 }
