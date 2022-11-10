cmake_minimum_required(VERSION 3.18)
set(RUSTC_MINIMUM_REQUIRED 1.56)

project(vaccelrt-agent
        VERSION "0.3.0"
        DESCRIPTION "A simple server to handle vAccel API operations.")

set(CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake" ${CMAKE_MODULE_PATH})

#
# Find Build Tools
#
set(MAINTAINER_MODE_DEFAULT OFF)
option(MAINTAINER_MODE
    "Use `cbindgen` to generate Rust library API headers."
    ${MAINTAINER_MODE_DEFAULT})

find_package(Rust REQUIRED)

# Always use '-fPIC'/'-fPIE' option.
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Include GNUInstallDirs for access to CMAKE_INSTALL_LIBDIR, etc
include(GNUInstallDirs)

# Enable CTest
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
    include(CTest)
    enable_testing()
endif()

add_rust_executable(TARGET ${PROJECT_NAME}
    SOURCE_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    BINARY_DIRECTORY "${CMAKE_BINARY_DIR}"
)
# Linking with C libraries is done within `build.rs`
# Linking with nearby Rust crates is done within `Cargo.toml`
get_target_property(${PROJECT_NAME}_EXECUTABLE ${PROJECT_NAME} IMPORTED_LOCATION)
install(PROGRAMS ${${PROJECT_NAME}_EXECUTABLE} DESTINATION ${CMAKE_INSTALL_BINDIR} COMPONENT programs)

add_subdirectory(packaging)
