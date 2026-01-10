# --- Staccato Cross-Platform Toolchain ---

if (STACCATO_INSTALL_ROOT)
    message(STATUS "using install root:${STACCATO_INSTALL_ROOT}")
    set(CMAKE_FIND_ROOT_PATH "${STACCATO_INSTALL_ROOT}")
    set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
    set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
    set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
    set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
    set(STACCATO_INSTALL_ROOT "${STACCATO_INSTALL_ROOT}" CACHE PATH "Root path for staccato installation path")
else ()
    set(STACCATO_INSTALL_ROOT "${CMAKE_CURRENT_LIST_DIR}/../install-Debug/native")
    message(STATUS "using install root:${STACCATO_INSTALL_ROOT}")
    set(CMAKE_FIND_ROOT_PATH "${STACCATO_INSTALL_ROOT}")
    set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
    set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
    set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
    set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
endif()

# For macos:
# Using HomeBrew, it's usually newer
# For other operation system:
# Using installed toolchain is usually enough
if(APPLE)
    if(NOT STACCATO_LLVM_ROOT)
        set(BREW_LLVM_PATHS "/opt/homebrew/opt/llvm" "/usr/local/opt/llvm")
        foreach(path ${BREW_LLVM_PATHS})
            if(EXISTS "${path}/bin/clang")
                set(STACCATO_LLVM_ROOT "${path}")
                break()
            endif()
        endforeach()
    endif()

    if(STACCATO_LLVM_ROOT)
        message(STATUS "Staccato: Using Homebrew LLVM at ${STACCATO_LLVM_ROOT}")
        set(STACCATO_LLVM_ROOT "${STACCATO_LLVM_ROOT}" CACHE PATH "LLVM root path.")
        set(LLVM_BIN "${STACCATO_LLVM_ROOT}/bin" CACHE PATH "LLVM root path. From STACCATO_LLVM_ROOT.")
    else()
        message(WARNING "Staccato: Homebrew LLVM not found in macos(we recommend you to use homebrew to install llvm for a newer version), falling back to system compiler.")
    endif()
endif()

if(LLVM_BIN)
    set(CMAKE_C_COMPILER   "${LLVM_BIN}/clang")
    set(CMAKE_CXX_COMPILER "${LLVM_BIN}/clang++")
    set(CMAKE_AR           "${LLVM_BIN}/llvm-ar")
    set(CMAKE_RANLIB       "${LLVM_BIN}/llvm-ranlib")
    set(CMAKE_NM           "${LLVM_BIN}/llvm-nm")
    set(CMAKE_OBJCOPY      "${LLVM_BIN}/llvm-objcopy")
    set(CMAKE_OBJDUMP      "${LLVM_BIN}/llvm-objdump")
    set(CMAKE_STRIP        "${LLVM_BIN}/llvm-strip")
else()
    set(CMAKE_C_COMPILER   "clang")
    set(CMAKE_CXX_COMPILER "clang++")
    set(CMAKE_AR           "llvm-ar")
    set(CMAKE_RANLIB       "llvm-ranlib")
    set(CMAKE_NM           "llvm-nm")
    set(CMAKE_OBJCOPY      "llvm-objcopy")
    set(CMAKE_OBJDUMP      "llvm-objdump")
    set(CMAKE_STRIP        "llvm-strip")
endif()

set(CMAKE_C_COMPILER_ID "Clang")
set(CMAKE_CXX_COMPILER_ID "Clang")
set(CMAKE_C_STANDARD 17)
set(CMAKE_CXX_STANDARD 17)

if(APPLE)
    if (NOT CMAKE_OSX_SYSROOT)
        execute_process(COMMAND xcrun --show-sdk-path
                OUTPUT_VARIABLE STACCATO_SDK_PATH
                OUTPUT_STRIP_TRAILING_WHITESPACE)
        list(APPEND CMAKE_FIND_ROOT_PATH "${STACCATO_SDK_PATH}")
        set(CMAKE_OSX_SYSROOT ${STACCATO_SDK_PATH} CACHE PATH "macOS SDK root")
        message(STATUS "Staccato: Added macOS SDK to root path: ${STACCATO_SDK_PATH}")
    endif ()

    if(NOT CMAKE_OSX_DEPLOYMENT_TARGET)
        set(CMAKE_OSX_DEPLOYMENT_TARGET "13.0" CACHE STRING "Minimum macOS version")
    endif()
else()
    if (NOT STACCATO_SET_SYSTEM_ROOT)
        list(APPEND CMAKE_FIND_ROOT_PATH "/usr" "/")
        set(STACCATO_SET_SYSTEM_ROOT ON)
    endif ()
endif ()

set(STACCATO_LTO_FLAGS "-flto=thin")

if(APPLE)
    # lld works may not find on macos
    set(STACCATO_LLD_FLAGS "")
else()
    set(STACCATO_LLD_FLAGS "-fuse-ld=lld")
endif()

if(APPLE)
    # SDL needs them
    set(STACCATO_FRAMEWORKS "-framework CoreHaptics -framework GameController -framework CoreMedia")
else()
    set(STACCATO_FRAMEWORKS "")
endif()

# enable fPIC always
set(CMAKE_POSITION_INDEPENDENT_CODE ON CACHE BOOL "Enable fPIC globally" FORCE)

if(WIN32)
    set(CMAKE_FIND_LIBRARY_SUFFIXES ".lib" ".a" ".dll.a" CACHE STRING "Priority for static libs" FORCE)
else()
    set(CMAKE_FIND_LIBRARY_SUFFIXES ".a" ".dylib" ".so" CACHE STRING "Priority for static libs" FORCE)
endif()

set(CMAKE_C_FLAGS_INIT   "${STACCATO_LTO_FLAGS}")
set(CMAKE_CXX_FLAGS_INIT "${STACCATO_LTO_FLAGS}")
set(CMAKE_EXE_LINKER_FLAGS_INIT   "${STACCATO_LTO_FLAGS} ${STACCATO_LLD_FLAGS} ${STACCATO_FRAMEWORKS}")
set(CMAKE_SHARED_LINKER_FLAGS_INIT "${STACCATO_LTO_FLAGS} ${STACCATO_LLD_FLAGS} ${STACCATO_FRAMEWORKS}")
