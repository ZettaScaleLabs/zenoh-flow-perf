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
project(cyclonedds-ping-pong LANGUAGES C CXX)
cmake_minimum_required(VERSION 3.5)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

message("-- CMAKE_BUILD_TYPE = ${CMAKE_BUILD_TYPE}")

if(NOT TARGET CycloneDDS::ddsc)
  # Find the CycloneDDS package.
  find_package(CycloneDDS REQUIRED)
endif()

idlc_generate (TARGET evaluationdata FILES Evaluation.idl)
# idlcxx_generate(TARGET evaluationdata FILES Evaluation.idl)

add_executable(ping-pong-cyclone ping-pong.cpp)

# Link both executables to idl data type library and ddscxx.
target_link_libraries(ping-pong-cyclone CycloneDDS::ddsc evaluationdata)


set_property(TARGET ping-pong-cyclone PROPERTY CXX_STANDARD 17)
