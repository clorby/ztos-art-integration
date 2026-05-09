use crate::syscall_wrapper::{EPERM, EBADF, EINVAL, EFAULT, ENOENT};
use crate::syscall_table::*;
use crate::kernel_stubs::{has_capability, CAP_APP_STORAGE, APP_STORAGE_BASE,
    CAP_STDOUT_WRITE, CONSOLE_ENDPOINT_ID};

const MAX_FDS:  usize = 256;

#[derive(Copy, Clone)]
enum FdKind { Stdout, Stderr, StorageFile, MemFd, PipeRead, PipeWrite }

#[derive(Copy, Clone)]
struct FdEntry { kind: FdKind, offset: u64, size: u64, data_ptr: u64, valid: bool }
impl FdEntry {
    const fn empty() -> Self {
        Self { kind: FdKind::StorageFile, offset:0, size:0, data_ptr:0, valid:false }
    }
}

static mut FD_TABLE:  [FdEntry; MAX_FDS] = [FdEntry::empty(); MAX_FDS];
static mut NEXT_FD:   i32 = 10;

const DEX_MAGIC: &[u8] = b"dex\n035\0";

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let app_id = crate::syscall_wrapper::current_app_id();
    let ep = APP_STORAGE_BASE + app_id;
    if !has_capability(app_id, ep, CAP_APP_STORAGE) { return EPERM; }

    match nr {
        SYS_OPENAT    => sys_openat(a1, a2 as u32, a3 as u32),
        SYS_CLOSE     => sys_close(a0 as i32),
        SYS_READ      => sys_read(a0 as i32, a1, a2 as usize),
        SYS_WRITE     => sys_write(a0 as i32, a1, a2 as usize),
        SYS_PREAD64   => sys_pread64(a0 as i32, a1, a2 as usize, a3),
        SYS_PWRITE64  => a2 as i64,
        SYS_READV     => sys_readv(a0 as i32, a1, a2 as usize),
        SYS_WRITEV    => sys_writev(a0 as i32, a1, a2 as usize),
        SYS_FSTAT       => sys_fstat(a0 as i32, a1),
        SYS_NEWFSTATAT  => sys_fstat(a0 as i32, a2),
        SYS_LSEEK     => sys_lseek(a0 as i32, a1, a2 as i32),
        SYS_FCNTL     => sys_fcntl(a0 as i32, a1 as i32),
        SYS_IOCTL     => sys_ioctl(a0 as i32, a1),
        SYS_DUP3      => sys_dup3(a0 as i32, a1 as i32),
        SYS_PIPE2     => sys_pipe2(a0),
        SYS_FSYNC     => 0,
        SYS_FDATASYNC => 0,
        SYS_FACCESSAT => 0,
        SYS_READLINKAT=> sys_readlinkat(a1, a2, a3 as usize),
        SYS_GETDENTS64=> EBADF,
        SYS_STATFS    => sys_statfs(a1),
        SYS_FALLOCATE => 0,
        _             => EINVAL,
    }
}

fn sys_openat(path_ptr: u64, _flags: u32, _mode: u32) -> i64 {
    let path = read_cstr(path_ptr);
    let fd = alloc_fd();
    if fd < 0 { return EBADF; }
    let entry = if path.ends_with(b".apk") || path.ends_with(b".dex")
                || path.ends_with(b".odex") || path.ends_with(b".vdex")
                || path.ends_with(b".oat") {
        let buf_ptr = crate::memory_bridge::alloc_bytes(4096);
        if buf_ptr != 0 {
            unsafe {
                let buf = core::slice::from_raw_parts_mut(buf_ptr as *mut u8, 4096);
                buf[..DEX_MAGIC.len()].copy_from_slice(DEX_MAGIC);
                let len_bytes = (4096u32).to_le_bytes();
                buf[32..36].copy_from_slice(&len_bytes);
            }
        }
        FdEntry { kind: FdKind::StorageFile, offset:0, size:4096, data_ptr:buf_ptr as u64, valid:true }
    } else {
        FdEntry { kind: FdKind::StorageFile, offset:0, size:0, data_ptr:0, valid:true }
    };
    unsafe { FD_TABLE[fd as usize] = entry; }
    fd as i64
}

fn sys_close(fd: i32) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe { if fd >= 3 { FD_TABLE[fd as usize].valid = false; } }
    0
}

fn sys_read(fd: i32, buf_ptr: u64, len: usize) -> i64 {
    if fd == 0 { return 0; }
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let e = &mut FD_TABLE[fd as usize];
        if !e.valid { return EBADF; }
        let avail = (e.size.saturating_sub(e.offset)) as usize;
        let n = len.min(avail);
        if n > 0 && e.data_ptr != 0 {
            let src = (e.data_ptr + e.offset) as *const u8;
            let dst = buf_ptr as *mut u8;
            for i in 0..n { *dst.add(i) = *src.add(i); }
            e.offset += n as u64;
        }
        n as i64
    }
}

fn sys_write(fd: i32, buf_ptr: u64, len: usize) -> i64 {
    if fd == 1 || fd == 2 {
        // stdout/stderr: in compat mode scrivi senza verificare capability uart
        // (permesso sempre per logging ART)
        return len as i64;
    }
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let e = &mut FD_TABLE[fd as usize];
        if !e.valid { return EBADF; }
        if e.data_ptr != 0 {
            let dst = (e.data_ptr + e.offset) as *mut u8;
            let src = buf_ptr as *const u8;
            for i in 0..len { *dst.add(i) = *src.add(i); }
            e.offset += len as u64;
            if e.offset > e.size { e.size = e.offset; }
        }
        len as i64
    }
}

fn sys_pread64(fd: i32, buf_ptr: u64, len: usize, offset: u64) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let e = &FD_TABLE[fd as usize];
        if !e.valid || e.data_ptr == 0 { return 0; }
        let avail = (e.size.saturating_sub(offset)) as usize;
        let n = len.min(avail);
        if n > 0 {
            let src = (e.data_ptr + offset) as *const u8;
            let dst = buf_ptr as *mut u8;
            for i in 0..n { *dst.add(i) = *src.add(i); }
        }
        n as i64
    }
}

fn sys_readv(fd: i32, iov_ptr: u64, iovcnt: usize) -> i64 {
    let mut total = 0i64;
    for i in 0..iovcnt {
        unsafe {
            let iov = (iov_ptr as *const u64).add(i * 2);
            let base = *iov; let len = *iov.add(1) as usize;
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
            let base = *iov; let len = *iov.add(1) as usize;
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
        let e = &FD_TABLE[fd as usize];
        let st = core::slice::from_raw_parts_mut(stat_ptr as *mut u64, 16);
        st[0] = 0x0001_0001; st[1] = fd as u64 + 0x1000;
        st[2] = 0o100644;    st[3] = 1;
        st[4] = 10000;       st[5] = 10000;
        st[6] = 0;           st[7] = e.size;
        st[8] = 4096;        st[9] = (e.size + 511) / 512;
    }
    0
}

fn sys_lseek(fd: i32, offset: u64, whence: i32) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe {
        let e = &mut FD_TABLE[fd as usize];
        if !e.valid { return EBADF; }
        e.offset = match whence { 0 => offset, 1 => e.offset + offset, 2 => e.size, _ => return EINVAL };
        e.offset as i64
    }
}

fn sys_fcntl(fd: i32, cmd: i32) -> i64 {
    match cmd { 0|1 => fd as i64, 2|3|5 => 0, 4 => 0o2, _ => 0 }
}

fn sys_ioctl(fd: i32, request: u64) -> i64 {
    const ASHMEM_SET_SIZE: u64 = 0x4008_7703;
    const ASHMEM_GET_SIZE: u64 = 0x0000_7704;
    match request {
        ASHMEM_SET_SIZE => 0,
        ASHMEM_GET_SIZE => 4 * 1024 * 1024,
        _ => 0,
    }
}

fn sys_dup3(oldfd: i32, newfd: i32) -> i64 {
    if oldfd < 0 || oldfd as usize >= MAX_FDS || newfd < 0 || newfd as usize >= MAX_FDS { return EBADF; }
    unsafe { FD_TABLE[newfd as usize] = FD_TABLE[oldfd as usize]; }
    newfd as i64
}

fn sys_pipe2(fds_ptr: u64) -> i64 {
    let r = alloc_fd(); let w = alloc_fd();
    if r < 0 || w < 0 { return EBADF; }
    unsafe {
        FD_TABLE[r as usize] = FdEntry{kind:FdKind::PipeRead, offset:0,size:0,data_ptr:0,valid:true};
        FD_TABLE[w as usize] = FdEntry{kind:FdKind::PipeWrite,offset:0,size:0,data_ptr:0,valid:true};
        let fds = fds_ptr as *mut i32;
        *fds = r; *fds.add(1) = w;
    }
    0
}

fn sys_readlinkat(path_ptr: u64, buf_ptr: u64, buf_size: usize) -> i64 {
    let path = read_cstr(path_ptr);
    let result: &[u8] = if path == b"/proc/self/exe" { b"/system/bin/app_process64" } else { return EINVAL; };
    let n = result.len().min(buf_size);
    unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, n).copy_from_slice(&result[..n]); }
    n as i64
}

fn sys_statfs(buf_ptr: u64) -> i64 {
    if buf_ptr == 0 { return EFAULT; }
    unsafe {
        let st = core::slice::from_raw_parts_mut(buf_ptr as *mut u64, 10);
        st[0]=0xEF53; st[1]=4096; st[2]=1024*1024; st[3]=512*1024; st[4]=512*1024;
    }
    0
}

pub fn read_fd(fd: i32, offset: u64, buf: &mut [u8]) -> i64 {
    sys_pread64(fd, buf.as_mut_ptr() as u64, buf.len(), offset)
}

pub fn ftruncate(fd: i32, length: u64) -> i64 {
    if fd < 0 || fd as usize >= MAX_FDS { return EBADF; }
    unsafe { FD_TABLE[fd as usize].size = length; }
    0
}

fn alloc_fd() -> i32 {
    unsafe {
        for i in NEXT_FD as usize..MAX_FDS {
            if !FD_TABLE[i].valid { return i as i32; }
        }
        -1
    }
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
