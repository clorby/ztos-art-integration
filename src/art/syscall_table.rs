// Mapping syscall Linux AArch64 → capability ZTOS.
// Numeri syscall da: arch/arm64/include/uapi/asm/unistd.h (Linux 6.x)

// ── Numeri syscall AArch64 ────────────────────────────────────────────────
pub const SYS_IO_SETUP:           u64 = 0;
pub const SYS_IO_DESTROY:         u64 = 1;
pub const SYS_EPOLL_CREATE1:      u64 = 20;
pub const SYS_EPOLL_CTL:          u64 = 21;
pub const SYS_EVENTFD2:           u64 = 19;
pub const SYS_OPENAT:             u64 = 56;
pub const SYS_CLOSE:              u64 = 57;
pub const SYS_GETDENTS64:         u64 = 61;
pub const SYS_LSEEK:              u64 = 62;
pub const SYS_READ:               u64 = 63;
pub const SYS_WRITE:              u64 = 64;
pub const SYS_READV:              u64 = 65;
pub const SYS_WRITEV:             u64 = 66;
pub const SYS_PREAD64:            u64 = 67;
pub const SYS_PWRITE64:           u64 = 68;
pub const SYS_PREADV:             u64 = 69;
pub const SYS_PWRITEV:            u64 = 70;
pub const SYS_SENDFILE:           u64 = 71;
pub const SYS_SIGNALFD4:          u64 = 74;
pub const SYS_READLINKAT:         u64 = 78;
pub const SYS_NEWFSTATAT:         u64 = 79;
pub const SYS_FSTAT:              u64 = 80;
pub const SYS_FSYNC:              u64 = 82;
pub const SYS_FDATASYNC:          u64 = 83;
pub const SYS_FTRUNCATE:          u64 = 46;
pub const SYS_FALLOCATE:          u64 = 47;
pub const SYS_FACCESSAT:          u64 = 48;
pub const SYS_STATFS:             u64 = 43;
pub const SYS_FSTATFS:            u64 = 44;
pub const SYS_FCNTL:              u64 = 25;
pub const SYS_IOCTL:              u64 = 29;
pub const SYS_DUP3:               u64 = 24;
pub const SYS_PIPE2:              u64 = 59;
pub const SYS_FUTEX:              u64 = 98;
pub const SYS_NANOSLEEP:          u64 = 101;
pub const SYS_CLOCK_GETTIME:      u64 = 113;
pub const SYS_CLOCK_SETTIME:      u64 = 112;
pub const SYS_CLOCK_NANOSLEEP:    u64 = 115;
pub const SYS_SCHED_YIELD:        u64 = 124;
pub const SYS_SCHED_SETSCHEDULER: u64 = 156;
pub const SYS_SCHED_GETSCHEDULER: u64 = 157;
pub const SYS_SCHED_SETAFFINITY:  u64 = 122;
pub const SYS_SCHED_GETAFFINITY:  u64 = 123;
pub const SYS_KILL:               u64 = 129;
pub const SYS_TKILL:              u64 = 130;
pub const SYS_TGKILL:             u64 = 131;
pub const SYS_RT_SIGACTION:       u64 = 134;
pub const SYS_RT_SIGPROCMASK:     u64 = 135;
pub const SYS_RT_SIGPENDING:      u64 = 136;
pub const SYS_RT_SIGSUSPEND:      u64 = 133;
pub const SYS_SIGALTSTACK:        u64 = 132;
pub const SYS_PRCTL:              u64 = 167;
pub const SYS_GETPID:             u64 = 172;
pub const SYS_GETPPID:            u64 = 173;
pub const SYS_GETUID:             u64 = 174;
pub const SYS_GETEUID:            u64 = 175;
pub const SYS_GETGID:             u64 = 176;
pub const SYS_GETEGID:            u64 = 177;
pub const SYS_GETTID:             u64 = 178;
pub const SYS_GETCPU:             u64 = 168;
pub const SYS_GETTIMEOFDAY:       u64 = 169;
pub const SYS_UNAME:              u64 = 160;
pub const SYS_GETRLIMIT:          u64 = 163;
pub const SYS_SETRLIMIT:          u64 = 164;
pub const SYS_GETPGID:            u64 = 155;
pub const SYS_SETPGID:            u64 = 154;
pub const SYS_GETSID:             u64 = 156;
pub const SYS_SETSID:             u64 = 157;
pub const SYS_CLONE:              u64 = 220;
pub const SYS_FORK:               u64 = 1079; // not on aarch64, uses clone
pub const SYS_EXECVE:             u64 = 221;
pub const SYS_WAIT4:              u64 = 260;
pub const SYS_EXIT:               u64 = 93;
pub const SYS_EXIT_GROUP:         u64 = 94;
pub const SYS_BRK:                u64 = 214;
pub const SYS_MMAP:               u64 = 222;
pub const SYS_MUNMAP:             u64 = 215;
pub const SYS_MPROTECT:           u64 = 226;
pub const SYS_MADVISE:            u64 = 233;
pub const SYS_MREMAP:             u64 = 216;
pub const SYS_MSYNC:              u64 = 227;
pub const SYS_MINCORE:            u64 = 232;
pub const SYS_MEMFD_CREATE:       u64 = 279;
pub const SYS_MEMBARRIER:         u64 = 283;
pub const SYS_MLOCK:              u64 = 228;
pub const SYS_MUNLOCK:            u64 = 229;
pub const SYS_SOCKET:             u64 = 198;
pub const SYS_SOCKETPAIR:         u64 = 199;
pub const SYS_BIND:               u64 = 200;
pub const SYS_LISTEN:             u64 = 201;
pub const SYS_ACCEPT:             u64 = 202;
pub const SYS_CONNECT:            u64 = 203;
pub const SYS_GETSOCKNAME:        u64 = 204;
pub const SYS_GETPEERNAME:        u64 = 205;
pub const SYS_SENDTO:             u64 = 206;
pub const SYS_RECVFROM:           u64 = 207;
pub const SYS_SETSOCKOPT:         u64 = 208;
pub const SYS_GETSOCKOPT:         u64 = 209;
pub const SYS_SENDMSG:            u64 = 211;
pub const SYS_RECVMSG:            u64 = 212;
pub const SYS_EPOLL_PWAIT:        u64 = 281;
pub const SYS_PTRACE:             u64 = 117;
pub const SYS_PROCESS_VM_READV:   u64 = 270;
pub const SYS_PROCESS_VM_WRITEV:  u64 = 271;
pub const SYS_PERF_EVENT_OPEN:    u64 = 241;
pub const SYS_GETRANDOM:          u64 = 278;
pub const SYS_SECCOMP:            u64 = 277;

// ── Capability ZTOS richieste per ogni syscall ────────────────────────────
// Usa bitmask delle costanti da kernel/capability.rs

/// Descrittore di una syscall con la sua capability e handler.
#[derive(Copy, Clone)]
pub struct SyscallDesc {
    pub nr:         u64,
    pub name:       &'static str,
    pub cap_needed: u32,       // 0 = nessuna (sempre permessa)
    pub category:   SyscallCat,
}

#[derive(Copy, Clone, PartialEq)]
pub enum SyscallCat {
    Memory,
    Io,
    Thread,
    Signal,
    Sched,
    Net,
    ProcInfo,
    Debug,
    Noop,   // implementata nel kernel ZTOS, nessun server
    Stub,   // stub sicuro (ritorna valore plausibile)
}

// Importa capability constants
use crate::kernel::capability::*;

pub const SYSCALL_TABLE: &[SyscallDesc] = &[
    // ── MEMORY ──────────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_MMAP,        name: "mmap",        cap_needed: CAP_APP_STORAGE | CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MUNMAP,      name: "munmap",      cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MPROTECT,    name: "mprotect",    cap_needed: CAP_MEMORY_PROT,  category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MADVISE,     name: "madvise",     cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MREMAP,      name: "mremap",      cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MSYNC,       name: "msync",       cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MINCORE,     name: "mincore",     cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MEMFD_CREATE,name: "memfd_create",cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MEMBARRIER,  name: "membarrier",  cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Noop },
    SyscallDesc { nr: SYS_BRK,        name: "brk",         cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Memory },
    SyscallDesc { nr: SYS_MLOCK,      name: "mlock",       cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Noop },
    SyscallDesc { nr: SYS_MUNLOCK,    name: "munlock",     cap_needed: CAP_MEMORY_ALLOC, category: SyscallCat::Noop },
    // ── IO ──────────────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_OPENAT,     name: "openat",      cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_CLOSE,      name: "close",       cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_READ,       name: "read",        cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_WRITE,      name: "write",       cap_needed: CAP_APP_STORAGE | CAP_STDOUT_WRITE, category: SyscallCat::Io },
    SyscallDesc { nr: SYS_PREAD64,    name: "pread64",     cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_PWRITE64,   name: "pwrite64",    cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_READV,      name: "readv",       cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_WRITEV,     name: "writev",      cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_FSTAT,      name: "fstat",       cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_NEWFSTATAT, name: "newfstatat",  cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_LSEEK,      name: "lseek",       cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_FCNTL,      name: "fcntl",       cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_IOCTL,      name: "ioctl",       cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_DUP3,       name: "dup3",        cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_PIPE2,      name: "pipe2",       cap_needed: 0,                category: SyscallCat::Noop },
    SyscallDesc { nr: SYS_FSYNC,      name: "fsync",       cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_FALLOCATE,  name: "fallocate",   cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_FTRUNCATE,  name: "ftruncate",   cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_FACCESSAT,  name: "faccessat",   cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_READLINKAT, name: "readlinkat",  cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_GETDENTS64, name: "getdents64",  cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    SyscallDesc { nr: SYS_STATFS,     name: "statfs",      cap_needed: CAP_APP_STORAGE,  category: SyscallCat::Io },
    // ── THREAD ──────────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_CLONE,      name: "clone",       cap_needed: CAP_THREAD_CREATE,category: SyscallCat::Thread },
    SyscallDesc { nr: SYS_FUTEX,      name: "futex",       cap_needed: 0,                category: SyscallCat::Thread },
    SyscallDesc { nr: SYS_EVENTFD2,   name: "eventfd2",    cap_needed: 0,                category: SyscallCat::Thread },
    SyscallDesc { nr: SYS_EXIT,       name: "exit",        cap_needed: 0,                category: SyscallCat::Thread },
    SyscallDesc { nr: SYS_EXIT_GROUP, name: "exit_group",  cap_needed: 0,                category: SyscallCat::Thread },
    SyscallDesc { nr: SYS_WAIT4,      name: "wait4",       cap_needed: CAP_PROC_CONTROL, category: SyscallCat::Thread },
    // ── SIGNAL ──────────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_RT_SIGACTION,  name: "rt_sigaction",  cap_needed: 0, category: SyscallCat::Signal },
    SyscallDesc { nr: SYS_RT_SIGPROCMASK,name: "rt_sigprocmask",cap_needed: 0, category: SyscallCat::Signal },
    SyscallDesc { nr: SYS_RT_SIGSUSPEND, name: "rt_sigsuspend", cap_needed: 0, category: SyscallCat::Noop },
    SyscallDesc { nr: SYS_SIGALTSTACK,   name: "sigaltstack",   cap_needed: 0, category: SyscallCat::Signal },
    SyscallDesc { nr: SYS_TGKILL,        name: "tgkill",        cap_needed: CAP_SIGNAL, category: SyscallCat::Signal },
    SyscallDesc { nr: SYS_TKILL,         name: "tkill",         cap_needed: CAP_SIGNAL, category: SyscallCat::Signal },
    SyscallDesc { nr: SYS_KILL,          name: "kill",          cap_needed: CAP_SIGNAL, category: SyscallCat::Signal },
    // ── SCHED / TIMING ──────────────────────────────────────────────────
    SyscallDesc { nr: SYS_SCHED_YIELD,        name: "sched_yield",        cap_needed: 0, category: SyscallCat::Noop },
    SyscallDesc { nr: SYS_SCHED_SETSCHEDULER, name: "sched_setscheduler", cap_needed: CAP_SCHED, category: SyscallCat::Sched },
    SyscallDesc { nr: SYS_SCHED_GETSCHEDULER, name: "sched_getscheduler", cap_needed: 0, category: SyscallCat::Sched },
    SyscallDesc { nr: SYS_SCHED_SETAFFINITY,  name: "sched_setaffinity",  cap_needed: CAP_SCHED, category: SyscallCat::Sched },
    SyscallDesc { nr: SYS_SCHED_GETAFFINITY,  name: "sched_getaffinity",  cap_needed: 0, category: SyscallCat::Sched },
    SyscallDesc { nr: SYS_NANOSLEEP,      name: "nanosleep",    cap_needed: 0, category: SyscallCat::Sched },
    SyscallDesc { nr: SYS_CLOCK_GETTIME,  name: "clock_gettime",cap_needed: 0, category: SyscallCat::Sched },
    SyscallDesc { nr: SYS_CLOCK_NANOSLEEP,name: "clock_nanosleep",cap_needed:0,category: SyscallCat::Sched },
    SyscallDesc { nr: SYS_GETTIMEOFDAY,   name: "gettimeofday", cap_needed: 0, category: SyscallCat::Sched },
    // ── PROC INFO ───────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_GETPID,    name: "getpid",    cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_GETTID,    name: "gettid",    cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_GETPPID,   name: "getppid",   cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_GETUID,    name: "getuid",    cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_GETEUID,   name: "geteuid",   cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_GETGID,    name: "getgid",    cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_GETEGID,   name: "getegid",   cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_GETCPU,    name: "getcpu",    cap_needed: 0, category: SyscallCat::Stub },
    SyscallDesc { nr: SYS_UNAME,     name: "uname",     cap_needed: 0, category: SyscallCat::Stub },
    SyscallDesc { nr: SYS_GETRLIMIT, name: "getrlimit", cap_needed: 0, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_SETRLIMIT, name: "setrlimit", cap_needed: CAP_PROC_CONTROL, category: SyscallCat::ProcInfo },
    SyscallDesc { nr: SYS_PRCTL,     name: "prctl",     cap_needed: 0, category: SyscallCat::Thread },
    // ── NET ─────────────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_SOCKET,      name: "socket",      cap_needed: CAP_NET_SEND, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_CONNECT,     name: "connect",     cap_needed: CAP_NET_SEND, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_BIND,        name: "bind",        cap_needed: CAP_NET_RECV, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_LISTEN,      name: "listen",      cap_needed: CAP_NET_RECV, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_ACCEPT,      name: "accept",      cap_needed: CAP_NET_RECV, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_SENDTO,      name: "sendto",      cap_needed: CAP_NET_SEND, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_RECVFROM,    name: "recvfrom",    cap_needed: CAP_NET_RECV, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_SENDMSG,     name: "sendmsg",     cap_needed: CAP_NET_SEND, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_RECVMSG,     name: "recvmsg",     cap_needed: CAP_NET_RECV, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_SETSOCKOPT,  name: "setsockopt",  cap_needed: CAP_NET_SEND, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_GETSOCKOPT,  name: "getsockopt",  cap_needed: CAP_NET_SEND, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_GETSOCKNAME, name: "getsockname", cap_needed: CAP_NET_SEND, category: SyscallCat::Net },
    SyscallDesc { nr: SYS_EPOLL_CREATE1,name: "epoll_create1",cap_needed: CAP_NET_RECV,category: SyscallCat::Net },
    SyscallDesc { nr: SYS_EPOLL_PWAIT, name: "epoll_pwait", cap_needed: CAP_NET_RECV, category: SyscallCat::Net },
    // ── MISC ────────────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_GETRANDOM,  name: "getrandom",  cap_needed: CAP_ENTROPY,   category: SyscallCat::Stub },
    SyscallDesc { nr: SYS_SECCOMP,    name: "seccomp",    cap_needed: 0,             category: SyscallCat::Noop },
    // ── DEBUG ────────────────────────────────────────────────────────────
    SyscallDesc { nr: SYS_PTRACE,          name: "ptrace",           cap_needed: CAP_DEBUG, category: SyscallCat::Debug },
    SyscallDesc { nr: SYS_PROCESS_VM_READV, name: "process_vm_readv", cap_needed: CAP_DEBUG, category: SyscallCat::Debug },
    SyscallDesc { nr: SYS_PROCESS_VM_WRITEV,name: "process_vm_writev",cap_needed: CAP_DEBUG, category: SyscallCat::Debug },
    SyscallDesc { nr: SYS_PERF_EVENT_OPEN,  name: "perf_event_open",  cap_needed: CAP_PERF,  category: SyscallCat::Debug },
];

/// Cerca il descrittore per numero syscall.
pub fn lookup(nr: u64) -> Option<&'static SyscallDesc> {
    SYSCALL_TABLE.iter().find(|d| d.nr == nr)
}
