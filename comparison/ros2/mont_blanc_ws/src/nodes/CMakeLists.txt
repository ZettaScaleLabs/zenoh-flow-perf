#
# Copyright(c) 2020 ADLINK Technology Limited and others
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License v. 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Eclipse Distribution License
# v. 1.0 which is available at
# http://www.eclipse.org/org/documents/edl-v10.php.
#
# SPDX-License-Identifier: EPL-2.0 OR BSD-3-Clause
#
cmake_minimum_required(VERSION 3.5)
project(nodes)


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
find_package(std_msgs REQUIRED)
find_package(rclcpp REQUIRED)
find_package(geometry_msgs REQUIRED)
find_package(sensor_msgs REQUIRED)


include(FetchContent)
FetchContent_Declare(
    argparse
    GIT_REPOSITORY https://github.com/p-ranav/argparse.git
)
FetchContent_MakeAvailable(argparse)


## BUILD

include_directories(
  include
)

add_library(${PROJECT_NAME}_lib
  "src/nodes.cpp"
)

set(DEPENDENCIES
  "rclcpp"
  "std_msgs"
  "geometry_msgs"
  "sensor_msgs"
)

target_link_libraries(${PROJECT_NAME}_lib)
ament_target_dependencies(${PROJECT_NAME}_lib ${DEPENDENCIES})


function(setup NAME SOURCES)

add_executable(${NAME} ${SOURCES})
target_link_libraries(${NAME} ${PROJECT_NAME}_lib)
target_include_directories(${NAME} PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<INSTALL_INTERFACE:include>)

ament_target_dependencies(${NAME} ${DEPENDENCIES})

endfunction(setup)

setup(arequipa src/arequipa_node.cpp)
setup(barcelona src/barcelona_node.cpp)
setup(cordoba src/cordoba_node.cpp)
setup(delhi src/delhi_node.cpp)
setup(freeport src/freeport_node.cpp)
setup(geneva src/geneva_node.cpp)
setup(georgetown src/georgetown_node.cpp)
setup(hamburg src/hamburg_node.cpp)
setup(hebron src/hebron_node.cpp)
setup(kingston src/kingston_node.cpp)
setup(lyon src/lyon_node.cpp)
setup(madellin src/madellin_node.cpp)
setup(monaco src/monaco_node.cpp)
setup(osaka src/osaka_node.cpp)
setup(ponce src/ponce_node.cpp)
setup(portsmouth src/portsmouth_node.cpp)
setup(rotterdam src/rotterdam_node.cpp)
setup(taipei src/taipei_node.cpp)
setup(tripoli src/tripoli_node.cpp)


target_link_libraries(arequipa argparse)
# target_link_libraries(arequipa ${PROJECT_NAME}_lib)
# target_include_directories(arequipa PUBLIC
#   $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
#   $<INSTALL_INTERFACE:include>)

# ament_target_dependencies(arequipa ${DEPENDENCIES})

# INSTALL

install(TARGETS arequipa barcelona cordoba delhi freeport geneva georgetown hamburg hebron kingston lyon madellin monaco osaka ponce portsmouth rotterdam taipei tripoli
  DESTINATION lib/${PROJECT_NAME}
)

# AMENT ROS2 stuff

ament_export_include_directories(include)

ament_package()
