cmake_minimum_required(VERSION 3.0.2)
project(compute)

# Default to C++17
if(NOT CMAKE_CXX_STANDARD)
  set(CMAKE_CXX_STANDARD 17)
endif()



find_package(catkin REQUIRED COMPONENTS
  eval_interfaces REQUIRED
  roscpp REQUIRED
)
find_package(argparse REQUIRED)

add_executable(compute_node src/node_main.cpp src/compute.cpp )

include_directories(
  include
  ${catkin_INCLUDE_DIRS}
)

target_link_libraries(compute_node ${catkin_LIBRARIES})

install(TARGETS compute_node
  RUNTIME DESTINATION ${CATKIN_PACKAGE_BIN_DESTINATION})