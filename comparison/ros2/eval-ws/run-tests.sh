#!/usr/bin/env bash


plog () {
TS=`eval date "+%F-%T"`
   echo "[$TS]: $1"
}

TS=$(date "+%F-%T")


INITIAL_MSGS=1
#FINAL_MSGS=1
#FINAL_MSGS=1000
FINAL_MSGS=1000000

CHAIN_LENGTH=1

CHAIN_LENGTH_END=${CHAIN_LENGTH_END:-64}

source /opt/ros/foxy/setup.bash
source ./install/setup.bash

WD=$(pwd)

ROS_SRC="ros2 run sender sender_ros"
ROS_OP="ros2 run compute compute_ros"
ROS_SINK="ros2 run receiver receiver_ros"

OUT_DIR="logs"

DURATION=60

N_CPU=$(nproc)

mkdir -p $OUT_DIR

plog "Running ROS2 Latency test"
THR_FILE="$OUT_DIR/ros-lat-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $THR_FILE

length=$CHAIN_LENGTH
while  [ $length -le $CHAIN_LENGTH_END ]
do
   plog "[ RUN ] ROS for chain length $length"
   s=$INITIAL_MSGS
   while [ $s -le $FINAL_MSGS ]
   do

      ALL_PIDS=()

      plog "[ RUN ] ROS for msgs $s, chain length $length"
      pipeline="Source--->out_0"
      o=1
      while [ $o -le $length ]
      do

         pipeline="$pipeline<->out_$(($o - 1 ))--->Compute--->out_$o"


         #op_cmd="nohup taskset -c $(($o % $N_CPU)) $ROS_OP "out_$(($o - 1 ))" "out_$o"> /dev/null 2>&1 &"
         #echo "$op_cmd"

         nohup taskset -c $(($o % $N_CPU)) $ROS_OP "out_$(($o - 1 ))" "out_$o"> /dev/null 2>&1 &

         ALL_PIDS+=($!)
         o=$(( $o + 1 ))
      done

      #sender_cmd="$ROS_SRC $s > /dev/null 2>&1 &"
      #echo "Sender command is $sender_cmd"

      nohup $ROS_SRC $s > /dev/null 2>&1 &
      SRC_PID=$!
      pipeline="$pipeline<->out_$(( $o - 1 ))---><Sink"
      echo "Pipeline is: $pipeline"


      #receiver_cmd="timeout $DURATION taskset -c 2 $ROS_SINK $s $length "out_$(( $o - 1 ))" >> $THR_FILE"
      #echo "Receiver command is $receiver_cmd"

      timeout $DURATION $ROS_SINK $s $length "out_$(( $o - 1 ))" >> $THR_FILE



      kill -2 $SRC_PID
      kill -2 $SRC_PID
      kill -9 $SRC_PID

      ps -A | grep sender_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
      ps -A | grep compute_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
      ps -A | grep receiver_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

      plog "[ DONE ] ROS for msgs $s, chain length $length"
      sleep 1
      echo "Still running compute_ros: $(ps -A | grep compute_ros | wc -l) - This should be 0"
      echo "Still running receiver_ros: $(ps -A | grep receiver_ros | wc -l) - This should be 0"
      echo "Still running sender_ros: $(ps -A | grep sender_ros | wc -l) - This should be 0"

      s=$(($s * 10))

   done
   plog "[ DONE ] ROS for chain length $length"
   length=$(($length * 2))
done
TOT_SAMPLES=$(cat $THR_FILE | wc -l)

plog "Done Test results stored in $OUT_DIR - Total Samples: $TOT_SAMPLES"
plog "Bye!"


