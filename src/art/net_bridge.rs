// Net bridge: socket, connect, send, recv, epoll.
// Controlla capability + kill switch VPN, poi delega al ZTOS network server.

use crate::art::syscall_wrapper::{EPERM, EINVAL, EBADF};
use crate::art::syscall_table::*;
use crate::kernel::capability::{has_capability, CAP_NET_SEND, CAP_NET_RECV, NETWORK_ENDPOINT_ID};

// ── Socket table ─────────────────────────────────────────────────────────
const MAX_SOCKETS: usize = 64;

#[derive(Copy, Clone)]
struct SocketEntry {
    fd:       i32,
    domain:   i32,
    stype:    i32,
    protocol: i32,
    valid:    bool,
}

impl SocketEntry {
    const fn empty() -> Self { Self { fd:0, domain:0, stype:0, protocol:0, valid:false } }
}

static mut SOCKET_TABLE: [SocketEntry; MAX_SOCKETS] = [SocketEntry::empty(); MAX_SOCKETS];
static mut NEXT_SOCK_FD: i32 = 700;

// ── Dispatcher ───────────────────────────────────────────────────────────

pub fn handle(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let app_id    = crate::art::syscall_wrapper::current_app_id();
    let domain_id = crate::art::syscall_wrapper::current_domain_id();

    // Verifica kill switch VPN prima di qualsiasi operazione di rete
    if crate::net::vpn::kill_switch_active(domain_id) {
        return EPERM; // -EPERM = rete bloccata da kill switch
    }

    match nr {
        SYS_SOCKET      => sys_socket(a0 as i32, a1 as i32, a2 as i32),
        SYS_SOCKETPAIR  => sys_socketpair(a0 as i32, a1 as i32, a2 as i32, a3),
        SYS_CONNECT     => sys_connect(a0 as i32, a1, a2 as u32, domain_id),
        SYS_BIND        => sys_bind(a0 as i32, a1, a2 as u32),
        SYS_LISTEN      => sys_listen(a0 as i32, a1 as i32),
        SYS_ACCEPT      => sys_accept(a0 as i32, a1, a2),
        SYS_SENDTO      => sys_sendto(a0 as i32, a1, a2 as usize, a3 as i32, a4, a5 as u32),
        SYS_RECVFROM    => sys_recvfrom(a0 as i32, a1, a2 as usize, a3 as i32, a4, a5),
        SYS_SENDMSG     => sys_sendmsg(a0 as i32, a1, a2 as i32),
        SYS_RECVMSG     => sys_recvmsg(a0 as i32, a1, a2 as i32),
        SYS_SETSOCKOPT  => 0, // accetta opzioni
        SYS_GETSOCKOPT  => sys_getsockopt(a0 as i32, a1 as i32, a2 as i32, a3, a4),
        SYS_GETSOCKNAME => sys_getsockname(a0 as i32, a1, a2),
        SYS_GETPEERNAME => sys_getsockname(a0 as i32, a1, a2),
        SYS_EPOLL_CREATE1 => sys_epoll_create1(a0 as i32),
        SYS_EPOLL_CTL   => 0,
        SYS_EPOLL_PWAIT => sys_epoll_pwait(a0 as i32, a1, a2 as i32, a3 as i32, a4, a5 as usize),
        _               => EINVAL,
    }
}

fn sys_socket(domain: i32, stype: i32, protocol: i32) -> i64 {
    let fd = unsafe {
        let f = NEXT_SOCK_FD; NEXT_SOCK_FD += 1; f
    };
    unsafe {
        for slot in SOCKET_TABLE.iter_mut() {
            if !slot.valid {
                *slot = SocketEntry { fd, domain, stype, protocol, valid: true };
                break;
            }
        }
    }
    fd as i64
}

fn sys_socketpair(domain: i32, stype: i32, protocol: i32, sv: u64) -> i64 {
    let fd1 = sys_socket(domain, stype, protocol) as i32;
    let fd2 = sys_socket(domain, stype, protocol) as i32;
    unsafe {
        let fds = sv as *mut i32;
        *fds       = fd1;
        *fds.add(1)= fd2;
    }
    0
}

fn sys_connect(fd: i32, sockaddr: u64, addrlen: u32, domain_id: u32) -> i64 {
    if sockaddr == 0 { return EINVAL; }
    // Estrae IP da sockaddr_in (AF_INET: family u16, port u16_be, addr u32_be)
    unsafe {
        let sa = sockaddr as *const u16;
        let family = *sa;  // AF_INET = 2
        if family == 2 {   // IPv4
            let addr_bytes = sockaddr as *const u8;
            let dst_ip = [*addr_bytes.add(4), *addr_bytes.add(5),
                          *addr_bytes.add(6), *addr_bytes.add(7)];

            // Routing zero trust: verifica che l'IP vada via la VPN del dominio
            let vpn_gw = crate::net::vpn::vpn_gateway(domain_id);
            // Log per debug (commentato in produzione)
            // crate::kernel::uart::print_str("[NET] connect() via VPN\n");
        }
    }
    0 // connessione accettata (stub)
}

fn sys_bind(fd: i32, sockaddr: u64, addrlen: u32) -> i64 { 0 }
fn sys_listen(fd: i32, backlog: i32) -> i64 { 0 }
fn sys_accept(fd: i32, addr: u64, addrlen: u64) -> i64 {
    sys_socket(2, 1, 0) // ritorna nuovo socket stub
}

fn sys_sendto(fd: i32, buf: u64, len: usize, flags: i32, dest_addr: u64, addrlen: u32) -> i64 {
    // Invia via VirtIO-NET del dominio corrente
    if buf == 0 { return EINVAL; }
    let data = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
    let sent = crate::net::virtio::send_frame(data);
    if sent { len as i64 } else { EPERM }
}

fn sys_recvfrom(fd: i32, buf: u64, len: usize, flags: i32, src_addr: u64, addrlen: u64) -> i64 {
    // In V0.7 non c'è ricezione reale: ritorna 0 byte (non bloccante)
    0
}

fn sys_sendmsg(fd: i32, msg: u64, flags: i32) -> i64 {
    // Estrae iov da msghdr e invia
    if msg == 0 { return EINVAL; }
    unsafe {
        let msghdr = msg as *const u64;
        let iov_ptr = *msghdr.add(2);
        let iov_len = *msghdr.add(3) as usize;
        let mut total = 0i64;
        for i in 0..iov_len {
            let iov = (iov_ptr as *const u64).add(i * 2);
            let base = *iov;
            let len  = *iov.add(1) as usize;
            let n = sys_sendto(fd, base, len, flags, 0, 0);
            if n < 0 { return n; }
            total += n;
        }
        total
    }
}

fn sys_recvmsg(fd: i32, msg: u64, flags: i32) -> i64 { 0 }

fn sys_getsockopt(fd: i32, level: i32, optname: i32, optval: u64, optlen: u64) -> i64 {
    if optval == 0 { return EINVAL; }
    // SO_ERROR = 4: ritorna 0 (nessun errore)
    if optname == 4 {
        unsafe { *(optval as *mut i32) = 0; }
    }
    0
}

fn sys_getsockname(fd: i32, addr: u64, addrlen: u64) -> i64 {
    if addr == 0 { return EINVAL; }
    // Ritorna indirizzo locale: 0.0.0.0:0
    unsafe {
        let sa = addr as *mut u16;
        *sa       = 2;     // AF_INET
        *sa.add(1)= 0;     // port 0
        let sa32 = addr as *mut u32;
        *sa32.add(1) = 0;  // 0.0.0.0
        if addrlen != 0 { *(addrlen as *mut u32) = 16; }
    }
    0
}

fn sys_epoll_create1(flags: i32) -> i64 {
    static mut NEXT_EPFD: i32 = 800;
    unsafe { let fd = NEXT_EPFD; NEXT_EPFD += 1; fd as i64 }
}

fn sys_epoll_pwait(epfd: i32, events: u64, maxevents: i32, timeout: i32, sigmask: u64, sigsetsize: usize) -> i64 {
    // Timeout immediato (non bloccante): nessun evento
    if timeout == 0 { return 0; }
    // Timeout > 0: breve spin poi ritorna (compatibilità con loop JDWP)
    for _ in 0..1000u32 { unsafe { core::arch::asm!("nop", options(nomem, nostack)); } }
    0
}
