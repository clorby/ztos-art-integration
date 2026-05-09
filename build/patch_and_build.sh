#!/bin/bash
################################################################################
# ZTOS ART Integration – Script di patch e build automatico
# Uso: ./patch_and_build.sh [--aosp-dir /path/to/aosp] [--skip-sync]
################################################################################

set -euo pipefail

# ── Configurazione ────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INTEGRATION_DIR="$(dirname "$SCRIPT_DIR")"
AOSP_DIR="${AOSP_DIR:-/aosp}"
SKIP_SYNC="${SKIP_SYNC:-false}"
OUT_DIR="${OUT_DIR:-${INTEGRATION_DIR}/output}"
JOBS="${JOBS:-$(nproc)}"

# Colori output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# ── Parse argomenti ────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case $1 in
        --aosp-dir) AOSP_DIR="$2"; shift 2 ;;
        --skip-sync) SKIP_SYNC=true; shift ;;
        --jobs) JOBS="$2"; shift 2 ;;
        --help) echo "Uso: $0 [--aosp-dir /path] [--skip-sync] [--jobs N]"; exit 0 ;;
        *) log_error "Argomento non riconosciuto: $1" ;;
    esac
done

# ── Step 1: Verifica prerequisiti ─────────────────────────────────────────
log_info "Step 1: Verifica prerequisiti..."

check_cmd() {
    command -v "$1" &>/dev/null || log_error "Comando non trovato: $1. Installa con: $2"
}

check_cmd cargo   "curl https://sh.rustup.rs | sh"
check_cmd repo    "apt install repo  OR  curl https://storage.googleapis.com/git-repo-downloads/repo > ~/bin/repo"
check_cmd patch   "apt install patch"
check_cmd cmake   "apt install cmake"
check_cmd ninja   "apt install ninja-build"

# Verifica target Rust
if ! rustup target list --installed | grep -q "aarch64-linux-android"; then
    log_warn "Aggiunta target Rust aarch64-linux-android..."
    rustup target add aarch64-linux-android
fi

log_ok "Prerequisiti OK"

# ── Step 2: Clone/sync AOSP ────────────────────────────────────────────────
if [[ "$SKIP_SYNC" == "false" ]]; then
    log_info "Step 2: Sync AOSP (solo moduli necessari)..."
    mkdir -p "$AOSP_DIR"
    cd "$AOSP_DIR"

    if [[ ! -d ".repo" ]]; then
        log_info "Inizializzazione repo AOSP..."
        repo init -u https://android.googlesource.com/platform/manifest \
                  -b android-14.0.0_r1 \
                  --depth=1
    fi

    log_info "Sync selettivo (art + bionic)..."
    repo sync -j"$JOBS" --no-tags --no-clone-bundle \
        art \
        bionic \
        external/libcxx \
        external/libcxxabi \
        libnativehelper \
        prebuilts/clang/host/linux-x86 \
        2>&1 | grep -v "^Fetching\|^remote:"

    log_ok "AOSP sync completato"
else
    log_warn "Step 2: Skip sync (--skip-sync specificato)"
fi

# ── Step 3: Compila libztos_compat.a ───────────────────────────────────────
log_info "Step 3: Compilazione libztos_compat.a..."

cd "$INTEGRATION_DIR"
mkdir -p src
# Crea lib.rs che include tutti i bridge
cat > src/lib.rs << 'RUST_EOF'
#![no_std]
#![allow(dead_code, unused_variables, unreachable_code)]

// ZTOS ART Compatibility Layer
// Questo crate è compilato come static library e linkato contro ART/bionic patchati.

extern crate core;

// Moduli bridge
pub mod syscall_table;
pub mod syscall_wrapper;
pub mod memory_bridge;
pub mod io_bridge;
pub mod thread_bridge;
pub mod net_bridge;
pub mod sched_bridge;

// Esporta le funzioni richieste dal C++ di bionic/ART
pub use syscall_wrapper::ztos_syscall_wrapper;
pub use syscall_wrapper::ztos_set_app_context;

// Re-esporta wrappers C-compatibili
#[no_mangle]
pub extern "C" fn ztos_mmap(addr: u64, len: usize, prot: u32, flags: u32, fd: i32, off: u64) -> u64 {
    memory_bridge::handle(
        syscall_table::SYS_MMAP,
        addr, len as u64, prot as u64, flags as u64, fd as u64, off,
    ) as u64
}

#[no_mangle]
pub extern "C" fn ztos_munmap(addr: u64, len: usize) -> i32 {
    memory_bridge::handle(syscall_table::SYS_MUNMAP, addr, len as u64, 0, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_mprotect(addr: u64, len: usize, prot: u32) -> i32 {
    memory_bridge::handle(syscall_table::SYS_MPROTECT, addr, len as u64, prot as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_openat_compat(path: *const u8, flags: u32, mode: u32) -> i32 {
    io_bridge::handle(
        syscall_table::SYS_OPENAT,
        0xFFFFFF9Cu64, // AT_FDCWD
        path as u64, flags as u64, mode as u64, 0, 0,
    ) as i32
}

#[no_mangle]
pub extern "C" fn ztos_read(fd: i32, buf: *mut u8, len: usize) -> i64 {
    io_bridge::handle(syscall_table::SYS_READ, fd as u64, buf as u64, len as u64, 0, 0, 0)
}

#[no_mangle]
pub extern "C" fn ztos_write(fd: i32, buf: *const u8, len: usize) -> i64 {
    io_bridge::handle(syscall_table::SYS_WRITE, fd as u64, buf as u64, len as u64, 0, 0, 0)
}

#[no_mangle]
pub extern "C" fn ztos_close(fd: i32) -> i32 {
    io_bridge::handle(syscall_table::SYS_CLOSE, fd as u64, 0, 0, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_fstat(fd: i32, stat_buf: *mut u8) -> i32 {
    io_bridge::handle(syscall_table::SYS_FSTAT, fd as u64, stat_buf as u64, 0, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_socket(domain: i32, stype: i32, proto: i32) -> i32 {
    net_bridge::handle(syscall_table::SYS_SOCKET, domain as u64, stype as u64, proto as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_connect(fd: i32, addr: *const u8, len: u32) -> i32 {
    net_bridge::handle(syscall_table::SYS_CONNECT, fd as u64, addr as u64, len as u64, 0, 0, 0) as i32
}

#[no_mangle]
pub extern "C" fn ztos_get_current_app_id() -> u32 {
    syscall_wrapper::current_app_id()
}

#[no_mangle]
pub extern "C" fn ztos_get_current_domain_id() -> u32 {
    syscall_wrapper::current_domain_id()
}

// panic handler per no_std
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
RUST_EOF

# Compila con NDK clang
NDK_CC="${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
if [[ -f "$NDK_CC" ]]; then
    CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_CC" \
    cargo build \
        --target aarch64-linux-android \
        --release \
        2>&1 | tail -20
    log_ok "libztos_compat.a compilata"
else
    log_warn "NDK non trovato in $ANDROID_NDK_HOME. Compilazione skippata."
    log_warn "Scarica NDK da: https://developer.android.com/ndk/downloads"
fi

# ── Step 4: Applica patch a AOSP ────────────────────────────────────────────
log_info "Step 4: Applicazione patch AOSP..."

cd "$AOSP_DIR"
PATCH_DIR="${INTEGRATION_DIR}/patches"
APPLIED=0
FAILED=0

for patch_file in "${PATCH_DIR}"/*.patch; do
    patch_name=$(basename "$patch_file")
    log_info "  Applico $patch_name..."
    if patch -p1 --forward --batch --dry-run < "$patch_file" &>/dev/null; then
        patch -p1 --forward --batch < "$patch_file"
        log_ok "  $patch_name applicata"
        ((APPLIED++))
    else
        # Controlla se già applicata
        if patch -p1 --forward --batch --dry-run -R < "$patch_file" &>/dev/null; then
            log_warn "  $patch_name già applicata"
        else
            log_warn "  $patch_name fallita (file potrebbe non esistere nel sync parziale)"
            ((FAILED++))
        fi
    fi
done

log_info "Patch: $APPLIED applicate, $FAILED fallite"

# ── Step 5: Configura build ART ─────────────────────────────────────────────
log_info "Step 5: Configurazione build ART con toolchain ZTOS..."

COMPAT_LIB="${INTEGRATION_DIR}/target/aarch64-linux-android/release/libztos_compat.a"
mkdir -p "$OUT_DIR/art_build"
cd "$OUT_DIR/art_build"

cmake "${AOSP_DIR}/art" \
    -DCMAKE_TOOLCHAIN_FILE="${SCRIPT_DIR}/toolchain-ztos.cmake" \
    -DZTOS_COMPAT=ON \
    -DZTOS_COMPAT_LIB="${COMPAT_LIB}" \
    -DCMAKE_BUILD_TYPE=Release \
    -DART_TARGET_ANDROID_VERSION=34 \
    -G Ninja \
    2>&1 | tail -30 || {
    log_warn "cmake fallito (normale se AOSP non completamente sincronizzato)"
    log_warn "Riprova dopo: repo sync -j$JOBS art bionic prebuilts/clang/host/linux-x86"
}

# ── Step 6: Build finale ─────────────────────────────────────────────────────
log_info "Step 6: Build ART patchato..."
if [[ -f "${OUT_DIR}/art_build/build.ninja" ]]; then
    ninja -C "${OUT_DIR}/art_build" -j"$JOBS" libart libart-compiler 2>&1 | tail -50
    log_ok "Build completato! Artefatti in: ${OUT_DIR}/art_build"
else
    log_warn "Skippo ninja: cmake non ha generato build.ninja"
fi

# ── Step 7: Verifica ─────────────────────────────────────────────────────────
log_info "Step 7: Verifica integrazione..."

LIBART="${OUT_DIR}/art_build/libart.so"
if [[ -f "$LIBART" ]]; then
    # Verifica che i simboli ZTOS siano presenti
    if nm "$LIBART" 2>/dev/null | grep -q "ztos_syscall_wrapper"; then
        log_ok "Simbolo ztos_syscall_wrapper trovato in libart.so"
    else
        log_warn "Simbolo ztos_syscall_wrapper non trovato (link non completato)"
    fi
    log_ok "libart.so: $(ls -sh "$LIBART")"
else
    log_warn "libart.so non generato (richiede AOSP completo)"
fi

# ── Sommario ──────────────────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════════"
echo " ZTOS ART Integration Build Summary"
echo "════════════════════════════════════════════════════════════"
echo " libztos_compat.a : $(ls -sh "${COMPAT_LIB}" 2>/dev/null || echo 'non disponibile')"
echo " Patch applicate   : $APPLIED"
echo " libart.so         : $(ls -sh "$LIBART" 2>/dev/null || echo 'non disponibile')"
echo ""
echo " Per eseguire ART su ZTOS:"
echo "   1. Copia libart.so nel ZTOS rootfs"
echo "   2. Lancia: app_process64 --ztos-mode /system/lib64 com.example.app"
echo "════════════════════════════════════════════════════════════"
