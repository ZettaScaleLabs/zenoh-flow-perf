cmake_minimum_required(VERSION 3.0.2)
project(receiver_thr)

# Default to C++17
if(NOT CMAKE_CXX_STANDARD)
  set(CMAKE_CXX_STANDARD 17)
endif()



find_package(catkin REQUIRED COMPONENTS
  eval_interfaces REQUIRED
  roscpp REQUIRED
)
find_package(argparse REQUIRED)

add_executable(receiver_node_thr src/node_main.cpp src/receiver.cpp )

include_directories(
  include
  ${catkin_INCLUDE_DIRS}
)


target_link_libraries(receiver_node_thr ${catkin_LIBRARIES})
install(TARGETS receiver_node_thr
  RUNTIME DESTINATION ${CATKIN_PACKAGE_BIN_DESTINATION})