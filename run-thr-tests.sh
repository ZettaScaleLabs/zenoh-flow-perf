#!/usr/bin/env bash


plog () {
TS=`eval date "+%F-%T"`
   echo "[$TS]: $1"
}

TS=$(date "+%F-%T")

INITIAL_SIZE=8
END_SIZE=16777216 #16MB

BIN_DIR="./target/release/examples"

WD=$(pwd)

FLUME_THR="thr-flume"
LINK_THR="thr-link"
ZF_THR="thr-static"

OUT_DIR="logs"

DURATION=20


mkdir -p $OUT_DIR

plog "Running Baseline flume channel benches"
THR_FILE="$OUT_DIR/flume-thr-$TS.csv"
echo "layer,scenario,test,name,size,messages" > $THR_FILE

x=$INITIAL_SIZE
while [ $x -le $END_SIZE ]
do
   plog "flume channels for size $x"
   ASYNC_STD_THREAD_COUNT=3 timeout $DURATION taskset -c 0,1,2 $BIN_DIR/$FLUME_THR -s $x >> $THR_FILE
   x=$(( $x * 2 ))
done

plog "Running zenoh flow channel benches"
THR_FILE="$OUT_DIR/link-thr-$TS.csv"
echo "layer,scenario,test,name,size,messages" > $THR_FILE
x=$INITIAL_SIZE
while [ $x -le $END_SIZE ]
do
   plog "Zenoh flow chennels bench for size $x"
   ASYNC_STD_THREAD_COUNT=3 timeout $DURATION taskset -c 0,1,2 $BIN_DIR/$LINK_THR -s $x >> $THR_FILE
   x=$(( $x * 2 ))
done

THR_FILE="$OUT_DIR/zf-thr-$TS.csv"
plog "Running zenoh flow dataflow benches"
echo "layer,scenario,test,name,size,messages" > $THR_FILE

x=$INITIAL_SIZE
while [ $x -le $END_SIZE ]
do
   plog "Zenoh flow bench for size $x"
   ASYNC_STD_THREAD_COUNT=3 timeout $DURATION taskset -c 0,1,2 $BIN_DIR/$ZF_THR -s $x >> $THR_FILE
   x=$(( $x * 2 ))
done





plog "Done Test results stored in $OUT_DIR"
plog "Bye!"


