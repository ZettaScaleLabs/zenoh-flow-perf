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

cmake_minimum_required(VERSION 3.0.2)
project(compute_thr)

# Default to C++17
if(NOT CMAKE_CXX_STANDARD)
  set(CMAKE_CXX_STANDARD 17)
endif()



find_package(catkin REQUIRED COMPONENTS
  eval_interfaces REQUIRED
  roscpp REQUIRED
)
find_package(argparse REQUIRED)

add_executable(compute_node_thr src/node_main.cpp src/compute.cpp )

include_directories(
  include
  ${catkin_INCLUDE_DIRS}
)

target_link_libraries(compute_node_thr ${catkin_LIBRARIES})

install(TARGETS compute_node_thr
  RUNTIME DESTINATION ${CATKIN_PACKAGE_BIN_DESTINATION})