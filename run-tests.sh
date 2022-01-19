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
CHAIN_LENGTH_END=64
BIN_DIR="./target/release/examples"

WD=$(pwd)

LAT="lat-static"
SERDE="lat-serde"

OUT_DIR="logs"

DURATION=60

mkdir -p $OUT_DIR

plog "Running Zenoh Flow Latency test - same process"
THR_FILE="$OUT_DIR/zf-lat-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency" > $THR_FILE

length=$CHAIN_LENGTH
while  [ $length -le $CHAIN_LENGTH_END ]
do
   plog "[ RUN ] Zenoh Flow for chain length $length"
   s=$INITIAL_MSGS
   while [ $s -le $FINAL_MSGS ]
   do

      timeout $DURATION $BIN_DIR/$LAT -m $s -p $length >> $THR_FILE

      ps -A | grep $LAT | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

      plog "[ DONE ] Zenoh Flow for msgs $s, chain length $length"
      sleep 1
      echo "Still running $LAT: $(ps -A | grep $LAT | wc -l) - This should be 0"

      s=$(($s * 10))

   done
   plog "[ DONE ] Zenoh Flow for chain length $length"
   length=$(($length * 2))
done
TOT_SAMPLES=$(cat $THR_FILE | wc -l)

plog "Done Test same process results stored in $OUT_DIR - Total Samples: $TOT_SAMPLES"


plog "Running Zenoh Flow Latency test - multi process"

THR_FILE="$OUT_DIR/zf-lat-mp-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency" > $THR_FILE

DYN_LAT="lat-dynamic"
N_CPU=$(nproc)

length=$CHAIN_LENGTH
while [ $length -le $CHAIN_LENGTH_END ]
do
   plog "[ RUN ] Zenoh Flow for chain length $length"
   s=$INITIAL_MSGS
   while [ $s -le $FINAL_MSGS ]
   do

      descriptor_file="descriptor-$s.yaml"


      $BIN_DIR/$DYN_LAT -m $s -p $length -d $descriptor_file > /dev/null 2>&1

      o=1
      while [ $o -le $length ]
      do

         nohup taskset -c $(($o % $N_CPU)) $BIN_DIR/$DYN_LAT -r -n "comp$(($o - 1 ))" -d $descriptor_file > /dev/null 2>&1 &
         ALL_PIDS+=($!)
         o=$(( $o + 1 ))
      done

      nohup $BIN_DIR/$DYN_LAT -r -n "src" -d $descriptor_file > /dev/null 2>&1 &

      timeout $DURATION $BIN_DIR/$DYN_LAT -r -n "snk" -d $descriptor_file >> $THR_FILE

      ps -A | grep $DYN_LAT | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

      plog "[ DONE ] Zenoh Flow for msgs $s, chain length $length"
      sleep 1
      echo "Still running $DYN_LAT: $(ps -A | grep $DYN_LAT | wc -l) - This should be 0"
      rm $descriptor_file

      s=$(($s * 10))

   done
   plog "[ DONE ] Zenoh Flow for chain length $length"
   length=$(($length * 2))
done
TOT_SAMPLES=$(cat $THR_FILE | wc -l)

plog "Done Test results stored in $OUT_DIR - Total Samples: $TOT_SAMPLES"


plog "Running Zenoh Flow Latency test - serde"
THR_FILE="$OUT_DIR/zf-serde-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency" > $THR_FILE

length=$CHAIN_LENGTH
while  [ $length -le $CHAIN_LENGTH_END ]
do
   plog "[ RUN ] Zenoh Flow for chain length $length"
   s=$INITIAL_MSGS
   while [ $s -le $FINAL_MSGS ]
   do

      timeout $DURATION $BIN_DIR/$SERDE -m $s -p $length >> $THR_FILE

      ps -A | grep $SERDE | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

      plog "[ DONE ] Zenoh Flow for msgs $s, chain length $length"
      sleep 1
      echo "Still running $SERDE: $(ps -A | grep $SERDE | wc -l) - This should be 0"

      s=$(($s * 10))

   done
   plog "[ DONE ] Zenoh Flow for chain length $length"
   length=$(($length * 2))
done
TOT_SAMPLES=$(cat $THR_FILE | wc -l)

plog "Done Test same process results stored in $OUT_DIR - Total Samples: $TOT_SAMPLES"

plog "Bye!"


