# ZTOS ART Integration – Guida Completa

## Panoramica

Questa documentazione descrive come compilare Android Runtime (ART) su ZTOS.
L'approccio è un "stub layer": invece di portare ART completamente su bare metal,
usiamo una libreria di compatibilità Rust (`libztos_compat.a`) che intercetta
tutte le syscall Linux e le traduce in chiamate ai server ZTOS.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Applicazione Android (.apk)                       │
├─────────────────────────────────────────────────────────────────────┤
│          Android Runtime (ART) – compilato con -DZTOS_COMPAT        │
├─────────────────────────────────────────────────────────────────────┤
│    bionic (libc Android) – patchata per redirigere a ZTOS           │
│    ┌──────────────────────────────────────────────────────────────┐ │
│    │  syscall() → ztos_syscall_wrapper()  (invece di svc #0)      │ │
│    └──────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│              libztos_compat.a (questo progetto)                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │  memory  │ │    io    │ │  thread  │ │   net    │ │  sched   │ │
│  │  bridge  │ │  bridge  │ │  bridge  │ │  bridge  │ │  bridge  │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ │
├───────┼────────────┼────────────┼────────────┼────────────┼────────┤
│       ↓            ↓            ↓            ↓            ↓        │
│  ZTOS Kernel (capability check + server dispatch)                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐             │
│  │ Storage  │ │ Network  │ │  Memory  │ │ Display  │             │
│  │  Server  │ │  Server  │ │  Server  │ │  Server  │             │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘             │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Prerequisiti

### Software necessario
```bash
# Rust nightly con target Android
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain nightly
rustup target add aarch64-linux-android

# Android NDK r26b (per il compilatore aarch64-linux-android21-clang)
wget https://dl.google.com/android/repository/android-ndk-r26b-linux.zip
unzip android-ndk-r26b-linux.zip -d /opt
export ANDROID_NDK_HOME=/opt/android-ndk-r26b
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"

# AOSP repo tool
curl https://storage.googleapis.com/git-repo-downloads/repo > ~/bin/repo
chmod +x ~/bin/repo

# Build tools
sudo apt install cmake ninja-build patch git python3
```

### Spazio disco richiesto
| Componente | Dimensione |
|------------|-----------|
| AOSP sync (solo art+bionic) | ~8 GB |
| AOSP sync completo | ~120 GB |
| Android NDK | ~3 GB |
| Output build | ~2 GB |

---

## Installazione rapida (con Docker)

```bash
# Clone del progetto
git clone https://github.com/yourorg/ztos
cd ztos/art_integration

# Build con Docker (include tutto il toolchain)
docker build -t ztos-art-build ./build/

# Entra nel container
docker run -it -v $(pwd):/ztos/art_integration ztos-art-build

# Dentro il container, esegui lo script di build
cd /ztos/art_integration && ./build/patch_and_build.sh --skip-sync
```

---

## Installazione manuale

### Step 1: Sync AOSP selettivo

```bash
mkdir /aosp && cd /aosp
repo init -u https://android.googlesource.com/platform/manifest \
          -b android-14.0.0_r1 --depth=1

# Solo i moduli necessari (~8GB invece di ~120GB)
repo sync -j8 --no-tags art bionic external/libcxx libnativehelper \
          prebuilts/clang/host/linux-x86
```

### Step 2: Compila libztos_compat.a

```bash
cd /ztos/art_integration

# Compila la libreria di compatibilità Rust
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=aarch64-linux-android21-clang \
cargo build \
    --target aarch64-linux-android \
    --release

# Risultato: target/aarch64-linux-android/release/libztos_compat.a
ls -sh target/aarch64-linux-android/release/libztos_compat.a
```

### Step 3: Applica le patch

```bash
cd /aosp

# Patch bionic (ridirigi syscall)
patch -p1 < /ztos/art_integration/patches/bionic_syscall.patch

# Patch pthread_create (propaga contesto ZTOS)
patch -p1 < /ztos/art_integration/patches/bionic_pthread.patch

# Patch ART MemMap (usa memory_bridge)
patch -p1 < /ztos/art_integration/patches/art_mem_map.patch

# Patch ART native I/O
patch -p1 < /ztos/art_integration/patches/art_native_io.patch
```

### Step 4: Configura e compila ART

```bash
mkdir -p /out/art_build && cd /out/art_build

cmake /aosp/art \
    -DCMAKE_TOOLCHAIN_FILE=/ztos/art_integration/build/toolchain-ztos.cmake \
    -DZTOS_COMPAT=ON \
    -DZTOS_COMPAT_LIB=/ztos/art_integration/target/aarch64-linux-android/release/libztos_compat.a \
    -DCMAKE_BUILD_TYPE=Release \
    -G Ninja

ninja -j$(nproc) libart libart-compiler
```

### Step 5: Verifica

```bash
# Controlla che i simboli ZTOS siano presenti
nm /out/art_build/libart.so | grep ztos_syscall_wrapper
# Atteso: T ztos_syscall_wrapper (simbolo esportato)

# Controlla le dimensioni
ls -sh /out/art_build/libart.so /out/art_build/libart-compiler.so

# Test di linking
aarch64-linux-android21-clang \
    -o /tmp/test_art \
    /ztos/art_integration/target/.../libztos_compat.a \
    /out/art_build/libart.so \
    -Wl,--allow-shlib-undefined
```

---

## Come funziona il mapping syscall → capability

### Esempio: mmap() in ART

**Flusso originale (Linux):**
```
ART::MemMap::MapAnonymous()
  → mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_ANONYMOUS|MAP_PRIVATE, -1, 0)
    → svc #0 (syscall __NR_mmap=222)
      → kernel Linux: alloca pagine fisiche
```

**Flusso ZTOS:**
```
ART::MemMap::MapAnonymous()
  → ztos_mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_ANONYMOUS|MAP_PRIVATE, -1, 0)
    → ztos_syscall_wrapper(222, NULL, size, PROT_R|W, MAP_ANON|PRIV, -1, 0)
      → capability check: has_capability(app_id, storage_ep, CAP_APP_STORAGE)
        → memory_bridge::sys_mmap()
          → bump_alloc(size) dal pool ZTOS (256MB statico)
            → restituisce puntatore al pool
```

### Tabella mapping (selezione)

| Syscall Linux | ZTOS Handler | Capability | Server |
|---------------|--------------|------------|--------|
| `mmap(anon)`  | `memory_bridge::sys_mmap` | `CAP_MEMORY_ALLOC` | Memory pool |
| `mmap(file)`  | `io_bridge::map_file` | `CAP_APP_STORAGE` | Storage server |
| `mprotect(EXEC)` | `memory_bridge::sys_mprotect` | `CAP_JIT_EXEC` | Kernel |
| `openat` | `io_bridge::sys_openat` | `CAP_APP_STORAGE` | Storage server |
| `read/write` | `io_bridge::sys_read/write` | `CAP_APP_STORAGE` | Storage server |
| `write(fd=1)` | `io_bridge::sys_write` | `CAP_STDOUT_WRITE` | UART |
| `clone` | `thread_bridge::sys_clone` | `CAP_THREAD_CREATE` | Scheduler |
| `futex` | `thread_bridge::sys_futex` | nessuna | Noop/spin |
| `socket` | `net_bridge::sys_socket` | `CAP_NET_SEND` | Network server |
| `connect` | `net_bridge::sys_connect` | `CAP_NET_SEND` + VPN check | Network server |
| `clock_gettime` | `sched_bridge::sys_clock_gettime` | nessuna | Orologio ZTOS |

---

## Troubleshooting

### Errore: "capability denied" durante mmap
```
[ZTOS] Syscall mmap denied for app_id=101 (capability check failed)
```
**Causa:** L'app non ha `CAP_MEMORY_ALLOC` o `CAP_APP_STORAGE`.
**Soluzione:** Verificare che `package_manager::install()` abbia creato correttamente la sandbox:
```rust
art::package_manager::install(&APP_SIGNAL, DOMAIN_PUBLIC, true);
// Questo chiama create_sandbox() che concede le capability necessarie
```

### Errore: "kill switch active" durante connect()
```
[ZTOS] Syscall connect denied for app_id=102 (kill switch active for domain 20)
```
**Causa:** La VPN del dominio PRIVATE è disconnessa.
**Soluzione:** Verificare stato VPN:
```rust
net::vpn::connect(DOMAIN_PRIVATE);  // Riconnette la VPN
```

### ART crasha in JIT compilation
Il JIT richiede `mprotect(PROT_EXEC)` che in ZTOS richiede `CAP_JIT_EXEC`.
Aggiungere al sandbox dell'app:
```rust
grant_capability(app_id, DISPLAY_ENDPOINT_ID, CAP_JIT_EXEC);
```

### linker: undefined symbol `ztos_syscall_wrapper`
```
ld: error: undefined symbol: ztos_syscall_wrapper
```
Assicurarsi di linkare `libztos_compat.a`:
```bash
-Wl,--whole-archive /path/to/libztos_compat.a -Wl,--no-whole-archive
```

---

## Limitazioni note in V0.7

1. **GC non completamente funzionante**: Il garbage collector di ART usa pattern di memoria
   complessi (read barriers, card tables) che richiedono il pool ZTOS esteso.
   
2. **JIT code cache**: La compilazione JIT funziona ma i thread JIT non sono schedulati
   preemptivamente (ZTOS usa scheduling cooperativo).

3. **Multi-threading**: `clone()` registra i thread ma non li esegue su core separati
   (ZTOS V0.7 è single-threaded cooperativo).

4. **Binder IPC**: Il Binder Android (usato per IPC tra app) non è implementato;
   le app che lo usano intensamente (Gmail, etc.) potrebbero non funzionare.

5. **Ashmem**: `/dev/ashmem` è sostituito con `memfd_create`; la compatibilità è parziale.

---

## Roadmap V0.8+

- [ ] Scheduler preemptivo ZTOS (necessario per JIT + GC concorrente)
- [ ] Binder IPC → ZTOS IPC bridge
- [ ] Ashmem → memfd completo
- [ ] ART GC con pool dinamico (mmap reale con MMU)
- [ ] Dalvik bytecode interpreter come fallback JIT
- [ ] Profilo SELinux → ZTOS capability automatica
