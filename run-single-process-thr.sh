#!/usr/bin/env bash


plog () {
   LOG_TS=`eval date "+%F-%T"`
   echo "[$LOG_TS]: $1"
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



TS=$(date +%Y%m%d.%H%M%S)

N_CPU=$(nproc)

CHAIN_LENGTH=1
BIN_DIR="./target/release/examples"
ROS_BIN_DIR="./comparison/ros/eval-ws/build"
# CYCLONEDDS_URI="./comparison/cyclonedds/cyclonedds.xml"

WD=$(pwd)


THR_DYNAMIC="thr-dynamic"
PP_ZENOH="ping-pong-zenoh"
PP_STATIC="ping-pong-static"

CDDS_COMPARISON_DIR="./comparison/cyclonedds/ping-pong/build"
ROS2_COMPARISON_DIR="./comparison/ros2/eval-ws"

ROS2_SRC="ros2 run sender_thr sender_ros_thr"
ROS2_OP="ros2 run compute_thr compute_ros_thr"
ROS2_SINK="ros2 run receiver_thr receiver_ros_thr"
PP_CDDS="ping-pong-cyclone"

ROS_SRC="sender_thr/sender_node_thr"
ROS_OP="compute_thr/compute_node_thr"
ROS_SINK="receiver_thr/receiver_node_thr"

OUT_DIR="${OUT_DIR:-breakdown-logs}"
MSGS=${MSGS:-1}
DURATION=${DURATION:-60}
SIZE=${SIZE:-8}
CPUS="${CPUS:-0,1}"
mkdir -p $OUT_DIR
NICE="${NICE:--10}"
ROS_MASTER_URI="${ROS_MASTER_URI:-http://127.0.0.1:11311}"
ROS_IP="${ROS_IP:-127.0.0.1}"
CYCLONEDDS_URI="${CYCLONEDDS_URI}"
LISTEN="${LISTEN:-tcp/127.0.0.1:7447}"
CONNECT="${LISTEN:-tcp/127.0.0.1:7887}"


# Run source by default:
# - 1 = Source
# - 2 = Operator
# - 3 = Sink
# - 4 = ROS Master (only for ROS1 tests)
TORUN=1


plog "[ INIT ] Duration will be $DURATION seconds"
plog "[ INIT ] Size for throughput test will be $SIZE"
plog "[ INIT ] ROS_IP is $ROS_IP"

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
      descriptor_file="thr-descriptor-src-op-sink-$SIZE.yaml"
      case ${TORUN} in
      1)
         plog "[ RUN ] Running Zenoh Flow source with size $SIZE"
         $BIN_DIR/$THR_DYNAMIC -s $SIZE -d $descriptor_file > /dev/null 2>&1
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$THR_DYNAMIC -r -n "src" -d $descriptor_file --listen $LISTEN --connect $CONNECT > /dev/null 2>&1
         plog "[ DONE ] Running Zenoh Flow source with size $SIZE"
         rm $descriptor_file
         ;;
      2)
         plog "[ RUN ] Running Zenoh Flow operator with size $MSGS"
         $BIN_DIR/$THR_DYNAMIC -s $SIZE -d $descriptor_file > /dev/null 2>&1
         nice $NICE taskset -c $CPUS $BIN_DIR/$THR_DYNAMIC -r -n "comp0" -d $descriptor_file --listen $LISTEN --connect $CONNECT > /dev/null 2>&1
         plog "[ DONE ] Running Zenoh Flow operator with size $MSGS"
         rm $descriptor_file
         ;;
      3)
         LOG_FILE="$OUT_DIR/zf-thr-dynamic-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE

         plog "[ RUN ] Running Zenoh Flow sink with size $SIZE logging to $LOG_FILE"
         $BIN_DIR/$THR_DYNAMIC -s $SIZE -d $descriptor_file > /dev/null 2>&1
         nice $NICE taskset -c $CPUS $BIN_DIR/$THR_DYNAMIC -r -n "snk" -d $descriptor_file --listen $LISTEN --connect $CONNECT >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running Zenoh Flow sink with size $SIZE, logged to $LOG_FILE"
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
      export CYCLONEDDS_URI=$CYCLONEDDS_URI
      source /opt/ros/galactic/setup.bash
      source $ROS2_COMPARISON_DIR/install/setup.bash


      case ${TORUN} in
      1)
         plog "[ RUN ] Running ROS2 source with size $SIZE"
         timeout $DURATION nice $NICE taskset -c $CPUS $ROS2_SRC $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running ROS2 source with size $SIZE"
         ps -ax | grep sender_ros_thr | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ;;
      2)
         plog "[ RUN ] Running ROS2 operator with size $SIZE"
         nice $NICE taskset -c $CPUS $ROS2_OP "out_0" "out_1" > /dev/null  2>&1
         plog "[ DONE ] Running ROS2 operator with size $SIZE"
         ps -ax | grep compute_ros_thr | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ;;
      3)
         LOG_FILE="$OUT_DIR/ros2-thr-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE

         plog "[ RUN ] Running ROS2 sink with size $SIZE logging to $LOG_FILE"
         nice $NICE taskset -c $CPUS $ROS2_SINK $SIZE $CHAIN_LENGTH "out_1" >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running ROS2 sink with size $SIZE, logged to $LOG_FILE"
         ps -ax | grep receiver_ros_thr | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ;;
      *)
         usage
         ;;
      esac
      unset RMW_IMPLEMENTATION
      unset ROS_LOCALHOST_ONLY
      unset CYCLONEDDS_URI
      ;;
   R)
      # ROS

      source /opt/ros/noetic/setup.bash
      export ROS_MASTER_URI=$ROS_MASTER_URI
      export ROS_IP="$ROS_IP"

      case ${TORUN} in
      1)
         plog "[ RUN ] Running ROS source withs size $SIZE"
         timeout $DURATION nice $NICE taskset -c $CPUS $ROS_BIN_DIR/$ROS_SRC $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running ROS source with size $SIZE"
         ps -ax | grep sender_node | awk {'print $1'} | xargs kill -9 > /dev/null 2>&1
         ;;
      2)
         plog "[ RUN ] Running ROS operator with size $SIZE"
         nice $NICE taskset -c $CPUS $ROS_BIN_DIR/$ROS_OP "out_0" "out_1" > /dev/null 2>&1
         plog "[ DONE ] Running ROS operator with size $SIZE"
         ps -ax | grep compute_node | awk {'print $1'} | xargs kill -9 > /dev/null 2>&1
         ;;
      3)
         LOG_FILE="$OUT_DIR/ros-thr-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE

         plog "[ RUN ] Running ROS sink with size $SIZE logging to $LOG_FILE"
         nice $NICE taskset -c $CPUS $ROS_BIN_DIR/$ROS_SINK $SIZE $CHAIN_LENGTH "out_1" >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running ROS sink with size $SIZE, logged to $LOG_FILE"
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