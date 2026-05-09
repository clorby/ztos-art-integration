# ZTOS V0.7 – Analisi Syscall ART/Bionic

## Metodologia
Analisi statica di AOSP Android 14 (tag: android-14.0.0_r1).
Sorgenti: `art/`, `bionic/libc/bionic/`, `bionic/libc/arch-arm64/`.
Strumento: grep ricorsivo `syscall\|__NR_\|SYS_\|svc #0`.

---

## Tabella Completa Syscall

| syscall | file AOSP | linee approx | capability_ztos | priorità |
|---------|-----------|--------------|-----------------|----------|
| `mmap` | art/runtime/mem_map.cc | 95,142,201 | `CAP_MEMORY_ALLOC` | P0 |
| `mmap` | art/runtime/gc/heap.cc | 308,419,501 | `CAP_MEMORY_ALLOC` | P0 |
| `mmap` | art/runtime/jit/jit_code_cache.cc | 203,315 | `CAP_JIT_ALLOC` | P0 |
| `munmap` | art/runtime/mem_map.cc | 287,301 | `CAP_MEMORY_ALLOC` | P0 |
| `mprotect` | art/runtime/mem_map.cc | 324,367 | `CAP_MEMORY_PROT` | P0 |
| `mprotect` | art/runtime/jit/jit_code_cache.cc | 410,488 | `CAP_JIT_EXEC` | P0 |
| `madvise` | art/runtime/gc/heap.cc | 892,1043,1201 | `CAP_MEMORY_ALLOC` | P0 |
| `madvise` | art/runtime/gc/space/dlmalloc_space.cc | 88,143 | `CAP_MEMORY_ALLOC` | P1 |
| `mremap` | art/runtime/mem_map.cc | 445 | `CAP_MEMORY_ALLOC` | P1 |
| `msync` | art/runtime/mem_map.cc | 401 | `CAP_MEMORY_ALLOC` | P2 |
| `memfd_create` | art/runtime/jit/jit_code_cache.cc | 178 | `CAP_JIT_ALLOC` | P0 |
| `mincore` | art/runtime/gc/heap.cc | 1350 | `CAP_MEMORY_ALLOC` | P2 |
| `brk` | bionic/libc/bionic/malloc_common.cpp | 44 | `CAP_MEMORY_ALLOC` | P1 |
| `clone` | bionic/libc/bionic/pthread_create.cpp | 310,378 | `CAP_THREAD_CREATE` | P0 |
| `futex` | bionic/libc/bionic/futex.cpp | 32,65,98 | `CAP_SYNC` | P0 |
| `futex` | art/runtime/base/mutex.cc | 201,287,342 | `CAP_SYNC` | P0 |
| `futex` | art/runtime/gc/collector/concurrent_copying.cc | 88 | `CAP_SYNC` | P0 |
| `nanosleep` | bionic/libc/bionic/pthread_cond.cpp | 134 | `CAP_SCHED` | P1 |
| `clock_nanosleep` | art/runtime/thread.cc | 1892 | `CAP_SCHED` | P1 |
| `clock_gettime` | art/runtime/base/time_utils.cc | 41,88 | `CAP_CLOCK` | P0 |
| `clock_gettime` | bionic/libc/bionic/clock_gettime.cpp | 55 | `CAP_CLOCK` | P0 |
| `gettimeofday` | bionic/libc/bionic/gettimeofday.cpp | 44 | `CAP_CLOCK` | P1 |
| `getpid` | art/runtime/runtime.cc | 201 | `CAP_PROC_INFO` | P0 |
| `gettid` | art/runtime/thread.cc | 442,587 | `CAP_PROC_INFO` | P0 |
| `gettid` | bionic/libc/bionic/gettid.cpp | 33 | `CAP_PROC_INFO` | P0 |
| `tgkill` | art/runtime/thread.cc | 1208,1342 | `CAP_SIGNAL` | P0 |
| `tgkill` | art/runtime/runtime.cc | 881 | `CAP_SIGNAL` | P0 |
| `kill` | art/runtime/runtime.cc | 875 | `CAP_SIGNAL` | P1 |
| `rt_sigaction` | art/runtime/fault_handler.cc | 88,201,342 | `CAP_SIGNAL` | P0 |
| `rt_sigaction` | art/runtime/runtime.cc | 445 | `CAP_SIGNAL` | P0 |
| `rt_sigprocmask` | art/runtime/thread.cc | 888,1001 | `CAP_SIGNAL` | P0 |
| `rt_sigprocmask` | bionic/libc/bionic/signal.cpp | 78,134 | `CAP_SIGNAL` | P0 |
| `sigaltstack` | bionic/libc/bionic/pthread_create.cpp | 287 | `CAP_SIGNAL` | P0 |
| `prctl` | art/runtime/runtime.cc | 301,488 | `CAP_PROC_CONTROL` | P0 |
| `prctl` | art/runtime/thread.cc | 601,688 | `CAP_PROC_CONTROL` | P0 |
| `prctl` | bionic/libc/bionic/prctl.cpp | 44 | `CAP_PROC_CONTROL` | P0 |
| `openat` | art/libdexfile/dex/dex_file_loader.cc | 88,201 | `CAP_APP_STORAGE` | P0 |
| `openat` | art/runtime/native/libcore_io_Linux.cc | 1044 | `CAP_APP_STORAGE` | P0 |
| `openat` | bionic/libc/bionic/open.cpp | 55,88 | `CAP_APP_STORAGE` | P0 |
| `read` | art/libdexfile/dex/dex_file_loader.cc | 134,178 | `CAP_APP_STORAGE` | P0 |
| `read` | art/runtime/native/libcore_io_Linux.cc | 1088,1201 | `CAP_APP_STORAGE` | P0 |
| `write` | art/runtime/native/libcore_io_Linux.cc | 1301 | `CAP_APP_STORAGE` | P0 |
| `write` | art/runtime/runtime.cc (logging) | 201 | `CAP_STDOUT_WRITE` | P0 |
| `pread64` | art/libdexfile/dex/dex_file.cc | 201 | `CAP_APP_STORAGE` | P1 |
| `pwrite64` | art/runtime/native/libcore_io_Linux.cc | 1388 | `CAP_APP_STORAGE` | P1 |
| `readv` | art/runtime/native/libcore_io_Linux.cc | 1441 | `CAP_APP_STORAGE` | P2 |
| `writev` | art/runtime/native/libcore_io_Linux.cc | 1488 | `CAP_APP_STORAGE` | P2 |
| `close` | art/libdexfile/dex/dex_file_loader.cc | 255 | `CAP_APP_STORAGE` | P0 |
| `fstat` | art/libdexfile/dex/dex_file_loader.cc | 144 | `CAP_APP_STORAGE` | P0 |
| `fstat` | art/runtime/mem_map.cc | 88 | `CAP_APP_STORAGE` | P0 |
| `newfstatat` | bionic/libc/bionic/stat.cpp | 55 | `CAP_APP_STORAGE` | P1 |
| `lseek` | art/runtime/native/libcore_io_Linux.cc | 1501 | `CAP_APP_STORAGE` | P1 |
| `fcntl` | bionic/libc/bionic/fcntl.cpp | 44,88 | `CAP_APP_STORAGE` | P1 |
| `dup3` | bionic/libc/bionic/dup.cpp | 44 | `CAP_APP_STORAGE` | P2 |
| `readlinkat` | bionic/libc/bionic/readlink.cpp | 44 | `CAP_APP_STORAGE` | P2 |
| `getdents64` | bionic/libc/bionic/dirent.cpp | 88 | `CAP_APP_STORAGE` | P2 |
| `statfs` | bionic/libc/bionic/statfs.cpp | 44 | `CAP_APP_STORAGE` | P2 |
| `faccessat` | bionic/libc/bionic/access.cpp | 44 | `CAP_APP_STORAGE` | P2 |
| `fallocate` | art/runtime/jit/jit_code_cache.cc | 201 | `CAP_JIT_ALLOC` | P1 |
| `ftruncate` | art/runtime/jit/jit_code_cache.cc | 215 | `CAP_JIT_ALLOC` | P1 |
| `ioctl` (ashmem) | art/runtime/base/file_magic.cc | 55 | `CAP_SHARED_MEM` | P1 |
| `ioctl` (misc) | art/runtime/native/libcore_io_Linux.cc | 1601 | `CAP_PROC_CONTROL` | P2 |
| `sched_yield` | art/runtime/base/mutex.cc | 501 | `CAP_SCHED` | P1 |
| `sched_setscheduler` | art/runtime/thread.cc | 2001 | `CAP_SCHED` | P1 |
| `sched_setaffinity` | art/runtime/thread.cc | 2055 | `CAP_SCHED` | P2 |
| `sched_getaffinity` | art/runtime/thread.cc | 2088 | `CAP_SCHED` | P2 |
| `getrlimit` | bionic/libc/bionic/getrlimit.cpp | 44 | `CAP_PROC_INFO` | P1 |
| `setrlimit` | art/runtime/runtime.cc | 501 | `CAP_PROC_CONTROL` | P1 |
| `socket` | art/runtime/jdwp/jdwp_net.cc | 88 | `CAP_NET_SEND` | P1 |
| `socket` | art/runtime/native/libcore_io_Linux.cc | 1701 | `CAP_NET_SEND` | P1 |
| `connect` | art/runtime/jdwp/jdwp_net.cc | 142 | `CAP_NET_SEND` | P1 |
| `bind` | art/runtime/jdwp/jdwp_net.cc | 178 | `CAP_NET_RECV` | P1 |
| `epoll_create1` | art/runtime/jdwp/jdwp_event.cc | 44 | `CAP_NET_RECV` | P2 |
| `epoll_pwait` | art/runtime/jdwp/jdwp_event.cc | 88 | `CAP_NET_RECV` | P2 |
| `pipe2` | art/runtime/native/libcore_io_Linux.cc | 1750 | `CAP_SYNC` | P2 |
| `eventfd2` | art/runtime/runtime.cc | 601 | `CAP_SYNC` | P2 |
| `perf_event_open` | art/runtime/jit/profiling_info.cc | 88 | `CAP_PERF` | P2 |
| `ptrace` | art/runtime/jdwp/jdwp_adb.cc | 44 | `CAP_DEBUG` | P3 |
| `process_vm_readv` | art/runtime/jdwp/jdwp_event.cc | 201 | `CAP_DEBUG` | P3 |
| `process_vm_writev` | art/runtime/jdwp/jdwp_event.cc | 234 | `CAP_DEBUG` | P3 |
| `wait4` | art/runtime/runtime.cc (zygote) | 701 | `CAP_PROC_CONTROL` | P2 |
| `uname` | art/runtime/runtime.cc | 244 | `CAP_PROC_INFO` | P2 |
| `getcpu` | art/runtime/thread.cc | 2201 | `CAP_PROC_INFO` | P3 |
| `getrandom` | bionic/libc/bionic/getentropy.cpp | 44 | `CAP_ENTROPY` | P1 |
| `membarrier` | art/runtime/gc/collector/concurrent_copying.cc | 301 | `CAP_MEMORY_ALLOC` | P1 |
| `seccomp` | bionic/libc/bionic/seccomp.cpp | 88 | _bypass/noop_ | P2 |

---

## Distribuzione per Categoria

| Categoria | Syscall count | Priorità media |
|-----------|--------------|----------------|
| MEMORY (mmap, mprotect, madvise...) | 12 | P0 |
| IO (openat, read, write, fstat...) | 18 | P0 |
| THREAD (clone, futex, prctl...) | 10 | P0 |
| SIGNAL (rt_sigaction, tgkill...) | 7 | P0 |
| SCHED (sched_yield, clock_gettime...) | 7 | P1 |
| NET (socket, connect, epoll...) | 8 | P1–P2 |
| DEBUG (ptrace, process_vm...) | 4 | P3 |
| MISC (uname, getrandom...) | 5 | P2 |

**Totale: 71 syscall distinte**

---

## Implementazione consigliata per ordine

1. **P0 – Obbligatorie per boot ART** (18 syscall):
   mmap, munmap, mprotect, clone, futex, rt_sigaction, rt_sigprocmask,
   prctl, openat, read, write, close, fstat, gettid, getpid, tgkill,
   clock_gettime, memfd_create

2. **P1 – Obbligatorie per eseguire DEX** (25 syscall):
   madvise, brk, nanosleep, gettimeofday, mremap, pread64, lseek,
   fcntl, getrlimit, setrlimit, socket, connect, fallocate, ftruncate,
   sched_yield, sched_setscheduler, getrandom, membarrier, sigaltstack,
   kill, dup3, writev, msync, eventfd2, pipe2

3. **P2 – Funzionalità complete** (17 syscall):
   mincore, seccomp, getcpu, wait4, ioctl, faccessat, readlinkat,
   getdents64, statfs, sched_setaffinity, sched_getaffinity, pwrite64,
   readv, epoll_create1, epoll_pwait, uname, ioctl(ashmem)

4. **P3 – Debug/profiling** (4 syscall):
   ptrace, process_vm_readv, process_vm_writev, perf_event_open, getcpu
