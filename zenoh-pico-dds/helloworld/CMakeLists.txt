cmake_minimum_required(VERSION 3.19)

# This is your project statement. You should always list languages;
# Listing the version is nice here since it sets lots of useful variables
project(
    zenoh-cdr-example
    VERSION 1.0
    LANGUAGES C)

# Platform information for zenoh
if(CMAKE_SYSTEM_NAME MATCHES "Linux")
    add_definitions(-DZENOH_LINUX)
    set(JNI_PLATFORM_NAME "linux")
elseif(CMAKE_SYSTEM_NAME MATCHES "Darwin")
    add_definitions(-DZENOH_MACOS)
    set(MACOSX_RPATH "ON")
else()
    message(FATAL_ERROR "zenoh-pico is not yet available on ${CMAKE_SYSTEM_NAME} platform")
    return()
endif()

# We assume that IDLC has been installed on build machine
find_program(IDLC idlc)

if(NOT IDLC)
    message(FATAL_ERROR "IDLC not found!")
endif()

# Runs the command to get the pluginval version
execute_process(COMMAND ${IDLC} -v
    OUTPUT_VARIABLE IDLC_VERSION_RAW_OUTPUT)

string(REPLACE "idlc (Eclipse Cyclone DDS) " "" IDLC_VERSION ${IDLC_VERSION_RAW_OUTPUT})

set(IDLC_MIN_REQ "0.11.0")

if("${IDLC_VERSION}" VERSION_LESS "${IDLC_MIN_REQ}")
    message(FATAL_ERROR "IDLC is version ${IDLC_VERSION}"
        "Must be greater then ${IDLC_MIN_REQ}\n"
        "Please compile and install CycloneDDS from source")
endif()

include(FetchContent)

# CycloneDDS CDR lib (note: working with master branch since commit aa35c3c)
FetchContent_Declare(cyclonedds_cdr
    GIT_REPOSITORY "https://github.com/eclipse-cyclonedds/cyclonedds"
    GIT_TAG "origin/master"
    SOURCE_DIR "${CMAKE_CURRENT_BINARY_DIR}/cyclonedds"
    SOURCE_SUBDIR "src/core/cdr"
)
FetchContent_MakeAvailable(cyclonedds_cdr)

# IDL code generation
find_package(CycloneDDS REQUIRED)
file(GLOB IDL_FILES idl/*.idl)
idlc_generate(TARGET IdlGenerated_lib FILES ${IDL_FILES} WARNINGS no-implicit-extensibility)
include_directories(${CMAKE_BINARY_DIR})

# Zenoh-Pico lib
FetchContent_Declare(zenoh-pico
    GIT_REPOSITORY "https://github.com/eclipse-zenoh/zenoh-pico"
    GIT_TAG "origin/master"
    SOURCE_DIR "${CMAKE_CURRENT_BINARY_DIR}/zenoh-pico"
)
FetchContent_MakeAvailable(zenoh-pico)
set(ZENOHPICO_DIR ${CMAKE_CURRENT_BINARY_DIR}/zenoh-pico)
include_directories(${ZENOHPICO_DIR}/include)

# z_pub_cdr and z_sub_cdr
add_executable(z_pub_cdr z_pub_cdr.c HelloWorldData.c)
add_dependencies(z_pub_cdr zenohpico IdlGenerated_lib)
target_link_libraries(z_pub_cdr zenohpico cdr)

add_executable(z_sub_cdr z_sub_cdr.c HelloWorldData.c)
add_dependencies(z_sub_cdr zenohpico IdlGenerated_lib)
target_link_libraries(z_sub_cdr zenohpico cdr)

# CycloneDDS HelloworldPublisher and HelloworldSubscriber
add_executable(HelloworldPublisher publisher.c HelloWorldData.c)
add_dependencies(HelloworldPublisher IdlGenerated_lib)
target_link_libraries(HelloworldPublisher CycloneDDS::ddsc)

add_executable(HelloworldSubscriber subscriber.c HelloWorldData.c)
add_dependencies(HelloworldSubscriber IdlGenerated_lib)
target_link_libraries(HelloworldSubscriber CycloneDDS::ddsc)
