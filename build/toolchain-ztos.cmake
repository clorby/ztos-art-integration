################################################################################
# CMake Toolchain File: aarch64-linux-android → ZTOS
# Usato per compilare ART con i bridge ZTOS invece del kernel Android
################################################################################

set(CMAKE_SYSTEM_NAME Android)
set(CMAKE_SYSTEM_VERSION 21)  # API 21 = Android 5.0 minimo per ART v2
set(CMAKE_ANDROID_ARCH_ABI arm64-v8a)

# ── Toolchain ─────────────────────────────────────────────────────────────
set(ANDROID_NDK $ENV{ANDROID_NDK_HOME})
set(NDK_TOOLCHAIN "${ANDROID_NDK}/toolchains/llvm/prebuilt/linux-x86_64")

set(CMAKE_C_COMPILER   "${NDK_TOOLCHAIN}/bin/aarch64-linux-android21-clang")
set(CMAKE_CXX_COMPILER "${NDK_TOOLCHAIN}/bin/aarch64-linux-android21-clang++")
set(CMAKE_ASM_COMPILER "${NDK_TOOLCHAIN}/bin/aarch64-linux-android21-clang")
set(CMAKE_AR           "${NDK_TOOLCHAIN}/bin/llvm-ar")
set(CMAKE_RANLIB       "${NDK_TOOLCHAIN}/bin/llvm-ranlib")
set(CMAKE_STRIP        "${NDK_TOOLCHAIN}/bin/llvm-strip")

# ── Flag ZTOS ─────────────────────────────────────────────────────────────
set(ZTOS_COMPAT_FLAGS
    "-DZTOS_COMPAT=1"
    "-DZTOS_VERSION=7"
    "-DANDROID_API_LEVEL=34"
    # Disabilita funzionalità Android che non esistono su ZTOS
    "-DART_USE_READ_BARRIER=0"          # GC read barrier: gestito da memory_bridge
    "-DART_USE_GENERATIONAL_CC=0"        # GC generazionale: semplificato in V0.7
    "-DART_USE_SEALING=0"                # Memory sealing: non disponibile
    "-DART_ENABLE_FAST_JNI=1"            # Fast JNI: compatibile
)

set(CMAKE_C_FLAGS   "${CMAKE_C_FLAGS}   ${ZTOS_COMPAT_FLAGS}")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${ZTOS_COMPAT_FLAGS} -std=c++17")
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} ${ZTOS_COMPAT_FLAGS}")

# ── Include paths ─────────────────────────────────────────────────────────
set(ZTOS_INCLUDE_DIR "${CMAKE_SOURCE_DIR}/../art_integration/include")

include_directories(
    "${NDK_TOOLCHAIN}/sysroot/usr/include"
    "${NDK_TOOLCHAIN}/sysroot/usr/include/aarch64-linux-android"
    "${ZTOS_INCLUDE_DIR}"
    "${ZTOS_INCLUDE_DIR}/ztos"
)

# ── Link flags ────────────────────────────────────────────────────────────
set(ZTOS_COMPAT_LIB "" CACHE PATH "Path to libztos_compat.a")

if(ZTOS_COMPAT_LIB)
    set(CMAKE_EXE_LINKER_FLAGS
        "${CMAKE_EXE_LINKER_FLAGS} -Wl,--whole-archive ${ZTOS_COMPAT_LIB} -Wl,--no-whole-archive")
    set(CMAKE_SHARED_LINKER_FLAGS
        "${CMAKE_SHARED_LINKER_FLAGS} -Wl,--whole-archive ${ZTOS_COMPAT_LIB} -Wl,--no-whole-archive")
endif()

# Sysroot Android (per libc, libdl, etc.)
set(CMAKE_SYSROOT "${NDK_TOOLCHAIN}/sysroot")

# ── Ricerca librerie ──────────────────────────────────────────────────────
set(CMAKE_FIND_ROOT_PATH "${CMAKE_SYSROOT}")
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)

# ── Funzionalità disabilitate in ZTOS V0.7 ───────────────────────────────
# Queste funzionalità richiedono infrastruttura Android non disponibile
set(ART_BUILD_NDEBUG         ON)
set(ART_BUILD_HOST           OFF)  # Solo target AArch64
set(ART_USE_HSPACE_COMPACT   OFF)  # Heap compaction: semplificato
set(ART_HEAP_POISONING       OFF)  # Debug heap: disabilitato per performance

# ── Definizioni version ───────────────────────────────────────────────────
add_definitions(
    -DART_BASE_ADDRESS=0x70000000  # Base address ART heap in ZTOS
    -DZTOS_MEMORY_POOL_BASE=0x80000000  # Pool memoria ZTOS
    -DZTOS_MEMORY_POOL_SIZE=0x10000000  # 256MB
)

message(STATUS "ZTOS Toolchain configurato:")
message(STATUS "  Compiler: ${CMAKE_C_COMPILER}")
message(STATUS "  ZTOS compat lib: ${ZTOS_COMPAT_LIB}")
message(STATUS "  Include: ${ZTOS_INCLUDE_DIR}")
