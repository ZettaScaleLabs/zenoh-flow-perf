#!/usr/bin/env bash


plog () {
TS=`eval date "+%F-%T"`
   echo "[$TS]: $1"
}

usage() { printf "Usage: $0 \n\t-f flume\n\t-l link\n\t-s static\n\t-d dynamic\n\t-z zenoh\n\t-c CycloneDDS\n\t-r ROS2" 1>&2; exit 1; }



if [[ ! $@ =~ ^\-.+ ]]
then
  usage;
fi



TS=$(date "+%F-%T")

N_CPU=$(nproc)

INITIAL_MSGS=1

FINAL_MSGS=1000000

CHAIN_LENGTH=1
BIN_DIR="./target/release/examples"

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

ROS_SRC="ros2 run sender sender_ros"
ROS_OP="ros2 run compute compute_ros"
ROS_SINK="ros2 run receiver receiver_ros"
PP_CDDS="ping-pong-cyclone"


OUT_DIR="${OUT_DIR:-breakdown-logs}"

DURATION=${DURATION:-60}

mkdir -p $OUT_DIR


plog "[ INIT ] Duration will be $DURATION seconds for each test"
while getopts "hflsdzcr" arg; do
   case ${arg} in
   h)
      usage
      ;;
   f)
      # Flume

      plog "[ START ] baseline Flume Latency test"
      LOG_FILE="$OUT_DIR/flume-$TS.csv"
      echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         plog "[ RUN ] Flume for msg/s $s"

         timeout $DURATION taskset -c 0,1 $BIN_DIR/$LAT_FLUME -m $s >> $LOG_FILE

         ps -ax | grep $LAT_FLUME | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] Flume for msg/s $s"
         sleep 1
         echo "Still running $LLAT_FLUMEAT: $(ps -ax | grep $LAT_FLUME | wc -l) - This should be 0"
         s=$(($s * 10))

      done
      plog "[ END ] baseline Flume Latency test"
      ;;
   l)
      # ZF Link

      plog "[ START ] baseline ZF Link Latency test"
      LOG_FILE="$OUT_DIR/zf-link-$TS.csv"
      echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         plog "[ RUN ] ZF Link for msg/s $s"

         timeout $DURATION taskset -c 0,1 $BIN_DIR/$LAT_LINK -m $s >> $LOG_FILE

         ps -ax | grep $LAT_LINK | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] ZF Link for msg/s $s"
         sleep 1
         echo "Still running $LAT_LINK: $(ps -ax | grep $LAT_LINK | wc -l) - This should be 0"
         s=$(($s * 10))

      done
      plog "[ END ] baseline ZF Link Latency test"
      ;;
   s)
      # Static Source->Op->Sink

      plog "[ START ] Zenoh Flow Source->Operator->Sink Static Latency w/ Ping test"
      LOG_FILE="$OUT_DIR/zf-src-op-sink-static-pp-$TS.csv"
      echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         plog "[ RUN ] Zenoh Flow Source->Operator->Sink Static for msg/s $s w/ Ping"

         timeout $DURATION taskset -c 0,1,2,3 $BIN_DIR/$PP_STATIC -m $s -p >> $LOG_FILE

         ps -ax | grep $LAT_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] Zenoh Flow Source->Operator->Sink Static for msg/s $s  w/ Ping"
         sleep 1
         echo "Still running $LAT_STATIC: $(ps -ax | grep $LAT_STATIC | wc -l) - This should be 0"
         rm $descriptor_file
         s=$(($s * 10))

      done
      plog "[ END ] Zenoh Flow Source->Operator->Sink Static Latency w/ Ping test"


      ;;
   d)
      # Dynamic Source->Op->Sink

      plog "[ START ] Zenoh Flow Source->Operator->Sink Dynamic Latency w/ Ping test"
      LOG_FILE="$OUT_DIR/zf-src-op-sink-dynamic-pp-$TS.csv"
      echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         plog "[ RUN ] Zenoh Flow Source->Operator->Sink Dynamic for msg/s $s"

         descriptor_file="descriptor-src-op-sink-$s.yaml"

         $BIN_DIR/$LAT_DYNAMIC --ping -m $s -d $descriptor_file > /dev/null 2>&1

         nohup taskset -c 0,1 $BIN_DIR/$LAT_DYNAMIC -r -n "comp0" -d $descriptor_file > /dev/null 2>&1 &
         nohup taskset -c 2,3 $BIN_DIR/$LAT_DYNAMIC -r -n "snk" -d $descriptor_file >> $LOG_FILE 2> /dev/null &

         sleep 3

         timeout $DURATION taskset -c 4,5 $BIN_DIR/$LAT_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1

         ps -ax | grep $LAT_DYNAMIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] Zenoh Flow Source->Operator->Sink Dynamic for msg/s $s w/ Ping"
         sleep 1
         echo "Still running $LAT_DYNAMIC: $(ps -ax | grep $LAT_DYNAMIC | wc -l) - This should be 0"
         rm $descriptor_file
         s=$(($s * 10))

      done
      plog "[ END ] Zenoh Flow Source->Operator->Sink Dynamic Latency w/ Ping test"
      ;;
   z)
      # Zenoh ping

      plog "[ START ] baseline Zenoh Latency test"
      LOG_FILE="$OUT_DIR/zenoh-pp-$TS.csv"
      echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         plog "[ RUN ] baseline Zenoh for msg/s $s"

         nohup taskset -c 0 $BIN_DIR/$PP_ZENOH -m $s >> $LOG_FILE 2> /dev/null &

         sleep 3

         timeout $DURATION taskset -c 1 $BIN_DIR/$PP_ZENOH -p -m $s > /dev/null 2>&1

         ps -ax | grep $PP_ZENOH | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] baseline Zenoh for msg/s $s"
         sleep 1
         echo "Still running $PP_ZENOH: $(ps -ax | grep $PP_ZENOH | wc -l) - This should be 0"

         s=$(($s * 10))
      done
      plog "[ END ] baseline Zenoh Latency test"
      ;;
   c)
      # CycloneDDS

      plog "[ START ] baseline CycloneDDS Latency test"
      LOG_FILE="$OUT_DIR/cyclone-lat-$TS.csv"
      echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         plog "[ RUN ] baseline CycloneDDS for msg/s $s"

         nohup taskset -c 0 $CDDS_COMPARISON_DIR/$PP_CDDS $s >> $LOG_FILE 2> /dev/null &

         sleep 3

         timeout $DURATION taskset -c 1 $CDDS_COMPARISON_DIR/$PP_CDDS $s --ping > /dev/null 2>&1

         ps -ax | grep $PP_CDDS | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] baseline CycloneDDS for msg/s $s"
         sleep 1
         echo "Still running $PP_CDDS: $(ps -ax | grep $PP_CDDS | wc -l) - This should be 0"

         s=$(($s * 10))
      done
      plog "[ END ] baseline CycloneDDS Latency test"
      ;;
   r)
      # ROS2

      plog "[ START ] ROS2 Latency test"
      LOG_FILE="$OUT_DIR/ros2-lat-$TS.csv"
      echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
      export ROS_LOCALHOST_ONLY=1
      source /opt/ros/foxy/setup.bash
      source $ROS2_COMPARISON_DIR/install/setup.bash

      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         plog "[ RUN ] ROS2 for msg/s $s"


         nohup taskset -c 0,1 $ROS_SINK $s 1 "out_1" >> $LOG_FILE 2> /dev/null &
         sleep 3
         nohup taskset -c 2,3 $ROS_OP "out_0" "out_1" > /dev/null 2>&1 &

         sleep 3

         timeout $DURATION taskset -c 4,5 $ROS_SRC $s #> /dev/null 2>&1

         ps -ax | grep compute_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ps -ax | grep receiver_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1
         ps -ax | grep sender_ros | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1


         plog "[ DONE ] ROS2 for msg/s $s w/ Ping"
         sleep 1
         echo "Still running compute_ros: $(ps -A | grep compute_ros | wc -l) - This should be 0"
         echo "Still running receiver_ros: $(ps -A | grep receiver_ros | wc -l) - This should be 0"
         echo "Still running sender_ros: $(ps -A | grep sender_ros | wc -l) - This should be 0"
         s=$(($s * 10))

      done
      plog "[ END ] ROS2 Latency test"
      ;;
   *)
      usage
      ;;
   esac
done

plog "Bye!"


# # Static Source->Sink

# plog "[ START ] Zenoh Flow Source->Sink Static Latency test"
# LOG_FILE="$OUT_DIR/zf-src-snk-static-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] Zenoh Flow Source->Sink Static for msg/s $s"

#    timeout $DURATION taskset -c 0,1,2 $BIN_DIR/$LAT_SRC_SNK_STATIC -m $s >> $LOG_FILE

#    ps -ax | grep $LAT_SRC_SNK_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] Zenoh Flow Source->Sink Static for msg/s $s"
#    sleep 1
#    echo "Still running $LAT_SRC_SNK_STATIC: $(ps -ax | grep $LAT_SRC_SNK_STATIC | wc -l) - This should be 0"
#    s=$(($s * 10))

# done
# plog "[ END ] Zenoh Flow Source->Sink Static Latency test"

# # Static Source->Op

# plog "[ START ] Zenoh Flow Source->Operator Static Latency test"
# LOG_FILE="$OUT_DIR/zf-src-op-static-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] Zenoh Flow Source->Operator Static for msg/s $s"

#    timeout $DURATION taskset -c 0,1,2 $BIN_DIR/$LAT_SRC_OP_STATIC -m $s >> $LOG_FILE

#    ps -ax | grep $LAT_SRC_OP_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] Zenoh Flow Source->Operator Static for msg/s $s"
#    sleep 1
#    echo "Still running $LAT_SRC_OP_STATIC: $(ps -ax | grep $LAT_SRC_OP_STATIC | wc -l) - This should be 0"
#    s=$(($s * 10))

# done
# plog "[ END ] Zenoh Flow Source->Operator Static Latency test"


# # Dynamic Source->Sink

# plog "[ START ] Zenoh Flow Source->Sink Dynamic Latency test"
# LOG_FILE="$OUT_DIR/zf-src-snk-dynamic-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] Zenoh Flow Source->Sink Dynamic for msg/s $s"

#    descriptor_file="descriptor-src-snk-$s.yaml"

#    $BIN_DIR/$LAT_SRC_SNK_DYNAMIC -m $s -d $descriptor_file > /dev/null 2>&1

#    nohup taskset -c 0,1 $BIN_DIR/$LAT_SRC_SNK_DYNAMIC -r -n "snk" -d $descriptor_file  >> $LOG_FILE 2> /dev/null &

#    timeout $DURATION taskset -c 2,3 $BIN_DIR/$LAT_SRC_SNK_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1

#    ps -ax | grep $LAT_SRC_SNK_DYNAMIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] Zenoh Flow Source->Sink Dynamic for msg/s $s"
#    sleep 1
#    echo "Still running $LAT_SRC_SNK_DYNAMIC: $(ps -ax | grep $LAT_SRC_SNK_DYNAMIC | wc -l) - This should be 0"
#    rm $descriptor_file
#    s=$(($s * 10))
# done

# # sed -i -e 's/zenoh-flow-multi/zf-source-sink-multi/g' $LOG_FILE
# # plog "[ END ] Zenoh Flow Source->Sink Dynamic Latency test"

# # Dynamic Source->Op

# plog "[ START ] Zenoh Flow Source->Operator Dynamic Latency test"
# LOG_FILE="$OUT_DIR/zf-src-op-dynamic-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] Zenoh Flow Source->Operator Dynamic for msg/s $s"

#    descriptor_file="descriptor-src-op-$s.yaml"

#    $BIN_DIR/$LAT_SRC_OP_DYNAMIC -m $s -d $descriptor_file > /dev/null 2>&1

#    nohup taskset -c 0,1 $BIN_DIR/$LAT_SRC_OP_DYNAMIC -r -n "comp" -d $descriptor_file >> $LOG_FILE 2> /dev/null &
#    nohup taskset -c 2,3 $BIN_DIR/$LAT_SRC_OP_DYNAMIC -r -n "snk" -d $descriptor_file > /dev/null 2>&1 &

#    timeout $DURATION taskset -c 2 $BIN_DIR/$LAT_SRC_OP_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1

#    ps -ax | grep $LAT_SRC_OP_DYNAMIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] Zenoh Flow Source->Operator Dynamic for msg/s $s"
#    sleep 1
#    echo "Still running $LAT_SRC_OP_DYNAMIC: $(ps -ax | grep $LAT_SRC_OP_DYNAMIC | wc -l) - This should be 0"
#    rm $descriptor_file
#    s=$(($s * 10))

# done
# plog "[ END ] Zenoh Flow Source->Operator Dynamic Latency test"


# # Static Source->Op->Sink

# plog "[ START ] Zenoh Flow Source->Operator->Sink Static Latency test"
# LOG_FILE="$OUT_DIR/zf-src-op-sink-static-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] Zenoh Flow Source->Operator->Sink Static for msg/s $s"

#    timeout $DURATION taskset -c 0,1,2,3 $BIN_DIR/$LAT_STATIC -m $s -p >> $LOG_FILE

#    ps -ax | grep $LAT_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] Zenoh Flow Source->Operator->Sink Static for msg/s $s"
#    sleep 1
#    echo "Still running $LAT_STATIC: $(ps -ax | grep $LAT_STATIC | wc -l) - This should be 0"
#    rm $descriptor_file
#    s=$(($s * 10))

# done
# plog "[ END ] Zenoh Flow Source->Operator->Sink Static Latency test"


# # Dynamic Source->Op->Sink

# # plog "[ START ] Zenoh Flow Source->Operator->Sink Dynamic Latency test"
# # LOG_FILE="$OUT_DIR/zf-src-op-sink-dynamic-$TS.csv"
# # echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# # s=$INITIAL_MSGS
# # while [ $s -le $FINAL_MSGS ]
# # do
# #    plog "[ RUN ] Zenoh Flow Source->Operator->Sink Dynamic for msg/s $s"

# #    descriptor_file="descriptor-src-op-sink-$s.yaml"

# #    $BIN_DIR/$LAT_DYNAMIC -m $s -d $descriptor_file > /dev/null 2>&1

# #    nohup taskset -c 0,1 $BIN_DIR/$LAT_DYNAMIC -r -n "comp0" -d $descriptor_file > /dev/null 2>&1 &
# #    nohup taskset -c 2,3 $BIN_DIR/$LAT_DYNAMIC -r -n "snk" -d $descriptor_file >> $LOG_FILE 2> /dev/null &

# #    timeout $DURATION taskset -c 4,5 $BIN_DIR/$LAT_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1

# #    ps -ax | grep $LAT_DYNAMIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

# #    plog "[ DONE ] Zenoh Flow Source->Operator->Sink Dynamic for msg/s $s"
# #    sleep 1
# #    echo "Still running $LAT_DYNAMIC: $(ps -ax | grep $LAT_DYNAMIC | wc -l) - This should be 0"
# #    rm $descriptor_file
# #    s=$(($s * 10))

# # done
# # plog "[ END ] Zenoh Flow Source->Operator->Sink Dynamic Latency test"





# # Zenoh

# plog "[ START ] baseline Zenoh Latency test"
# LOG_FILE="$OUT_DIR/zenoh-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] baseline Zenoh for msg/s $s"

#    nohup taskset -c 0 $BIN_DIR/$LAT_ZENOH -m $s >> $LOG_FILE 2> /dev/null &

#    timeout $DURATION taskset -c 1 $BIN_DIR/$LAT_ZENOH -p -m $s > /dev/null 2>&1

#    ps -ax | grep $LAT_ZENOH | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] baseline Zenoh for msg/s $s"
#    sleep 1
#    echo "Still running $LAT_ZENOH: $(ps -ax | grep $LAT_ZENOH | wc -l) - This should be 0"

#    s=$(($s * 10))
# done
# plog "[ END ] baseline Zenoh Latency test"

# # Zenoh UDP

# plog "[ START ] baseline Zenoh UDP Latency test"
# LOG_FILE="$OUT_DIR/zenoh-udp-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] baseline Zenoh UDP for msg/s $s"

#    nohup taskset -c 0 $BIN_DIR/$LAT_ZENOH -u -m $s >> $LOG_FILE 2> /dev/null &

#    timeout $DURATION taskset -c 1 $BIN_DIR/$LAT_ZENOH -u -p -m $s > /dev/null 2>&1

#    ps -ax | grep $LAT_ZENOH | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] baseline Zenoh UDP for msg/s $s"
#    sleep 1
#    echo "Still running $LAT_ZENOH: $(ps -ax | grep $LAT_ZENOH | wc -l) - This should be 0"

#    s=$(($s * 10))
# done
# plog "[ END ] baseline Zenoh UDP Latency test"


# # Zenoh UDP ping

# plog "[ START ] baseline Zenoh UDP Latency test"
# LOG_FILE="$OUT_DIR/zenoh-udp-pp-$TS.csv"
# echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
# s=$INITIAL_MSGS
# while [ $s -le $FINAL_MSGS ]
# do
#    plog "[ RUN ] baseline Zenoh UDP for msg/s $s"

#    nohup taskset -c 1 $BIN_DIR/$PP_ZENOH -u -m $s >> $LOG_FILE 2> /dev/null &

#    sleep 3

#    timeout $DURATION taskset -c 0 $BIN_DIR/$PP_ZENOH -u -p -m $s > /dev/null 2>&1

#    ps -ax | grep $PP_ZENOH | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

#    plog "[ DONE ] baseline Zenoh UDP for msg/s $s"
#    sleep 1
#    echo "Still running $PP_ZENOH: $(ps -ax | grep $PP_ZENOH | wc -l) - This should be 0"

#    s=$(($s * 10))
# done
# plog "[ END ] baseline Zenoh UDP Latency test"



