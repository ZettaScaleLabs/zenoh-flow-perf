#!/usr/bin/env bash


plog () {
TS=`eval date "+%F-%T"`
   echo "[$TS]: $1"
}

usage() { printf "Usage: $0 \n\t
   -i source\n\t
   -o dynamic\n\t
   -e sink\n\t
   -m ros_master\n\t
   -z Zenoh Flow\n\t
   -r ROS2\n\t
   -R ROS\n" 1>&2; exit 1; }



if [[ ! $@ =~ ^\-.+ ]]
then
  usage;
fi



TS=$(date "+%F-%T")

N_CPU=$(nproc)

CHAIN_LENGTH=1
BIN_DIR="./target/release/examples"
ROS_BIN_DIR="./comparison/ros/eval-ws/build"
CYCLONEDDS_URI="./comparison/cyclonedds/cyclonedds.xml"

WD=$(pwd)

LAT_FLUME="lat-flume"
LAT_LINK="lat-link"
LAT_SRC_SNK_STATIC="lat-source-sink-static"
LAT_SRC_OP_STATIC="lat-source-op-static"
LAT_SRC_SNK_DYNAMIC="lat-source-sink-dynamic"
LAT_SRC_OP_DYNAMIC="lat-source-op-dynamic"
LAT_ZENOH="lat-zenoh"
LAT_STATIC="lat-static"
LAT_DYNAMIC="lat-dynamic"
PP_ZENOH="ping-pong-zenoh"
PP_STATIC="ping-pong-static"

CDDS_COMPARISON_DIR="./comparison/cyclonedds/ping-pong/build"
ROS2_COMPARISON_DIR="./comparison/ros2/eval-ws"

ROS2_SRC="ros2 run sender sender_ros"
ROS2_OP="ros2 run compute compute_ros"
ROS2_SINK="ros2 run receiver receiver_ros"
PP_CDDS="ping-pong-cyclone"

ROS_SRC="sender/sender_node"
ROS_OP="compute/compute_node"
ROS_SINK="receiver/receiver_node"

OUT_DIR="${OUT_DIR:-breakdown-logs}"
MSGS=${MSGS:-1}
DURATION=${DURATION:-60}
SIZE=${SIZE:-8}
CPUS="${CPUS:-0,1}"
mkdir -p $OUT_DIR
NICE="${NICE:--10}"
ROS_MASTER_URI="${ROS_MASTER_URI:-http://127.0.0.1:11311}"
# Run source by default:
# - 1 = Source
# - 2 = Operator
# - 3 = Sink
# - 4 = ROS Master (only for ROS1 tests)
TORUN=1


plog "[ INIT ] Duration will be $DURATION seconds"
plog "[ INIT ] Sending rate will be $MSGS msg/s"
plog "[ INIT ] Size for throughput test will be $FINAL_SIZE"
while getopts "ioemzrR" arg; do
   case ${arg} in
   h)
      usage
      ;;
   i)
      # Start Source

      plog "[ INIT ] Running a source"
      TORUN=1
      ;;
   o)
      # Start operator

      plog "[ INIT ] Running an operator"
      TORUN=2
      ;;
   e)
      # Start sink

      plog "[ INIT ] Running a sink"
      TORUN=3
      ;;
   m)
      # Start the ROS master
      plog "[ INIT ] Running the rosmaster"
      TORUN=4
      ;;
   z)
      # Zenoh Flow
      case ${TORUN} in
      1)
         plog "[ RUN ] Running Zenoh Flow source with msg/s $MSGS"
         $BIN_DIR/$LAT_DYNAMIC --ping -m $MSGS -d $descriptor_file > /dev/null 2>&1
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$LAT_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1
         plog "[ DONE ] Running Zenoh Flow source with msg/s $MSGS"
         rm $descriptor_file
         ;;
      2)
         plog "[ RUN ] Running Zenoh Flow operator with msg/s $MSGS"
         $BIN_DIR/$LAT_DYNAMIC --ping -m $MSGS -d $descriptor_file > /dev/null 2>&1
         nice $NICE taskset -c $CPUS $BIN_DIR/$LAT_DYNAMIC -r -n "comp0" -d $descriptor_file > /dev/null 2>&1
         plog "[ DONE ] Running Zenoh Flow operator with msg/s $MSGS"
         rm $descriptor_file
         ;;
      3)
         LOG_FILE="$OUT_DIR/zf-src-op-sink-dynamic-pp-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE

         plog "[ RUN ] Running Zenoh Flow sink with msg/s $MSGS logging to $LOG_FILE"
         $BIN_DIR/$LAT_DYNAMIC --ping -m $MSGS -d $descriptor_file > /dev/null 2>&1
         nice $NICE taskset -c $CPUS $BIN_DIR/$LAT_DYNAMIC -r -n "snk" -d $descriptor_file >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running Zenoh Flow sink with msg/s $MSGS, logged to $LOG_FILE"
         rm $descriptor_file
         ;;
      *)
         usage
         ;;
      esac
      ;;
   r)
      # ROS2

      export RMW_IMPLEMENTATION=rmw_cyclonedds_cpp
      export ROS_LOCALHOST_ONLY=1
      source /opt/ros/galactic/setup.bash
      source $ROS2_COMPARISON_DIR/install/setup.bash

      case ${TORUN} in
      1)
         plog "[ RUN ] Running ROS2 source with msg/s $MSGS"
         timeout $DURATION nice $NICE taskset -c $CPUS $ROS2_SRC $MSGS > /dev/null 2>&1
         plog "[ DONE ] Running ROS2 source with msg/s $MSGS"
         ps -ax | grep sender_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ;;
      2)
         plog "[ RUN ] Running ROS2 operator with msg/s $MSGS"
         nice $NICE taskset -c $CPUS $ROS2_OP "out_0" "out_1" > /dev/null 2>&1
         plog "[ DONE ] Running ROS2 operator with msg/s $MSGS"
         ps -ax | grep compute_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ;;
      3)
         LOG_FILE="$OUT_DIR/ros2-lat-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE

         plog "[ RUN ] Running ROS2 sink with msg/s $MSGS logging to $LOG_FILE"
         nice $NICE taskset -c $CPUS $ROS2_SINK $s $CHAIN_LENGTH "out_1" >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running ROS2 sink with msg/s $MSGS, logged to $LOG_FILE"
         ps -ax | grep receiver_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ;;
      *)
         usage
         ;;
      esac
      unset RMW_IMPLEMENTATION
      unset ROS_LOCALHOST_ONLY
      ;;
   R)
      # ROS

      source /opt/ros/noetic/setup.bash
      export ROS_MASTER_URI=$ROS_MASTER_URI

      case ${TORUN} in
      1)
         plog "[ RUN ] Running ROS source with msg/s $MSGS"
         timeout $DURATION nice $NICE taskset -c $CPUS $ROS_BIN_DIR/$ROS_SRC $MSGS > /dev/null 2>&1
         plog "[ DONE ] Running ROS source with msg/s $MSGS"
         ps -ax | grep sender_node | awk {'print $1'} | xargs kill -9 > /dev/null 2>&1
         ;;
      2)
         plog "[ RUN ] Running ROS operator with msg/s $MSGS"
         nice $NICE taskset -c $CPUS $ROS_BIN_DIR/$ROS_OP "out_0" "out_1" > /dev/null 2>&1
         plog "[ DONE ] Running ROS operator with msg/s $MSGS"
         ps -ax | grep compute_node | awk {'print $1'} | xargs kill -9 > /dev/null 2>&1
         ;;
      3)
         LOG_FILE="$OUT_DIR/ros-lat-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE

         plog "[ RUN ] Running ROS sink with msg/s $MSGS logging to $LOG_FILE"
         nice $NICE taskset -c $CPUS $ROS_BIN_DIR/$ROS_SINK $s $CHAIN_LENGTH "out_1" >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running ROS sink with msg/s $MSGS, logged to $LOG_FILE"
         ps -ax | grep receiver_node | awk {'print $1'} | xargs kill -9 > /dev/null 2>&1
         ;;
      4)
         plog "[ RUN ] Running ROS master"
         nice $NICE taskset -c $CPUS roscore -p 11311
         ;;
      *)
         usage
         ;;
      esac
      unset ROS_MASTER_URI
      ;;
   *)
      usage
      ;;
   esac
done
``
plog "Bye!"