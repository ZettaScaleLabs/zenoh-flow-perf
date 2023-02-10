#!/usr/bin/env bash


plog () {
   LOG_TS=`eval date "+%F-%T"`
   echo "[$LOG_TS]: $1"
}

usage() { printf "Usage: $0 \n\t-h print this help\n\t-t Zenoh-Flow Througput\n\t-l Zenoh-Flow Latency\n\t-T ERDOS Througput\n\t-L ERDOS Latency\n" 1>&2; exit 1; }



if [[ ! $@ =~ ^\-.+ ]]
then
  usage;
fi



TS=$(date +%Y%m%d.%H%M%S)

N_CPU=$(nproc)

INITIAL_MSGS=1
INITIAL_SIZE=


CHAIN_LENGTH=1
BIN_DIR="./target/release/examples"
ERDOS_BIN_DIR="./comparison/erdos/rust/target/release"
WD=$(pwd)


LAT_STATIC="lat-static"
THR_STATIC="thr-static"

ERDOS_LAT="latency"
ERDOS_THR="throughput"

OUT_DIR="${OUT_DIR:-zenoh-flow-logs}"

FINAL_MSGS=${FINAL_MSGS:-1000000}
DURATION=${DURATION:-60}

INITIAL_SIZE=1
FINAL_SIZE=${FINAL_SIZE:-16777216} #16MB  #134217728} # 128MB

mkdir -p $OUT_DIR


plog "[ INIT ] Duration will be $DURATION seconds for each test"
plog "[ INIT ] Max sending rate will be $FINAL_MSGS msg/s for each test"
plog "[ INIT ] Max size for throughput tests will be $FINAL_SIZE"
while getopts "tlTL" arg; do
   case ${arg} in
   h)
      usage
      ;;
   t)
      # Zenoh-Flow Throughput

      plog "[ START ] Zenoh-Flow Throughput test"
      s=$INITIAL_SIZE
      while [ $s -le $FINAL_SIZE ]
      do
         LOG_FILE="$OUT_DIR/zenoh-flow-throughput-$s-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE
         plog "[ RUN ] Zenoh-Flow for $s bytes payload"

         taskset -c 0,1,2 $BIN_DIR/$THR_STATIC -s $s -d $DURATION >> $LOG_FILE

         ps -ax | grep $THR_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] Zenoh-Flow for $s bytes payload"
         sleep 1
         s=$(($s * 2))

      done
      plog "[ END ] Zenoh-Flow Throughput test"
      ;;
   l)
      # ZF latency

      plog "[ START ] Zenoh-Flow  Link Latency test"

      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         LOG_FILE="$OUT_DIR/zenoh-flow-latency-$s-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE
         plog "[ RUN ] Zenoh-Flow for msg/s $s"

         timeout $DURATION taskset -c 0,1 $BIN_DIR/$LAT_STATIC -m $s >> $LOG_FILE

         ps -ax | grep $LAT_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] Zenoh-Flow for msg/s $s"
         sleep 1
         s=$(($s * 10))

      done
      plog "[ END ] Zenoh-Flow Latency test"
      ;;
   T)
      # ERDOS Throughput

      plog "[ START ] ERDOS Throughput test"
      s=$INITIAL_SIZE
      while [ $s -le $FINAL_SIZE ]
      do
         LOG_FILE="$OUT_DIR/erdos-througput-$s-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE
         plog "[ RUN ] ERDOS for $s bytes payload"

         timeout $DURATION taskset -c 0,1,2 $ERDOS_BIN_DIR/$ERDOS_THR -s $s >> $LOG_FILE

         ps -ax | grep $ERDOS_THR | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] ERDOS for $s bytes payload"
         sleep 1
         s=$(($s * 2))

      done
      plog "[ END ] ERDOS Throughput test"
      ;;
   L)
      # ERDOS latency

      plog "[ START ] ERDOS  Link Latency test"

      s=$INITIAL_MSGS
      while [ $s -le $FINAL_MSGS ]
      do
         LOG_FILE="$OUT_DIR/erdos-latency-$s-$TS.csv"
         echo "framework,scenario,test,pipeline,payload,rate,value,unit" > $LOG_FILE
         plog "[ RUN ] ERDOS for msg/s $s"

         timeout $DURATION taskset -c 0,1 $ERDOS_BIN_DIR/$ERDOS_LAT -m $s >> $LOG_FILE

         ps -ax | grep $ERDOS_LAT | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

         plog "[ DONE ] ERDOS for msg/s $s"
         sleep 1
         s=$(($s * 10))

      done
      plog "[ END ] ERDOS Latency test"
      ;;
   *)
      usage
      ;;
   esac
done
``
plog "Bye!"


