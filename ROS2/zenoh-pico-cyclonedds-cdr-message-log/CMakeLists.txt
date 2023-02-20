cmake_minimum_required(VERSION 3.19)

# This is your project statement. You should always list languages;
# Listing the version is nice here since it sets lots of useful variables
project(
  zenoh-ros2
  VERSION 1.0
  LANGUAGES C)

# Platform information for zenoh
if(CMAKE_SYSTEM_NAME MATCHES "Linux")
  add_definitions(-DZENOH_LINUX)
  set(JNI_PLATFORM_NAME "linux")
else()
  message(FATAL_ERROR "zenoh-pico is not yet available on ${CMAKE_SYSTEM_NAME} platform")
  return()
endif()

if(DEFINED ENV{ROS_DISTRO})
  message(FATAL_ERROR "ROS2 environment sourced, you shouldn't source ROS")
endif()

if(NOT DEFINED ROS_DISTRO)
  message(FATAL_ERROR "No ROS_DISTRO defined")
else()
  message("-- Using ROS ${ROS_DISTRO}")
endif()

# ROS Paths
set(ROS_PATH "/opt/ros/${ROS_DISTRO}/share")
set(RCL_INTERFACES_PATH "${ROS_PATH}/rcl_interfaces")
set(BUILTIN_INTERFACES_PATH "${ROS_PATH}/builtin_interfaces")

# CycloneDDS config

set(BUILD_IDLC_ONLY YES)

# CycloneDDS IDL
include(FetchContent)
FetchContent_Declare(cyclonedds
  GIT_REPOSITORY "https://github.com/eclipse-cyclonedds/cyclonedds"
  GIT_TAG "origin/master"
  SOURCE_DIR "${CMAKE_CURRENT_BINARY_DIR}/cyclonedds"
)
FetchContent_MakeAvailable(cyclonedds)

set(CYCLONEDDS_DIR ${CMAKE_CURRENT_BINARY_DIR}/cyclonedds)
include("${CYCLONEDDS_DIR}/cmake/Modules/Generate.cmake")

include("${RCL_INTERFACES_PATH}/cmake/rosidl_cmake-extras.cmake")
include("${BUILTIN_INTERFACES_PATH}/cmake/rosidl_cmake-extras.cmake")

foreach(_idl ${rcl_interfaces_IDL_FILES})
  list(APPEND IDL_FILES "${RCL_INTERFACES_PATH}/${_idl}")
endforeach()

foreach(_idl ${builtin_interfaces_IDL_FILES})
  list(APPEND IDL_FILES "${BUILTIN_INTERFACES_PATH}/${_idl}")
endforeach()

idlc_generate(TARGET rcl_interfaces_msgs FILES ${IDL_FILES} INCLUDES ${ROS_PATH} BASE_DIR ${ROS_PATH} WARNINGS no-implicit-extensibility)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
include_directories (${CMAKE_BINARY_DIR})
include_directories (${CYCLONEDDS_DIR}/src/core/cdr/include)
include_directories (${CYCLONEDDS_DIR}/src/ddsrt/include)
include_directories (${CYCLONEDDS_DIR}/src/core/ddsc/include)
include_directories (${cyclonedds_BINARY_DIR}/src/core/include/)
include_directories (${cyclonedds_BINARY_DIR}/src/ddsrt/include/)

# Zenoh-Pico
include(FetchContent)
FetchContent_Declare(zenoh-pico
  GIT_REPOSITORY "https://github.com/eclipse-zenoh/zenoh-pico"
  GIT_TAG "origin/master"
  SOURCE_DIR "${CMAKE_CURRENT_BINARY_DIR}/zenoh-pico"
)
FetchContent_MakeAvailable(zenoh-pico)
set(ZENOHPICO_DIR ${CMAKE_CURRENT_BINARY_DIR}/zenoh-pico)
include_directories (${ZENOHPICO_DIR}/include)

# Adding something we can run - Output name matches target name
add_executable(z_pub_ros2
               z_pub_ros2.c
               hal/heap.c
               hal/log.c
               rcl_interfaces/msg/Log.c
               builtin_interfaces/msg/Time.c
               ${CYCLONEDDS_DIR}/src/core/cdr/src/dds_cdrstream.c
               ${CYCLONEDDS_DIR}/src/ddsrt/src/bswap.c)
add_dependencies(z_pub_ros2 zenohpico)

add_executable(z_sub_ros2
               z_sub_ros2.c
               hal/heap.c
               hal/log.c
               rcl_interfaces/msg/Log.c
               builtin_interfaces/msg/Time.c
               ${CYCLONEDDS_DIR}/src/core/cdr/src/dds_cdrstream.c
               ${CYCLONEDDS_DIR}/src/ddsrt/src/bswap.c)
add_dependencies(z_sub_ros2 zenohpico)

set(CMAKE_CXX_IMPLICIT_LINK_DIRECTORIES /usr/local/lib ${CMAKE_CXX_IMPLICIT_LINK_DIRECTORIES})
target_compile_definitions(z_sub_ros2 PRIVATE "DDS_LOG=0")
target_compile_definitions(z_pub_ros2 PRIVATE "DDS_LOG=0")
target_link_libraries(z_sub_ros2 zenohpico)
target_link_libraries(z_pub_ros2 zenohpico)
