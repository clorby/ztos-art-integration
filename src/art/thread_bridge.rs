// Thread bridge: clone, futex, prctl, signal, exit.

use crate::art::syscall_wrapper::{EPERM, EINVAL, ENOMEM};
use crate::art::syscall_table::*;

// ── CLONE flags ──────────────────────────────────────────────────────────
const CLONE_VM:         u64 = 0x00000100;
const CLONE_FS:         u64 = 0x00000200;
const CLONE_FILES:      u64 = 0x00000400;
const CLONE_SIGHAND:    u64 = 0x00000800;
const CLONE_THREAD:     u64 = 0x00010000;
const CLONE_SETTLS:     u64 = 0x00080000;
const CLONE_CHILD_CLEARTID: u64 = 0x00200000;

// ── Thread table ZTOS ────────────────────────────────────────────────────
const MAX_THREADS: usize = 64;

#[derive(Copy, Clone)]
struct ThreadEntry {
    tid:        u32,
    func_ptr:   u64,  // entry function (fn() in Rust ABI)
    stack_ptr:  u64,
    tls_ptr:    u64,
    ctid_ptr:   u64,  // clear_child_tid
    app_id:     u32,
    valid:      bool,
}

impl ThreadEntry {
    const fn empty() -> Self {
        Self { tid:0, func_ptr:0, stack_ptr:0, tls_ptr:0, ctid_ptr:0, app_id:0, valid:false }
    }
}

static mut THREAD_TABLE: [ThreadEntry; MAX_THREADS] = [ThreadEntry::empty(); MAX_THREADS];
static mut NEXT_TID: u32 = 100;

// ── Futex state ──────────────────────────────────────────────────────────
// In ZTOS senza scheduler preemptive, i futex sono no-op per i wait (single thread cooperativo)
// ma usiamo una tabella per tracciare gli waiters per correttezza formale.
const MAX_FUTEX: usize = 256;

#[derive(Copy, Clone)]
struct FutexEntry {
    addr:    u64,
    waiters: u32,
}

impl FutexEntry {
    const fn empty() -> Self { Self { addr: 0, waiters: 0 } }
}

static mut FUTEX_TABLE: [FutexEntry; MAX_FUTEX] = [FutexEntry::empty(); MAX_FUTEX];

// ── Signal table ─────────────────────────────────────────────────────────
#[derive(Copy, Clone)]
struct SigAction {
    handler: u64,  // SA_SIGACTION puntatore
    flags:   u64,
    valid:   bool,
}

impl SigAction {
    const fn empty() -> Self { Self { handler: 0, flags: 0, valid: false } }
}

static mut SIG_TABLE: [SigAction; 65] = [SigAction::empty(); 65];
static mut SIG_MASK: u64 = 0;
static mut ALT_STACK_PTR: u64 = 0;
static mut ALT_STACK_SIZE: usize = 0;

// ── Dispatcher ───────────────────────────────────────────────────────────

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    match nr {
        SYS_CLONE     => sys_clone(a0, a1, a2, a3, a4),
        SYS_FUTEX     => sys_futex(a0, a1 as i32, a2 as i32, a3, a4, a5 as i32),
        SYS_EVENTFD2  => sys_eventfd2(a0 as u32, a1 as u32),
        SYS_EXIT      => sys_exit(a0 as i32),
        SYS_EXIT_GROUP=> sys_exit(a0 as i32),
        SYS_WAIT4     => sys_wait4(a0 as i32, a1, a2 as i32),
        SYS_PRCTL     => crate::art::syscall_wrapper::handle_prctl_pub(a0, a1, a2, a3, a4),
        _             => EINVAL,
    }
}

pub fn handle_signal(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    match nr {
        SYS_RT_SIGACTION  => sys_rt_sigaction(a0 as i32, a1, a2, a3 as usize),
        SYS_RT_SIGPROCMASK=> sys_rt_sigprocmask(a0 as i32, a1, a2, a3 as usize),
        SYS_RT_SIGSUSPEND => 0, // stub
        SYS_SIGALTSTACK   => sys_sigaltstack(a0, a1),
        SYS_TGKILL        => sys_tgkill(a0 as i32, a1 as i32, a2 as i32),
        SYS_TKILL         => sys_tkill(a0 as i32, a1 as i32),
        SYS_KILL          => sys_kill(a0 as i32, a1 as i32),
        _                 => EINVAL,
    }
}

// ── clone ─────────────────────────────────────────────────────────────────

fn sys_clone(flags: u64, stack: u64, parent_tid: u64, child_tid: u64, tls: u64) -> i64 {
    if flags & CLONE_THREAD == 0 {
        // fork() via clone: ritorna 0 per figlio, pid per padre (stub: no fork reale)
        return unsafe { NEXT_TID as i64 };
    }

    let tid = unsafe {
        let t = NEXT_TID;
        NEXT_TID += 1;
        t
    };

    let entry = ThreadEntry {
        tid,
        func_ptr:  0,  // In produzione: recuperato da stack (bionic lo mette lì)
        stack_ptr: stack,
        tls_ptr:   tls,
        ctid_ptr:  child_tid,
        app_id:    crate::art::syscall_wrapper::current_app_id(),
        valid:     true,
    };

    unsafe {
        for slot in THREAD_TABLE.iter_mut() {
            if !slot.valid { *slot = entry; break; }
        }
        // Scrivi il TID nel child_tid_ptr (per CLONE_CHILD_CLEARTID)
        if child_tid != 0 && flags & CLONE_CHILD_CLEARTID != 0 {
            *(child_tid as *mut u32) = tid;
        }
        // Scrivi il TID nel parent_tid_ptr (per CLONE_PARENT_SETTID)
        if parent_tid != 0 {
            *(parent_tid as *mut u32) = tid;
        }
    }

    // In ZTOS V0.7, i thread ART girano cooperativamente.
    // La funzione thread viene schedulata dallo scheduler ZTOS.
    // Per ora la registriamo ma non la eseguiamo immediatamente.
    tid as i64
}

// ── futex ─────────────────────────────────────────────────────────────────

const FUTEX_WAIT:        i32 = 0;
const FUTEX_WAKE:        i32 = 1;
const FUTEX_WAIT_BITSET: i32 = 9;
const FUTEX_WAKE_BITSET: i32 = 10;
const FUTEX_PRIVATE:     i32 = 128;

fn sys_futex(uaddr: u64, op: i32, val: i32, timeout: u64, uaddr2: u64, val3: i32) -> i64 {
    let real_op = op & !FUTEX_PRIVATE;
    match real_op {
        FUTEX_WAIT | FUTEX_WAIT_BITSET => {
            // In ZTOS single-threaded: se il valore atteso non corrisponde, ritorna EAGAIN
            // Altrimenti, lo scheduler dovrebbe cedere il controllo (futex_wait cooperativo)
            let current_val = unsafe { core::ptr::read_volatile(uaddr as *const i32) };
            if current_val != val { return -11; } // EAGAIN
            // No sleep reale in V0.7: ritorna immediatamente (ART gestisce i loop)
            0
        }
        FUTEX_WAKE | FUTEX_WAKE_BITSET => {
            // Wake: ritorna il numero di thread svegliati (stub: 1)
            val.min(1) as i64
        }
        _ => 0,
    }
}

// ── eventfd ──────────────────────────────────────────────────────────────

fn sys_eventfd2(initval: u32, flags: u32) -> i64 {
    // Ritorna un fd virtuale per event notification
    static mut NEXT_EVFD: i32 = 500;
    unsafe {
        let fd = NEXT_EVFD;
        NEXT_EVFD += 1;
        fd as i64
    }
}

// ── exit ─────────────────────────────────────────────────────────────────

fn sys_exit(status: i32) -> i64 {
    // In ZTOS, terminazione dell'app
    let app_id = crate::art::syscall_wrapper::current_app_id();
    crate::kernel::capability::revoke_all_for_task(app_id);
    // Il task ZTOS viene rimosso dallo scheduler nella versione futura
    loop { unsafe { core::arch::asm!("wfe"); } }
}

fn sys_wait4(pid: i32, status_ptr: u64, options: i32) -> i64 {
    // No child processes in V0.7
    -10 // ECHILD
}

// ── signal handling ───────────────────────────────────────────────────────

fn sys_rt_sigaction(signum: i32, act_ptr: u64, oldact_ptr: u64, sigsetsize: usize) -> i64 {
    if signum < 1 || signum > 64 { return EINVAL; }
    unsafe {
        if oldact_ptr != 0 && SIG_TABLE[signum as usize].valid {
            let old = oldact_ptr as *mut u64;
            *old = SIG_TABLE[signum as usize].handler;
            *old.add(1) = SIG_TABLE[signum as usize].flags;
            *old.add(2) = 0; // sa_mask
            *old.add(3) = 0;
        }
        if act_ptr != 0 {
            let act = act_ptr as *const u64;
            SIG_TABLE[signum as usize] = SigAction {
                handler: *act,
                flags:   *act.add(1),
                valid:   true,
            };
        }
    }
    0
}

fn sys_rt_sigprocmask(how: i32, set_ptr: u64, oldset_ptr: u64, sigsetsize: usize) -> i64 {
    unsafe {
        if oldset_ptr != 0 { *(oldset_ptr as *mut u64) = SIG_MASK; }
        if set_ptr != 0 {
            let new_mask = *(set_ptr as *const u64);
            match how {
                0 => SIG_MASK |=  new_mask, // SIG_BLOCK
                1 => SIG_MASK &= !new_mask, // SIG_UNBLOCK
                2 => SIG_MASK  =  new_mask, // SIG_SETMASK
                _ => return EINVAL,
            }
        }
    }
    0
}

fn sys_sigaltstack(ss_ptr: u64, oss_ptr: u64) -> i64 {
    unsafe {
        if oss_ptr != 0 {
            // Ritorna stack alternativo corrente
            let oss = oss_ptr as *mut u64;
            *oss       = ALT_STACK_PTR;
            *oss.add(1)= if ALT_STACK_PTR == 0 { 2 } else { 0 }; // SS_DISABLE o SS_ONSTACK
            *oss.add(2)= ALT_STACK_SIZE as u64;
        }
        if ss_ptr != 0 {
            let ss = ss_ptr as *const u64;
            ALT_STACK_PTR  = *ss;
            ALT_STACK_SIZE = *ss.add(2) as usize;
        }
    }
    0
}

fn sys_tgkill(tgid: i32, tid: i32, sig: i32) -> i64 {
    // In ZTOS, il segnale SIGSEGV (11) deve essere gestito (stack overflow)
    // Gli altri segnali sono stub
    if sig == 11 && ALT_STACK_PTR != 0 {
        // SIGSEGV: invoca il signal handler se registrato
        unsafe {
            if SIG_TABLE[11].valid && SIG_TABLE[11].handler != 0 {
                // In produzione: esegui l'handler via indirect call
                // type Handler = extern "C" fn(i32);
                // let h: Handler = core::mem::transmute(SIG_TABLE[11].handler);
                // h(sig);
            }
        }
    }
    0
}

fn sys_tkill(tid: i32, sig: i32) -> i64 { sys_tgkill(0, tid, sig) }
fn sys_kill(pid: i32, sig: i32) -> i64  { sys_tgkill(pid, 0, sig) }

// Wrapper pubblico per prctl (chiamato anche da handle_proc_info)
pub fn handle_prctl_from_thread(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64 {
    crate::art::syscall_wrapper::handle_prctl_pub(a0, a1, a2, a3, a4)
}
