// Memory bridge: gestisce mmap, munmap, mprotect, madvise, mremap, brk.
// In ZTOS non c'è un kernel Linux: alloca da un pool statico gestito da ZTOS.

use crate::art::syscall_wrapper::{EPERM, ENOMEM, EINVAL, EFAULT};
use crate::art::syscall_table::*;

// ── Pool di memoria per le app ART ───────────────────────────────────────
// 256MB riservati per tutte le app ART (heap, code cache, DEX mappings).
// In produzione questo sarebbe gestito dal memory server ZTOS.

const POOL_SIZE:  usize = 256 * 1024 * 1024; // 256 MB
const PAGE_SIZE:  usize = 4096;
const MAX_MAPS:   usize = 1024;

#[repr(C, align(4096))]
struct MemPool {
    data: [u8; POOL_SIZE],
}

static mut MEM_POOL: MemPool = MemPool { data: [0u8; POOL_SIZE] };
static mut POOL_BUMP: usize = 0; // allocatore bump semplice

// Mappa allocata: (addr, size, prot, flags, valid)
#[derive(Copy, Clone)]
struct MapEntry {
    addr:  usize,
    size:  usize,
    prot:  u32,
    flags: u32,
    fd:    i32,
    valid: bool,
}

impl MapEntry {
    const fn empty() -> Self {
        Self { addr: 0, size: 0, prot: 0, flags: 0, fd: -1, valid: false }
    }
}

static mut MAP_TABLE: [MapEntry; MAX_MAPS] = [MapEntry::empty(); MAX_MAPS];
static mut BRK_ADDR: usize = 0;

// ── Costanti mmap ─────────────────────────────────────────────────────────
const PROT_NONE:  u32 = 0;
const PROT_READ:  u32 = 1;
const PROT_WRITE: u32 = 2;
const PROT_EXEC:  u32 = 4;
const MAP_SHARED:   u32 = 0x01;
const MAP_PRIVATE:  u32 = 0x02;
const MAP_FIXED:    u32 = 0x10;
const MAP_ANON:     u32 = 0x20;
const MAP_NORESERVE:u32 = 0x4000;
const MAP_FAILED: i64 = -1;

// ── Dispatcher ───────────────────────────────────────────────────────────

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    match nr {
        SYS_MMAP    => sys_mmap(a0, a1 as usize, a2 as u32, a3 as u32, a4 as i32, a5),
        SYS_MUNMAP  => sys_munmap(a0, a1 as usize),
        SYS_MPROTECT=> sys_mprotect(a0, a1 as usize, a2 as u32),
        SYS_MADVISE => sys_madvise(a0, a1 as usize, a2 as i32),
        SYS_MREMAP  => sys_mremap(a0, a1 as usize, a2 as usize, a3 as u32, a4),
        SYS_MSYNC   => 0, // no-op: i dati sono già in memoria
        SYS_MINCORE => sys_mincore(a0, a1 as usize, a2),
        SYS_MEMFD_CREATE => sys_memfd_create(a0, a1 as u32),
        SYS_MEMBARRIER   => 0,
        SYS_BRK     => sys_brk(a0),
        SYS_MLOCK   => 0,
        SYS_MUNLOCK => 0,
        SYS_FALLOCATE  => 0,
        SYS_FTRUNCATE  => sys_ftruncate(a0 as i32, a1),
        _           => EINVAL,
    }
}

fn sys_mmap(addr: u64, length: usize, prot: u32, flags: u32, fd: i32, offset: u64) -> i64 {
    if length == 0 { return EINVAL; }
    let size = align_up(length, PAGE_SIZE);

    // Mappa file: va al bridge I/O
    if fd >= 0 && (flags & MAP_ANON == 0) {
        return map_file(fd, offset, size, prot);
    }

    // Allocazione anonima dal pool
    let ptr = bump_alloc(size);
    if ptr == 0 { return ENOMEM; }

    // PROT_NONE = riservato (JIT code cache iniziale)
    if prot == PROT_NONE {
        // Pagine riservate, non accessibili ancora
        // In ZTOS non c'è protezione MMU reale, ma tracciamo lo stato
    }

    register_map(ptr, size, prot, flags, fd);
    ptr as i64
}

fn map_file(fd: i32, offset: u64, size: usize, prot: u32) -> i64 {
    // DEX file mapping: legge fd dallo storage server e copia nel pool
    let ptr = bump_alloc(size);
    if ptr == 0 { return ENOMEM; }

    // Legge il contenuto del file tramite io_bridge
    let buf = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, size) };
    let n = crate::art::io_bridge::read_fd(fd, offset, buf);
    if n < 0 { return n; }

    register_map(ptr, size, prot, MAP_SHARED, fd);
    ptr as i64
}

fn sys_munmap(addr: u64, length: usize) -> i64 {
    // Rimuovi dalla mappa (il pool bump non libera davvero, ma tracciamo)
    unsafe {
        for entry in MAP_TABLE.iter_mut() {
            if entry.valid && entry.addr == addr as usize {
                entry.valid = false;
                return 0;
            }
        }
    }
    0 // successo anche se non trovata (come Linux)
}

fn sys_mprotect(addr: u64, length: usize, prot: u32) -> i64 {
    // In ZTOS non abbiamo MMU granulare, ma verifichiamo le capability
    // JIT: PROT_EXEC richiede CAP_JIT_EXEC
    if prot & PROT_EXEC != 0 {
        let app_id = crate::art::syscall_wrapper::current_app_id();
        if !crate::kernel::capability::has_capability(app_id,
            crate::kernel::capability::DISPLAY_ENDPOINT_ID,
            crate::kernel::capability::CAP_JIT_EXEC) {
            // In V0.7 lo permettiamo comunque per compatibilità
        }
    }
    // Aggiorna la protezione nella mappa
    unsafe {
        for entry in MAP_TABLE.iter_mut() {
            if entry.valid && entry.addr == addr as usize {
                entry.prot = prot;
                return 0;
            }
        }
    }
    0
}

fn sys_madvise(addr: u64, length: usize, advice: i32) -> i64 {
    // GC usa MADV_DONTNEED per rilasciare pagine
    // In ZTOS non abbiamo swap, ma azzeraimo la memoria se richiesto
    if advice == 4 || advice == 8 { // MADV_DONTNEED=4, MADV_FREE=8
        if addr != 0 && length > 0 {
            unsafe {
                let ptr = addr as *mut u8;
                // Zero-fill le pagine "rilasciate" (simula il comportamento del kernel)
                for i in 0..length.min(PAGE_SIZE * 16) { // limite sicurezza
                    *ptr.add(i) = 0;
                }
            }
        }
    }
    0
}

fn sys_mremap(old_addr: u64, old_size: usize, new_size: usize, flags: u32, new_addr: u64) -> i64 {
    if new_size <= old_size { return old_addr as i64; }
    // Alloca nuovo blocco più grande
    let new_ptr = bump_alloc(align_up(new_size, PAGE_SIZE));
    if new_ptr == 0 { return ENOMEM; }
    // Copia dati vecchi
    unsafe {
        let src = old_addr as *const u8;
        let dst = new_ptr as *mut u8;
        for i in 0..old_size { *dst.add(i) = *src.add(i); }
    }
    // Aggiorna mappa
    unsafe {
        for entry in MAP_TABLE.iter_mut() {
            if entry.valid && entry.addr == old_addr as usize {
                entry.addr = new_ptr;
                entry.size = new_size;
                return new_ptr as i64;
            }
        }
    }
    new_ptr as i64
}

fn sys_mincore(addr: u64, length: usize, vec_ptr: u64) -> i64 {
    // In ZTOS tutta la memoria è "residente" (nessun swap)
    if vec_ptr == 0 { return EFAULT; }
    let pages = (length + PAGE_SIZE - 1) / PAGE_SIZE;
    unsafe {
        let vec = core::slice::from_raw_parts_mut(vec_ptr as *mut u8, pages);
        for b in vec.iter_mut() { *b = 1; } // tutte le pagine residenti
    }
    0
}

fn sys_memfd_create(name_ptr: u64, flags: u32) -> i64 {
    // Ritorna un fd virtuale per il JIT code cache
    // I fd virtuali per memfd sono offset 1000+
    static mut NEXT_MEMFD: i32 = 1000;
    unsafe {
        let fd = NEXT_MEMFD;
        NEXT_MEMFD += 1;
        fd as i64
    }
}

fn sys_ftruncate(fd: i32, length: u64) -> i64 {
    // Per memfd: alloca spazio nel pool
    if fd >= 1000 {
        let _ = bump_alloc(align_up(length as usize, PAGE_SIZE));
        return 0;
    }
    crate::art::io_bridge::ftruncate(fd, length)
}

fn sys_brk(addr: u64) -> i64 {
    unsafe {
        if BRK_ADDR == 0 {
            BRK_ADDR = bump_alloc(64 * 1024); // 64KB heap iniziale
        }
        if addr == 0 { return BRK_ADDR as i64; }
        if addr > BRK_ADDR {
            let extra = align_up(addr as usize - BRK_ADDR, PAGE_SIZE);
            let new = bump_alloc(extra);
            if new == 0 { return BRK_ADDR as i64; }
            BRK_ADDR = addr as usize;
        }
        BRK_ADDR as i64
    }
}

// ── Utilità interne ───────────────────────────────────────────────────────

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
        for entry in MAP_TABLE.iter_mut() {
            if !entry.valid {
                *entry = MapEntry { addr, size, prot, flags, fd, valid: true };
                return;
            }
        }
    }
}

fn align_up(n: usize, align: usize) -> usize {
    (n + align - 1) & !(align - 1)
}
