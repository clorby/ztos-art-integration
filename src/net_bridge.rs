use crate::syscall_wrapper::{EPERM, EINVAL, EFAULT, EBADF};
use crate::syscall_table::*;
use crate::kernel_stubs::{has_capability, CAP_NET_SEND, CAP_NET_RECV,
    NETWORK_ENDPOINT_ID, vpn_kill_switch_active, virtio_send_frame};

const MAX_SOCKS: usize = 64;

#[derive(Copy, Clone)]
struct SockEntry { fd: i32, domain: i32, stype: i32, valid: bool }
impl SockEntry { const fn empty() -> Self { Self{fd:0,domain:0,stype:0,valid:false} } }

static mut SOCK_TABLE: [SockEntry; MAX_SOCKS] = [SockEntry::empty(); MAX_SOCKS];
static mut NEXT_SOCK:  i32 = 700;

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let app_id    = crate::syscall_wrapper::current_app_id();
    let domain_id = crate::syscall_wrapper::current_domain_id();

    if vpn_kill_switch_active(domain_id) { return EPERM; }

    match nr {
        SYS_SOCKET       => sys_socket(a0 as i32, a1 as i32, a2 as i32),
        SYS_SOCKETPAIR   => sys_socketpair(a0 as i32, a1 as i32, a2 as i32, a3),
        SYS_CONNECT      => 0, // stub: connessione accettata
        SYS_BIND         => 0,
        SYS_LISTEN       => 0,
        SYS_ACCEPT       => sys_socket(2, 1, 0),
        SYS_SENDTO       => sys_sendto(a0 as i32, a1, a2 as usize),
        SYS_RECVFROM     => 0,
        SYS_SENDMSG      => sys_sendmsg(a0 as i32, a1),
        SYS_RECVMSG      => 0,
        SYS_SETSOCKOPT   => 0,
        SYS_GETSOCKOPT   => sys_getsockopt(a0 as i32, a2 as i32, a3, a4),
        SYS_GETSOCKNAME  => sys_getsockname(a0 as i32, a1, a2),
        SYS_GETPEERNAME  => sys_getsockname(a0 as i32, a1, a2),
        SYS_EPOLL_CREATE1 => { static mut N: i32 = 800; unsafe { let f=N; N+=1; f as i64 } },
        SYS_EPOLL_CTL    => 0,
        SYS_EPOLL_PWAIT  => { if a3 as i32 == 0 { 0 } else {
            for _ in 0..1000u32 { unsafe { core::arch::asm!("nop", options(nomem, nostack)); } }
            0
        }},
        _                => EINVAL,
    }
}

fn sys_socket(domain: i32, stype: i32, _proto: i32) -> i64 {
    let fd = unsafe { let f = NEXT_SOCK; NEXT_SOCK += 1; f };
    unsafe {
        for s in SOCK_TABLE.iter_mut() {
            if !s.valid { *s = SockEntry{fd, domain, stype, valid:true}; break; }
        }
    }
    fd as i64
}

fn sys_socketpair(domain: i32, stype: i32, proto: i32, sv: u64) -> i64 {
    let f1 = sys_socket(domain, stype, proto) as i32;
    let f2 = sys_socket(domain, stype, proto) as i32;
    if sv != 0 { unsafe { let p = sv as *mut i32; *p = f1; *p.add(1) = f2; } }
    0
}

fn sys_sendto(fd: i32, buf: u64, len: usize) -> i64 {
    if buf == 0 { return EINVAL; }
    let data = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
    if virtio_send_frame(data) { len as i64 } else { EPERM }
}

fn sys_sendmsg(fd: i32, msg: u64) -> i64 {
    if msg == 0 { return EINVAL; }
    unsafe {
        let hdr = msg as *const u64;
        let iov_ptr = *hdr.add(2);
        let iov_cnt = *hdr.add(3) as usize;
        let mut total = 0i64;
        for i in 0..iov_cnt {
            let iov = (iov_ptr as *const u64).add(i * 2);
            let n = sys_sendto(fd, *iov, *iov.add(1) as usize);
            if n < 0 { return n; }
            total += n;
        }
        total
    }
}

fn sys_getsockopt(fd: i32, optname: i32, optval: u64, optlen: u64) -> i64 {
    if optval != 0 && optname == 4 { unsafe { *(optval as *mut i32) = 0; } }
    0
}

fn sys_getsockname(fd: i32, addr: u64, addrlen: u64) -> i64 {
    if addr == 0 { return EINVAL; }
    unsafe {
        let sa = addr as *mut u16; *sa = 2; *sa.add(1) = 0;
        let sa32 = addr as *mut u32; *sa32.add(1) = 0;
        if addrlen != 0 { *(addrlen as *mut u32) = 16; }
    }
    0
}
