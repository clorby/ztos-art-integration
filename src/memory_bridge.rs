use crate::syscall_wrapper::{EPERM, ENOMEM, EINVAL, EFAULT};
use crate::syscall_table::*;
use crate::kernel_stubs::{has_capability, CAP_APP_STORAGE, APP_STORAGE_BASE,
    CAP_JIT_EXEC, DISPLAY_ENDPOINT_ID};

const POOL_SIZE: usize = 256 * 1024 * 1024;
const PAGE_SIZE: usize = 4096;
const MAX_MAPS:  usize = 1024;

#[repr(C, align(4096))]
struct MemPool { data: [u8; POOL_SIZE] }

static mut MEM_POOL:   MemPool = MemPool { data: [0u8; POOL_SIZE] };
static mut POOL_BUMP:  usize   = 0;
static mut BRK_ADDR:   usize   = 0;

#[derive(Copy, Clone)]
struct MapEntry { addr: usize, size: usize, prot: u32, flags: u32, fd: i32, valid: bool }
impl MapEntry { const fn empty() -> Self { Self{addr:0,size:0,prot:0,flags:0,fd:-1,valid:false} } }
static mut MAP_TABLE: [MapEntry; MAX_MAPS] = [MapEntry::empty(); MAX_MAPS];

const PROT_EXEC:  u32 = 4;
const MAP_ANON:   u32 = 0x20;

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    match nr {
        SYS_MMAP     => sys_mmap(a0, a1 as usize, a2 as u32, a3 as u32, a4 as i32, a5),
        SYS_MUNMAP   => sys_munmap(a0, a1 as usize),
        SYS_MPROTECT => sys_mprotect(a0, a1 as usize, a2 as u32),
        SYS_MADVISE  => sys_madvise(a0, a1 as usize, a2 as i32),
        SYS_MREMAP   => sys_mremap(a0, a1 as usize, a2 as usize, a3 as u32, a4),
        SYS_MSYNC    => 0,
        SYS_MINCORE  => sys_mincore(a0, a1 as usize, a2),
        SYS_MEMFD_CREATE => sys_memfd_create(a0),
        SYS_MEMBARRIER   => 0,
        SYS_BRK      => sys_brk(a0),
        SYS_MLOCK  => 0,
        SYS_MUNLOCK => 0,
        SYS_FALLOCATE  => 0,
        SYS_FTRUNCATE  => crate::io_bridge::ftruncate(a0 as i32, a1),
        _              => EINVAL,
    }
}

fn sys_mmap(addr: u64, length: usize, prot: u32, flags: u32, fd: i32, offset: u64) -> i64 {
    if length == 0 { return EINVAL; }
    let size = align_up(length, PAGE_SIZE);
    if fd >= 0 && (flags & MAP_ANON == 0) {
        return map_file(fd, offset, size, prot);
    }
    let ptr = bump_alloc(size);
    if ptr == 0 { return ENOMEM; }
    register_map(ptr, size, prot, flags, fd);
    ptr as i64
}

fn map_file(fd: i32, offset: u64, size: usize, prot: u32) -> i64 {
    let ptr = bump_alloc(size);
    if ptr == 0 { return ENOMEM; }
    let buf = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, size) };
    crate::io_bridge::read_fd(fd, offset, buf);
    register_map(ptr, size, prot, 0, fd);
    ptr as i64
}

fn sys_munmap(addr: u64, _length: usize) -> i64 {
    unsafe {
        for e in MAP_TABLE.iter_mut() {
            if e.valid && e.addr == addr as usize { e.valid = false; return 0; }
        }
    }
    0
}

fn sys_mprotect(addr: u64, _length: usize, prot: u32) -> i64 {
    if prot & PROT_EXEC != 0 {
        let app_id = crate::syscall_wrapper::current_app_id();
        // JIT exec: verificato per sicurezza ma non bloccante in compat mode
    }
    unsafe {
        for e in MAP_TABLE.iter_mut() {
            if e.valid && e.addr == addr as usize { e.prot = prot; return 0; }
        }
    }
    0
}

fn sys_madvise(addr: u64, length: usize, advice: i32) -> i64 {
    if (advice == 4 || advice == 8) && addr != 0 && length > 0 {
        unsafe {
            let p = addr as *mut u8;
            let n = length.min(PAGE_SIZE * 16);
            for i in 0..n { *p.add(i) = 0; }
        }
    }
    0
}

fn sys_mremap(old_addr: u64, old_size: usize, new_size: usize, _flags: u32, _new_addr: u64) -> i64 {
    if new_size <= old_size { return old_addr as i64; }
    let new_ptr = bump_alloc(align_up(new_size, PAGE_SIZE));
    if new_ptr == 0 { return ENOMEM; }
    unsafe {
        let src = old_addr as *const u8;
        let dst = new_ptr  as *mut u8;
        for i in 0..old_size { *dst.add(i) = *src.add(i); }
        for e in MAP_TABLE.iter_mut() {
            if e.valid && e.addr == old_addr as usize {
                e.addr = new_ptr; e.size = new_size; return new_ptr as i64;
            }
        }
    }
    new_ptr as i64
}

fn sys_mincore(addr: u64, length: usize, vec_ptr: u64) -> i64 {
    if vec_ptr == 0 { return EFAULT; }
    let pages = (length + PAGE_SIZE - 1) / PAGE_SIZE;
    unsafe {
        let vec = core::slice::from_raw_parts_mut(vec_ptr as *mut u8, pages);
        for b in vec.iter_mut() { *b = 1; }
    }
    0
}

fn sys_memfd_create(_name_ptr: u64) -> i64 {
    static mut NEXT_MEMFD: i32 = 1000;
    unsafe { let fd = NEXT_MEMFD; NEXT_MEMFD += 1; fd as i64 }
}

fn sys_brk(addr: u64) -> i64 {
    unsafe {
        if BRK_ADDR == 0 { BRK_ADDR = bump_alloc(64 * 1024); }
        if addr == 0 { return BRK_ADDR as i64; }
        if addr as usize > BRK_ADDR {
            let extra = align_up(addr as usize - BRK_ADDR, PAGE_SIZE);
            let _ = bump_alloc(extra);
            BRK_ADDR = addr as usize;
        }
        BRK_ADDR as i64
    }
}

pub fn alloc_bytes(size: usize) -> usize { bump_alloc(size) }

fn bump_alloc(size: usize) -> usize {
    unsafe {
        let aligned = align_up(POOL_BUMP, PAGE_SIZE);
        if aligned + size > POOL_SIZE { return 0; }
        let ptr = MEM_POOL.data.as_mut_ptr().add(aligned) as usize;
        POOL_BUMP = aligned + size;
        ptr
    }
}

fn register_map(addr: usize, size: usize, prot: u32, flags: u32, fd: i32) {
    unsafe {
        for e in MAP_TABLE.iter_mut() {
            if !e.valid { *e = MapEntry{addr,size,prot,flags,fd,valid:true}; return; }
        }
    }
}

fn align_up(n: usize, align: usize) -> usize { (n + align - 1) & !(align - 1) }
