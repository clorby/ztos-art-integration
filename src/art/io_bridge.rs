// I/O bridge: gestisce openat, read, write, close, fstat, lseek.
// Traduce le syscall POSIX in chiamate al ZTOS storage server.

use crate::art::syscall_wrapper::{EPERM, EBADF, EINVAL, EFAULT, ENOENT};
use crate::art::syscall_table::*;
use crate::kernel::ipc;
use crate::kernel::capability::{has_capability, CAP_APP_STORAGE, APP_STORAGE_BASE};

// ── Tabella file descriptor virtuali ─────────────────────────────────────
const MAX_FDS: usize = 256;
const INVALID_FD: i32 = -1;

#[derive(Copy, Clone)]
enum FdKind {
    Stdout,      // fd 1
    Stderr,      // fd 2
    StorageFile, // file nel ZTOS storage
    MemFd,       // memfd_create, per JIT
    PipeRead,
    PipeWrite,
}

#[derive(Copy, Clone)]
struct FdEntry {
    kind:     FdKind,
    offset:   u64,
    size:     u64,
    data_ptr: u64,   // puntatore ai dati in memoria (per file mappati)
    valid:    bool,
}

impl FdEntry {
    const fn empty() -> Self {
        Self { kind: FdKind::Stdout, offset: 0, size: 0, data_ptr: 0, valid: false }
    }
}

static mut FD_TABLE: [FdEntry; MAX_FDS] = [FdEntry::empty(); MAX_FDS];
static mut NEXT_FD: i32 = 10; // 0,1,2 riservati

// DEX "files" virtualizzati (contenuto hardcoded per il demo)
const DEX_MAGIC: &[u8] = b"dex\n035\0"; // magic DEX format

// ── Dispatcher ───────────────────────────────────────────────────────────

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let app_id = crate::art::syscall_wrapper::current_app_id();

    // Verifica capability storage
    let storage_ep = APP_STORAGE_BASE + app_id;
    if !has_capability(app_id, storage_ep, CAP_APP_STORAGE) {
        return EPERM;
    }

    match nr {
        SYS_OPENAT    => sys_openat(a0 as i32, a1, a2 as u32, a3 as u32),
        SYS_CLOSE     => sys_close(a0 as i32),
        SYS_READ      => sys_read(a0 as i32, a1, a2 as usize),
        SYS_WRITE     => sys_write(a0 as i32, a1, a2 as usize),
        SYS_PREAD64   => sys_pread64(a0 as i32, a1, a2 as usize, a3),
        SYS_PWRITE64  => sys_pwrite64(a0 as i32, a1, a2 as usize, a3),
        SYS_READV     => sys_readv(a0 as i32, a1, a2 as usize),
        SYS_WRITEV    => sys_writev(a0 as i32, a1, a2 as usize),
        SYS_FSTAT | SYS_NEWFSTATAT => sys_fstat(a0 as i32, a1),
        SYS_LSEEK     => sys_lseek(a0 as i32, a1, a2 as i32),
        SYS_FCNTL     => sys_fcntl(a0 as i32, a1 as i32, a2),
        SYS_IOCTL     => sys_ioctl(a0 as i32, a1, a2),
        SYS_DUP3      => sys_dup3(a0 as i32, a1 as i32, a2 as u32),
        SYS_PIPE2     => sys_pipe2(a0, a1 as u32),
        SYS_FSYNC | SYS_FDATASYNC => 0,
        SYS_FACCESSAT => sys_faccessat(a0 as i32, a1, a2 as u32),
        SYS_READLINKAT=> sys_readlinkat(a0 as i32, a1, a2, a3 as usize),
        SYS_GETDENTS64=> EBADF, // stub
        SYS_STATFS    => sys_statfs(a0, a1),
        SYS_FALLOCATE => 0,
        _             => EINVAL,
    }
}

fn sys_openat(dirfd: i32, path_ptr: u64, flags: u32, _mode: u32) -> i64 {
    let path = read_cstr(path_ptr);
    let fd = alloc_fd();
    if fd < 0 { return EBADF; }

    // Classi di file che ART può aprire:
    let entry = if path.starts_with(b"/dev/ashmem") || path.starts_with(b"/dev/") {
        FdEntry { kind: FdKind::MemFd, offset: 0, size: 0, data_ptr: 0, valid: true }
    } else if path.ends_with(b".apk") || path.ends_with(b".dex") || path.ends_with(b".odex") || path.ends_with(b".vdex") {
        // DEX file: alloca buffer con magic DEX
        let buf_ptr = alloc_buf(4096);
        unsafe {
            let buf = core::slice::from_raw_parts_mut(buf_ptr as *mut u8, 4096);
            buf[..DEX_MAGIC.len()].copy_from_slice(DEX_MAGIC);
            // Simula una classes.dex minimale
            buf[8] = 0x70; // header_size
            buf[32..36].copy_from_slice(&(buf.len() as u32).to_le_bytes()); // file_size
        }
        FdEntry { kind: FdKind::StorageFile, offset: 0, size: 4096, data_ptr: buf_ptr as u64, valid: true }
    } else if path.starts_with(b"/proc/") || path.starts_with(b"/sys/") {
        FdEntry { kind: FdKind::StorageFile, offset: 0, size: 0, data_ptr: 0, valid: true }
    } else if path.starts_with(b"/data/") {
        FdEntry { kind: FdKind::StorageFile, offset: 0, size: 0, data_ptr: 0, valid: true }
    } else {
        FdEntry { kind: FdKind::StorageFile, offset: 0, size: 0, data_ptr: 0, valid: true }
    };

    unsafe { FD_TABLE[fd as usize] = entry; }
    fd as i64
}

fn sys_close(fd: i32) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        if fd >= 3 { FD_TABLE[fd as usize].valid = false; }
    }
    0
}

fn sys_read(fd: i32, buf_ptr: u64, len: usize) -> i64 {
    if fd == 0 { return 0; } // stdin: EOF
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let entry = &mut FD_TABLE[fd as usize];
        if !entry.valid { return EBADF; }
        let avail = (entry.size - entry.offset.min(entry.size)) as usize;
        let n = len.min(avail);
        if n > 0 && entry.data_ptr != 0 {
            let src = (entry.data_ptr + entry.offset) as *const u8;
            let dst = buf_ptr as *mut u8;
            for i in 0..n { *dst.add(i) = *src.add(i); }
            entry.offset += n as u64;
        }
        n as i64
    }
}

fn sys_write(fd: i32, buf_ptr: u64, len: usize) -> i64 {
    match fd {
        1 | 2 => {
            // stdout/stderr: stampa via UART
            let app_id = crate::art::syscall_wrapper::current_app_id();
            let ok = has_capability(app_id,
                crate::kernel::capability::CONSOLE_ENDPOINT_ID,
                crate::kernel::capability::CAP_STDOUT_WRITE);
            if !ok { return EPERM; }
            let data = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
            for &b in data {
                let arr = [b];
                if let Ok(s) = core::str::from_utf8(&arr) {
                    crate::kernel::uart::print_str(s);
                }
            }
            len as i64
        }
        _ => {
            if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
            unsafe {
                let entry = &mut FD_TABLE[fd as usize];
                if !entry.valid { return EBADF; }
                if entry.data_ptr != 0 {
                    let dst = (entry.data_ptr + entry.offset) as *mut u8;
                    let src = buf_ptr as *const u8;
                    for i in 0..len {
                        unsafe { *dst.add(i) = *src.add(i); }
                    }
                    entry.offset += len as u64;
                    if entry.offset > entry.size { entry.size = entry.offset; }
                }
                len as i64
            }
        }
    }
}

fn sys_pread64(fd: i32, buf_ptr: u64, len: usize, offset: u64) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let entry = &FD_TABLE[fd as usize];
        if !entry.valid || entry.data_ptr == 0 { return 0; }
        let avail = (entry.size.saturating_sub(offset)) as usize;
        let n = len.min(avail);
        if n > 0 {
            let src = (entry.data_ptr + offset) as *const u8;
            let dst = buf_ptr as *mut u8;
            for i in 0..n { *dst.add(i) = *src.add(i); }
        }
        n as i64
    }
}

fn sys_pwrite64(fd: i32, buf_ptr: u64, len: usize, offset: u64) -> i64 {
    len as i64 // stub accettato
}

fn sys_readv(fd: i32, iov_ptr: u64, iovcnt: usize) -> i64 {
    // iovec { iov_base: *void, iov_len: size_t }
    let mut total = 0i64;
    for i in 0..iovcnt {
        unsafe {
            let iov = (iov_ptr as *const u64).add(i * 2);
            let base = *iov;
            let len  = *iov.add(1) as usize;
            let n = sys_read(fd, base, len);
            if n < 0 { return n; }
            total += n;
        }
    }
    total
}

fn sys_writev(fd: i32, iov_ptr: u64, iovcnt: usize) -> i64 {
    let mut total = 0i64;
    for i in 0..iovcnt {
        unsafe {
            let iov = (iov_ptr as *const u64).add(i * 2);
            let base = *iov;
            let len  = *iov.add(1) as usize;
            let n = sys_write(fd, base, len);
            if n < 0 { return n; }
            total += n;
        }
    }
    total
}

fn sys_fstat(fd: i32, stat_ptr: u64) -> i64 {
    if stat_ptr == 0 { return EFAULT; }
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let entry = &FD_TABLE[fd as usize];
        // struct stat (128 bytes su AArch64)
        let st = core::slice::from_raw_parts_mut(stat_ptr as *mut u64, 16);
        st[0] = 0x00010001;  // st_dev
        st[1] = (fd as u64) + 0x1000; // st_ino
        st[2] = 0o100644u64; // st_mode: regular file
        st[3] = 1;           // st_nlink
        st[4] = 10000;       // st_uid
        st[5] = 10000;       // st_gid
        st[6] = 0;           // st_rdev
        st[7] = entry.size;  // st_size
        st[8] = 4096;        // st_blksize
        st[9] = (entry.size + 511) / 512; // st_blocks
    }
    0
}

fn sys_lseek(fd: i32, offset: u64, whence: i32) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let entry = &mut FD_TABLE[fd as usize];
        if !entry.valid { return EBADF; }
        let new_off: u64 = match whence {
            0 => offset,                           // SEEK_SET
            1 => entry.offset + offset,            // SEEK_CUR
            2 => entry.size.saturating_add(offset),// SEEK_END (offset negativo non gestito)
            _ => return EINVAL,
        };
        entry.offset = new_off;
        new_off as i64
    }
}

fn sys_fcntl(fd: i32, cmd: i32, arg: u64) -> i64 {
    match cmd {
        0 => fd as i64,   // F_DUPFD
        1 => fd as i64,   // F_DUPFD_CLOEXEC
        2 => 0,           // F_GETFD
        3 => 0,           // F_SETFD (FD_CLOEXEC)
        4 => 0o2,         // F_GETFL: O_RDWR
        5 => 0,           // F_SETFL
        _ => 0,
    }
}

fn sys_ioctl(fd: i32, request: u64, arg: u64) -> i64 {
    // ASHMEM ioctls (usati da Android per memoria condivisa)
    const ASHMEM_SET_SIZE: u64   = 0x40087703;
    const ASHMEM_GET_SIZE: u64   = 0x00007704;
    const ASHMEM_SET_NAME: u64   = 0x41007701;
    match request {
        ASHMEM_SET_SIZE => 0,
        ASHMEM_GET_SIZE => 4 * 1024 * 1024, // 4MB default
        ASHMEM_SET_NAME => 0,
        _ => 0, // ignora altri ioctl
    }
}

fn sys_dup3(oldfd: i32, newfd: i32, flags: u32) -> i64 {
    if oldfd < 0 || oldfd as usize >= MAX_FDS { return EBADF; }
    if newfd < 0 || newfd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        FD_TABLE[newfd as usize] = FD_TABLE[oldfd as usize];
    }
    newfd as i64
}

fn sys_pipe2(fds_ptr: u64, flags: u32) -> i64 {
    let rfd = alloc_fd();
    let wfd = alloc_fd();
    if rfd < 0 || wfd < 0 { return EBADF; }
    unsafe {
        FD_TABLE[rfd as usize] = FdEntry { kind: FdKind::PipeRead, offset:0, size:0, data_ptr:0, valid:true };
        FD_TABLE[wfd as usize] = FdEntry { kind: FdKind::PipeWrite,offset:0, size:0, data_ptr:0, valid:true };
        let fds = fds_ptr as *mut i32;
        *fds       = rfd;
        *fds.add(1)= wfd;
    }
    0
}

fn sys_faccessat(_dirfd: i32, path_ptr: u64, _mode: u32) -> i64 {
    let path = read_cstr(path_ptr);
    // Tutte le path accettate per compatibilità
    if path.starts_with(b"/proc/") || path.starts_with(b"/sys/") || path.starts_with(b"/dev/") {
        return 0;
    }
    0 // Esiste per definizione nel sandbox ZTOS
}

fn sys_readlinkat(_dirfd: i32, path_ptr: u64, buf_ptr: u64, buf_size: usize) -> i64 {
    let path = read_cstr(path_ptr);
    let result: &[u8] = if path == b"/proc/self/exe" {
        b"/system/bin/app_process64"
    } else {
        return EINVAL;
    };
    let n = result.len().min(buf_size);
    unsafe {
        let dst = core::slice::from_raw_parts_mut(buf_ptr as *mut u8, n);
        dst.copy_from_slice(&result[..n]);
    }
    n as i64
}

fn sys_statfs(path_ptr: u64, buf_ptr: u64) -> i64 {
    if buf_ptr == 0 { return EFAULT; }
    unsafe {
        let st = core::slice::from_raw_parts_mut(buf_ptr as *mut u64, 10);
        st[0] = 0xEF53;         // EXT4 magic
        st[1] = 4096;           // block size
        st[2] = 1024*1024;      // total blocks
        st[3] = 512*1024;       // free blocks
        st[4] = 512*1024;       // available blocks
    }
    0
}

// ── Funzioni ausiliarie esposte ad altri moduli ───────────────────────────

pub fn read_fd(fd: i32, offset: u64, buf: &mut [u8]) -> i64 {
    sys_pread64(fd, buf.as_mut_ptr() as u64, buf.len(), offset)
}

pub fn ftruncate(fd: i32, length: u64) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        FD_TABLE[fd as usize].size = length;
    }
    0
}

// ── Utilità ───────────────────────────────────────────────────────────────

fn alloc_fd() -> i32 {
    unsafe {
        for i in (NEXT_FD as usize)..MAX_FDS {
            if !FD_TABLE[i].valid {
                if i > NEXT_FD as usize { /* bump */ }
                return i as i32;
            }
        }
        -1
    }
}

fn alloc_buf(size: usize) -> usize {
    crate::art::memory_bridge::ALLOC_BYTES(size)
}

fn read_cstr(ptr: u64) -> &'static [u8] {
    if ptr == 0 { return b""; }
    unsafe {
        let p = ptr as *const u8;
        let mut len = 0usize;
        while *p.add(len) != 0 && len < 4096 { len += 1; }
        core::slice::from_raw_parts(p, len)
    }
}
