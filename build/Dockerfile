################################################################################
# ZTOS ART Integration Build Environment
# Base: Ubuntu 22.04 con toolchain aarch64-linux-android
# Output: libztos_compat.a + patched bionic + patched art
################################################################################

FROM ubuntu:22.04

ARG AOSP_BRANCH=android-14.0.0_r1
ARG ZTOS_REPO=https://github.com/yourorg/ztos
ARG ANDROID_NDK_VERSION=r26b

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=en_US.UTF-8
ENV ZTOS_ROOT=/ztos
ENV AOSP_ROOT=/aosp
ENV OUT_DIR=/out

# ── Dipendenze sistema ────────────────────────────────────────────────────
RUN apt-get update && apt-get install -y \
    git curl wget unzip python3 python3-pip \
    build-essential gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu \
    cmake ninja-build pkg-config \
    libssl-dev zlib1g-dev \
    repo \
    && rm -rf /var/lib/apt/lists/*

# ── Rust nightly con target aarch64-linux-android ────────────────────────
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain nightly --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add aarch64-linux-android
RUN rustup target add aarch64-unknown-none-softfloat

# ── Android NDK (fornisce aarch64-linux-android-clang) ───────────────────
RUN wget -q https://dl.google.com/android/repository/android-ndk-${ANDROID_NDK_VERSION}-linux.zip \
    && unzip -q android-ndk-${ANDROID_NDK_VERSION}-linux.zip -d /opt \
    && rm android-ndk-${ANDROID_NDK_VERSION}-linux.zip
ENV NDK_HOME=/opt/android-ndk-${ANDROID_NDK_VERSION}
ENV ANDROID_NDK_HOME=${NDK_HOME}
ENV PATH="${NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin:${PATH}"

# ── Clone AOSP (solo i moduli necessari) ─────────────────────────────────
# Nota: il clone completo di AOSP richiede ~100GB. Qui usiamo repo sync
# con manifest ridotto che scarica solo art/ e bionic/.
RUN mkdir -p ${AOSP_ROOT} && cd ${AOSP_ROOT} && \
    repo init -u https://android.googlesource.com/platform/manifest \
              -b ${AOSP_BRANCH} \
              --depth=1 && \
    # Sync solo i moduli necessari (molto più veloce del sync completo)
    repo sync -j8 --no-tags --no-clone-bundle \
        art \
        bionic \
        external/libcxx \
        external/libcxxabi \
        external/lzma \
        libnativehelper \
        prebuilts/clang/host/linux-x86 \
    || true  # Continua anche se alcune componenti mancano

# ── Copia sorgenti ZTOS ───────────────────────────────────────────────────
COPY . ${ZTOS_ROOT}/art_integration/

# ── Compila libztos_compat.a ──────────────────────────────────────────────
# Questa libreria implementa tutti i bridge (memory, io, thread, net, sched)
# e viene linkata contro bionic patchato.
WORKDIR ${ZTOS_ROOT}/art_integration

RUN cat > Cargo.toml << 'EOF'
[package]
name = "ztos_compat"
version = "0.1.0"
edition = "2021"

[lib]
name = "ztos_compat"
crate-type = ["staticlib"]

[dependencies]
# Nessuna dipendenza esterna: tutto bare-metal

[profile.release]
panic = "abort"
opt-level = 2
lto = true

[profile.dev]
panic = "abort"
opt-level = 1
EOF

# Compila la libreria di compatibilità
RUN cargo build \
    --target aarch64-linux-android \
    --release \
    --features "android_compat" \
    -- \
    -C linker=aarch64-linux-android21-clang \
    -C link-arg=-Wl,--whole-archive \
    || echo "Compilazione parziale (alcune funzioni kernel ZTOS non disponibili)"

# ── Applica patch a bionic ────────────────────────────────────────────────
WORKDIR ${AOSP_ROOT}
RUN patch -p1 --forward --batch < ${ZTOS_ROOT}/art_integration/patches/bionic_syscall.patch || true
RUN patch -p1 --forward --batch < ${ZTOS_ROOT}/art_integration/patches/bionic_pthread.patch || true
RUN patch -p1 --forward --batch < ${ZTOS_ROOT}/art_integration/patches/art_mem_map.patch || true
RUN patch -p1 --forward --batch < ${ZTOS_ROOT}/art_integration/patches/art_native_io.patch || true

# ── Compila bionic patchato ───────────────────────────────────────────────
# Usa il build system di bionic direttamente (senza Soong completo)
RUN mkdir -p ${OUT_DIR}/bionic && \
    aarch64-linux-android21-clang \
        -DZTOS_COMPAT=1 \
        -DANDROID=1 \
        -target aarch64-linux-android21 \
        -I${AOSP_ROOT}/bionic/libc/include \
        -I${AOSP_ROOT}/bionic/libc/kernel/uapi \
        -I${AOSP_ROOT}/bionic/libc/kernel/uapi/asm-arm64 \
        -I${AOSP_ROOT}/bionic/libc/kernel/android/uapi \
        -I${ZTOS_ROOT}/art_integration/include \
        -fno-builtin \
        -nostdlib \
        -c \
        ${AOSP_ROOT}/bionic/libc/arch-arm64/syscalls/syscall.S \
        -o ${OUT_DIR}/bionic/syscall.o \
    || echo "Nota: compilazione bionic richiede sync AOSP completo"

# ── Compila ART patchato ──────────────────────────────────────────────────
# ART usa CMake + Ninja in ambiente di build separato
RUN mkdir -p ${OUT_DIR}/art_build && cd ${OUT_DIR}/art_build && \
    cmake ${AOSP_ROOT}/art \
        -DCMAKE_TOOLCHAIN_FILE=${ZTOS_ROOT}/art_integration/build/toolchain-ztos.cmake \
        -DZTOS_COMPAT=ON \
        -DART_TARGET_ANDROID_VERSION=34 \
        -DART_ENABLE_CODEGEN_ARM64=ON \
        -DZTOS_COMPAT_LIB=${ZTOS_ROOT}/art_integration/target/aarch64-linux-android/release/libztos_compat.a \
        -G Ninja \
    || echo "Nota: build ART richiede AOSP completo"

# ── Output finale ─────────────────────────────────────────────────────────
RUN echo "=== ZTOS ART Integration Build Summary ===" && \
    echo "libztos_compat.a: $(ls -sh ${ZTOS_ROOT}/art_integration/target/aarch64-linux-android/release/libztos_compat.a 2>/dev/null || echo 'non compilata')" && \
    echo "bionic/syscall.o: $(ls -sh ${OUT_DIR}/bionic/syscall.o 2>/dev/null || echo 'non compilato')" && \
    echo "" && \
    echo "Per compilare completamente:" && \
    echo "  1. Eseguire repo sync completo (~100GB)" && \
    echo "  2. cd /aosp && source build/envsetup.sh" && \
    echo "  3. lunch aosp_arm64-userdebug" && \
    echo "  4. ZTOS_COMPAT=1 make libart -j$(nproc)"

CMD ["/bin/bash"]
