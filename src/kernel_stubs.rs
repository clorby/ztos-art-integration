// Stub del kernel ZTOS per la libreria di compatibilità ART.
// In produzione questi chiamerebbero il kernel ZTOS reale via IPC.
// Qui forniscono implementazioni minimali che permettono la compilazione.

// ── Capability stubs ──────────────────────────────────────────────────────
// (valori copiati da kernel/capability.rs)
pub const CAP_SEND:           u32 = 0x0001;
pub const CAP_RECV:           u32 = 0x0002;
pub const CAP_READ_STORAGE:   u32 = 0x0004;
pub const CAP_WRITE_STORAGE:  u32 = 0x0008;
pub const CAP_BINDER_CLIENT:  u32 = 0x0010;
pub const CAP_STDOUT_WRITE:   u32 = 0x0020;
pub const CAP_GUI_RENDER:     u32 = 0x0040;
pub const CAP_TOUCH_READ:     u32 = 0x0080;
pub const CAP_APP_LAUNCH:     u32 = 0x0100;
pub const CAP_NET_SEND:       u32 = 0x0200;
pub const CAP_NET_RECV:       u32 = 0x0400;
pub const CAP_VPN_CONTROL:    u32 = 0x0800;
pub const CAP_ROUTE_SET:      u32 = 0x1000;
pub const CAP_CAMERA:         u32 = 0x0001_0000;
pub const CAP_CONTACTS:       u32 = 0x0002_0000;
pub const CAP_LOCATION:       u32 = 0x0004_0000;
pub const CAP_CALENDAR:       u32 = 0x0008_0000;
pub const CAP_EMAIL:          u32 = 0x0010_0000;
pub const CAP_APP_STORAGE:    u32 = 0x0020_0000;

// Capability aggiunte per ART/JIT
pub const CAP_MEMORY_ALLOC:   u32 = 0x0100_0000;
pub const CAP_MEMORY_PROT:    u32 = 0x0200_0000;
pub const CAP_JIT_ALLOC:      u32 = 0x0400_0000;
pub const CAP_JIT_EXEC:       u32 = 0x0800_0000;
pub const CAP_THREAD_CREATE:  u32 = 0x1000_0000;
pub const CAP_SHARED_MEM:     u32 = 0x2000_0000;
pub const CAP_SIGNAL:         u32 = 0x4000_0000;
pub const CAP_SCHED:          u32 = 0;  // nessuna capability richiesta per scheduling
pub const CAP_PROC_INFO:      u32 = 0x0000_0000; // libero, nessuna cap
pub const CAP_PROC_CONTROL:   u32 = 0x0000_0000; // libero
pub const CAP_ENTROPY:        u32 = 0x0000_0000; // libero
pub const CAP_DEBUG:          u32 = 0x0000_0001; // stessa di CAP_SEND (mai concessa a app)
pub const CAP_PERF:           u32 = 0x0000_0001;

pub const DISPLAY_ENDPOINT_ID:  u32 = 300;
pub const NETWORK_ENDPOINT_ID:  u32 = 400;
pub const CONSOLE_ENDPOINT_ID:  u32 = 102;
pub const APP_STORAGE_BASE:     u32 = 1000;

/// Verifica capability – in modalità compat ritorna sempre true per le capability
/// necessarie alle app (tranne CAP_DEBUG).
/// In produzione: IPC al capability server ZTOS.
pub fn has_capability(task_id: u32, endpoint_id: u32, required_rights: u32) -> bool {
    // Debug capabilities: SEMPRE negate (zero trust per debugger/ptrace)
    if required_rights & CAP_DEBUG != 0 && required_rights != CAP_SEND {
        return false;
    }
    // App con app_id > 0 ha tutto il necessario per girare
    task_id > 0
}

/// Revoca tutte le capability di un task.
pub fn revoke_all_for_task(_task_id: u32) {
    // In produzione: IPC al kernel ZTOS
}

// ── TEE stubs ─────────────────────────────────────────────────────────────
static mut TEE_KEY_COUNTER: u32 = 0;

pub fn tee_generate_key() -> u32 {
    unsafe { TEE_KEY_COUNTER += 1; TEE_KEY_COUNTER }
}

// ── VPN stubs ─────────────────────────────────────────────────────────────
/// Kill switch: false = VPN attiva = traffico permesso.
pub fn vpn_kill_switch_active(_domain_id: u32) -> bool {
    false // In compat mode: VPN sempre attiva
}

pub fn vpn_gateway(_domain_id: u32) -> [u8; 4] {
    [10, 0, 0, 1]
}

pub fn vpn_name(_domain_id: u32) -> &'static str {
    "VPN-compat"
}

// ── VirtIO NET stub ────────────────────────────────────────────────────────
/// Invia frame di rete: in compat mode lo scarta silenziosamente.
/// In produzione: VirtIO TX queue via memory-mapped I/O.
pub fn virtio_send_frame(_data: &[u8]) -> bool {
    true // "inviato" (noop)
}

// ── UART stub ─────────────────────────────────────────────────────────────
/// Stampa stringa – usa write(2, ...) sul fd stderr.
/// In produzione: semihosting ZTOS.
pub fn uart_print_str(s: &str) {
    // In no_std non possiamo chiamare write() direttamente.
    // In produzione: semihosting o UART MMIO.
    // Qui: noop (il codice client userà write() via io_bridge).
    let _ = s;
}
