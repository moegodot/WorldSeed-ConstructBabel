cmake_minimum_required(VERSION 4.1)

file(READ "${CMAKE_CURRENT_LIST_DIR}/../version.txt" VERSION_STRING)
string(STRIP "${VERSION_STRING}" VERSION_STRING)

project(Wscb VERSION "${VERSION_STRING}" LANGUAGES CXX C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Debug")
endif ()

if (NOT CMAKE_INSTALL_PREFIX)
    set(CMAKE_INSTALL_PREFIX "${CMAKE_CURRENT_SOURCE_DIR}/../artifact-${CMAKE_BUILD_TYPE}/native")
endif ()

set(CMAKE_CXX_STANDARD 20)

add_library(Wscb STATIC 
    "src/pluto_ft_bind.cpp"
)

set_target_properties(Wscb PROPERTIES WINDOWS_EXPORT_ALL_SYMBOLS ON)

find_package(Freetype REQUIRED)
find_package(harfbuzz REQUIRED)
find_package(SDL3 REQUIRED)
find_package(plutosvg REQUIRED)
find_package(PNG REQUIRED)
find_package(ZLIB REQUIRED)
find_package(BZIP2 REQUIRED)
find_library(BROTLIENC_LIBRARY NAMES brotlienc REQUIRED)
find_library(BROTLIDEC_LIBRARY NAMES brotlidec REQUIRED)
find_library(BROTLICOMMON_LIBRARY NAMES brotlicommon REQUIRED)

target_link_libraries(Wscb PRIVATE
    Freetype::Freetype
    SDL3::SDL3
    harfbuzz::harfbuzz
    plutosvg::plutosvg
    PNG::PNG
    ZLIB::ZLIB
    BZip2::BZip2
    ${BROTLICOMMON_LIBRARY}
    ${BROTLIENC_LIBRARY}
    ${BROTLIDEC_LIBRARY}
)

install(TARGETS Wscb LIBRARY DESTINATION .)

