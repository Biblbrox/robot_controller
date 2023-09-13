cmake_minimum_required(VERSION 3.5)

if (NOT CMAKE_VERSION VERSION_LESS 3.0)
    cmake_policy(SET CMP0048 NEW)
endif ()

set(PROJECT_NAME nodegraph)
project(${PROJECT_NAME} CXX C)

#################################################
# compiler settings
#################################################

# Default to C11
if (NOT CMAKE_C_STANDARD)
    set(CMAKE_C_STANDARD 11)
endif ()

# Default to C++17
if (NOT CMAKE_CXX_STANDARD)
    set(CMAKE_CXX_STANDARD 17)
endif ()

if (CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    add_compile_options(-Wall -Wextra -Wpedantic)
endif ()

set(CMAKE_VERBOSE_MAKEFILE ON)
set(BUILD_EXECUTABLE OFF)

#find_package(ament_cmake_ros REQUIRED)
#find_package(rcl REQUIRED)
#find_package(rcl_lifecycle REQUIRED)
#find_package(rclc REQUIRED)
find_package(rclcpp REQUIRED)
#find_package(rclc_lifecycle REQUIRED)
find_package(std_msgs REQUIRED)
find_package(sensor_msgs REQUIRED)
#find_package(ros2bag_cpp REQUIRED)
#find_package(lifecycle_msgs REQUIRED)
#find_package(example_interfaces REQUIRED)
#find_package(rclc_parameter REQUIRED)
#find_package(Threads REQUIRED)
# Find requirements
if (NOT fastcdr_FOUND)
    find_package(fastcdr REQUIRED)
endif ()

if (NOT fastrtps_FOUND)
    find_package(fastrtps REQUIRED)
endif ()

include(CheckCXXCompilerFlag)
#include_directories(/opt/ros/humble/include/fastrtps)

FILE(GLOB_RECURSE SOURCES RELATIVE ${CMAKE_SOURCE_DIR} "src/*.cpp" "src/*.hpp" "src/*.h")


if (BUILD_EXECUTABLE)
    message("Building executable")
    add_executable(${PROJECT_NAME} ${SOURCES})
else ()
    message("Building library")
    list(REMOVE_ITEM SOURCES "src/main.cpp")
    add_library(${PROJECT_NAME} SHARED ${SOURCES})
endif ()

set(ROS_VERSION humble)
target_include_directories(${PROJECT_NAME} PRIVATE /opt/ros/${ROS_VERSION}/include/fastrtps /opt/ros/${ROS_VERSION}/include/ /opt/ros/${ROS_VERSION}/include/rclcpp)
target_link_libraries(${PROJECT_NAME} PUBLIC fastrtps)
ament_target_dependencies(${PROJECT_NAME} PUBLIC rclcpp sensor_msgs)

ament_export_targets(${PROJECT_NAME}  HAS_LIBRARY_TARGET)
ament_export_dependencies(rclcpp sensor_msgs)


#ament_package()

install(TARGETS
        ${PROJECT_NAME}
        DESTINATION ${CMAKE_SOURCE_DIR}/lib/${PROJECT_NAME}
)

#ament_package()