/*
 * ZTOS Syscall Compatibility Header
 * Include questo file nei sorgenti ART/bionic patchati per ottenere
 * le dichiarazioni di tutte le funzioni ZTOS che rimpiazzano le syscall.
 *
 * Il preprocessore #ZTOS_COMPAT deve essere definito durante la compilazione.
 */

#pragma once

#ifdef ZTOS_COMPAT

#include <stdint.h>
#include <stddef.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Context management ──────────────────────────────────────────────────── */

/** Imposta il contesto app corrente per le verifiche capability. */
void ztos_set_app_context(uint32_t app_id, uint32_t domain_id);
uint32_t ztos_get_current_app_id(void);
uint32_t ztos_get_current_domain_id(void);

/* ── Syscall wrapper principale ─────────────────────────────────────────── */

/**
 * Entry point per tutte le syscall.
 * Chiamato da bionic patchato invece di `svc #0`.
 * @param nr  Numero syscall Linux AArch64
 * @param a0..a5  Argomenti syscall
 * @return Risultato (positivo=successo, negativo=-errno)
 */
long ztos_syscall_wrapper(long nr, long a0, long a1, long a2, long a3, long a4, long a5);

/* ── Memory ──────────────────────────────────────────────────────────────── */

void* ztos_mmap(void* addr, size_t length, int prot, int flags, int fd, off_t offset);
int   ztos_munmap(void* addr, size_t length);
int   ztos_mprotect(void* addr, size_t length, int prot);
int   ztos_madvise(void* addr, size_t length, int advice);
void* ztos_mremap(void* old_address, size_t old_size, size_t new_size, int flags, ...);
int   ztos_msync(void* addr, size_t length, int flags);
int   ztos_mlock(const void* addr, size_t length);
int   ztos_munlock(const void* addr, size_t length);
int   ztos_mincore(void* addr, size_t length, unsigned char* vec);
int   ztos_memfd_create(const char* name, unsigned int flags);
int   ztos_membarrier(int cmd, unsigned int flags, int cpu_id);
void* ztos_brk(void* addr);
int   ztos_mprotect_exec(void* addr, size_t length);  /* richiede CAP_JIT_EXEC */

/* ── I/O ──────────────────────────────────────────────────────────────────── */

int   ztos_openat_compat(const char* path, int flags, ...);
int   ztos_close(int fd);
long  ztos_read(int fd, void* buf, size_t count);
long  ztos_write(int fd, const void* buf, size_t count);
long  ztos_pread64(int fd, void* buf, size_t count, off_t offset);
long  ztos_pwrite64(int fd, const void* buf, size_t count, off_t offset);
long  ztos_readv(int fd, const struct iovec* iov, int iovcnt);
long  ztos_writev(int fd, const struct iovec* iov, int iovcnt);
int   ztos_fstat(int fd, struct stat* statbuf);
int   ztos_stat(const char* path, struct stat* statbuf);
off_t ztos_lseek(int fd, off_t offset, int whence);
int   ztos_fcntl(int fd, int cmd, ...);
int   ztos_ioctl(int fd, unsigned long request, ...);
int   ztos_dup3(int oldfd, int newfd, int flags);
int   ztos_pipe2(int pipefd[2], int flags);
int   ztos_fsync(int fd);
int   ztos_fallocate(int fd, int mode, off_t offset, off_t length);
int   ztos_ftruncate(int fd, off_t length);
int   ztos_faccessat(int dirfd, const char* path, int mode, int flags);
long  ztos_readlinkat(int dirfd, const char* path, char* buf, size_t bufsize);
long  ztos_getdents64(int fd, void* dirp, size_t count);
int   ztos_statfs(const char* path, struct statfs* buf);

/* ── Thread ──────────────────────────────────────────────────────────────── */

long  ztos_clone(unsigned long flags, void* stack, int* parent_tid,
                 unsigned long tls, int* child_tid);
int   ztos_futex(int* uaddr, int futex_op, int val,
                 const struct timespec* timeout, int* uaddr2, int val3);
int   ztos_eventfd2(unsigned int initval, int flags);
void  ztos_exit(int status) __attribute__((noreturn));
void  ztos_exit_group(int status) __attribute__((noreturn));
pid_t ztos_wait4(pid_t pid, int* wstatus, int options, struct rusage* rusage);

/* ── Signal ──────────────────────────────────────────────────────────────── */

int  ztos_rt_sigaction(int signum, const struct sigaction* act,
                       struct sigaction* oldact, size_t sigsetsize);
int  ztos_rt_sigprocmask(int how, const sigset_t* set,
                          sigset_t* oldset, size_t sigsetsize);
int  ztos_sigaltstack(const stack_t* ss, stack_t* old_ss);
int  ztos_tgkill(int tgid, int tid, int sig);
int  ztos_tkill(int tid, int sig);
int  ztos_kill(pid_t pid, int sig);

/* ── Sched / timing ──────────────────────────────────────────────────────── */

int  ztos_clock_gettime(clockid_t clockid, struct timespec* tp);
int  ztos_clock_nanosleep(clockid_t clockid, int flags,
                           const struct timespec* request, struct timespec* remain);
int  ztos_nanosleep(const struct timespec* req, struct timespec* rem);
int  ztos_gettimeofday(struct timeval* tv, struct timezone* tz);
int  ztos_sched_yield(void);
int  ztos_sched_setscheduler(pid_t pid, int policy, const struct sched_param* param);
int  ztos_sched_setaffinity(pid_t pid, size_t cpusetsize, const cpu_set_t* mask);
int  ztos_sched_getaffinity(pid_t pid, size_t cpusetsize, cpu_set_t* mask);
int  ztos_prctl(int option, unsigned long arg2, unsigned long arg3,
                unsigned long arg4, unsigned long arg5);

/* ── Proc info ───────────────────────────────────────────────────────────── */

pid_t  ztos_getpid(void);
pid_t  ztos_gettid(void);
pid_t  ztos_getppid(void);
uid_t  ztos_getuid(void);
uid_t  ztos_geteuid(void);
gid_t  ztos_getgid(void);
gid_t  ztos_getegid(void);
int    ztos_getrlimit(int resource, struct rlimit* rlim);
int    ztos_setrlimit(int resource, const struct rlimit* rlim);
int    ztos_uname(struct utsname* buf);

/* ── Networking ──────────────────────────────────────────────────────────── */

int   ztos_socket(int domain, int type, int protocol);
int   ztos_socketpair(int domain, int type, int protocol, int sv[2]);
int   ztos_connect(int sockfd, const struct sockaddr* addr, socklen_t addrlen);
int   ztos_bind(int sockfd, const struct sockaddr* addr, socklen_t addrlen);
int   ztos_listen(int sockfd, int backlog);
int   ztos_accept(int sockfd, struct sockaddr* addr, socklen_t* addrlen);
long  ztos_send(int sockfd, const void* buf, size_t len, int flags);
long  ztos_recv(int sockfd, void* buf, size_t len, int flags);
long  ztos_sendto(int sockfd, const void* buf, size_t len, int flags,
                  const struct sockaddr* dest_addr, socklen_t addrlen);
long  ztos_recvfrom(int sockfd, void* buf, size_t len, int flags,
                    struct sockaddr* src_addr, socklen_t* addrlen);
int   ztos_setsockopt(int sockfd, int level, int optname,
                      const void* optval, socklen_t optlen);
int   ztos_getsockopt(int sockfd, int level, int optname,
                      void* optval, socklen_t* optlen);
int   ztos_getsockname(int sockfd, struct sockaddr* addr, socklen_t* addrlen);
int   ztos_epoll_create1(int flags);
int   ztos_epoll_ctl(int epfd, int op, int fd, struct epoll_event* event);
int   ztos_epoll_pwait(int epfd, struct epoll_event* events, int maxevents,
                       int timeout, const sigset_t* sigmask, size_t sigsetsize);

/* ── Misc ────────────────────────────────────────────────────────────────── */

long  ztos_getrandom(void* buf, size_t buflen, unsigned int flags);
int   ztos_seccomp(unsigned int operation, unsigned int flags, void* args); /* noop */
int   ztos_getaddrinfo_stub(const char* node, const char* service,
                             const struct addrinfo* hints, struct addrinfo** res);

#ifdef __cplusplus
}  // extern "C"
#endif

/* ── Macro di sostituzione ───────────────────────────────────────────────── */
/*
 * Queste macro rimpiazzano le funzioni POSIX standard con le versioni ZTOS.
 * Vanno incluse DOPO gli header di sistema per evitare conflitti.
 * Usare: #include <ztos/syscall_compat.h> come ULTIMO include.
 */

#define mmap(addr, len, prot, flags, fd, off) \
    ztos_mmap((addr), (len), (prot), (flags), (fd), (off))
#define munmap(addr, len)           ztos_munmap((addr), (len))
#define mprotect(addr, len, prot)   ztos_mprotect((addr), (len), (prot))
#define madvise(addr, len, adv)     ztos_madvise((addr), (len), (adv))
#define mremap(...)                 ztos_mremap(__VA_ARGS__)
#define memfd_create(name, flags)   ztos_memfd_create((name), (flags))

#define read(fd, buf, count)        ztos_read((fd), (buf), (count))
#define write(fd, buf, count)       ztos_write((fd), (buf), (count))
#define close(fd)                   ztos_close(fd)
#define fstat(fd, st)               ztos_fstat((fd), (st))
#define lseek(fd, off, w)           ztos_lseek((fd), (off), (w))

#define clock_gettime(clk, tp)      ztos_clock_gettime((clk), (tp))
#define nanosleep(req, rem)         ztos_nanosleep((req), (rem))
#define gettimeofday(tv, tz)        ztos_gettimeofday((tv), (tz))

#define getpid()    ztos_getpid()
#define gettid()    ztos_gettid()
#define getuid()    ztos_getuid()
#define getgid()    ztos_getgid()
#define prctl(...)  ztos_prctl(__VA_ARGS__)

#define socket(d, t, p)             ztos_socket((d), (t), (p))
#define connect(fd, addr, len)      ztos_connect((fd), (addr), (len))
#define send(fd, buf, len, f)       ztos_send((fd), (buf), (len), (f))
#define recv(fd, buf, len, f)       ztos_recv((fd), (buf), (len), (f))

#endif  /* ZTOS_COMPAT */
