cmake_minimum_required(VERSION 3.5)
project(sender_thr)


if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

# Default to C++17
if(NOT CMAKE_CXX_STANDARD)
  set(CMAKE_CXX_STANDARD 17)
endif()

if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# find dependencies
find_package(ament_cmake REQUIRED)
find_package(eval_interfaces REQUIRED)
find_package(rclcpp REQUIRED)
find_package(argparse REQUIRED)

## BUILD

include_directories(
  include
)

add_library(${PROJECT_NAME}_lib
  "src/sender.cpp"
)

set(DEPENDENCIES
  "rclcpp"
  "eval_interfaces"
)

target_link_libraries(${PROJECT_NAME}_lib)
ament_target_dependencies(${PROJECT_NAME}_lib ${DEPENDENCIES})

set(EXECUTABLE_NAME "sender_ros_thr")

add_executable(${EXECUTABLE_NAME} src/node_main.cpp)
target_link_libraries(${EXECUTABLE_NAME} ${PROJECT_NAME}_lib)
target_include_directories(${EXECUTABLE_NAME} PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<INSTALL_INTERFACE:include>)

ament_target_dependencies(${EXECUTABLE_NAME} ${DEPENDENCIES})

# INSTALL

install(TARGETS ${EXECUTABLE_NAME}
  DESTINATION lib/${PROJECT_NAME}
)

# AMENT ROS2 stuff

ament_export_include_directories(include)

ament_package()
