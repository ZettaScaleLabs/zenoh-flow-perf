#!/usr/bin/env bash


plog () {
TS=`eval date "+%F-%T"`
   echo "[$TS]: $1"
}

TS=$(date "+%F-%T")

N_CPU=$(nproc)

INITIAL_MSGS=1
#FINAL_MSGS=1
#FINAL_MSGS=1000
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


OUT_DIR="breakdown-logs"

DURATION=60

mkdir -p $OUT_DIR

# Flume

plog "[ START ] baseline Flume Latency test"
LOG_FILE="$OUT_DIR/flume-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] Flume for msg/s $s"

   timeout $DURATION $BIN_DIR/$LAT_FLUME -m $s >> $LOG_FILE

   ps -ax | grep $LAT_FLUME | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] Flume for msg/s $s"
   sleep 1
   echo "Still running $LLAT_FLUMEAT: $(ps -ax | grep $LAT_FLUME | wc -l) - This should be 0"
   s=$(($s * 10))

done
plog "[ END ] baseline Flume Latency test"

# ZF Link

plog "[ START ] baseline ZF Link Latency test"
LOG_FILE="$OUT_DIR/zf-link-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] ZF Link for msg/s $s"

   timeout $DURATION $BIN_DIR/$LAT_LINK -m $s >> $LOG_FILE

   ps -ax | grep $LAT_LINK | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] ZF Link for msg/s $s"
   sleep 1
   echo "Still running $LAT_LINK: $(ps -ax | grep $LAT_LINK | wc -l) - This should be 0"
   s=$(($s * 10))

done
plog "[ END ] baseline ZF Link Latency test"


# Zenoh

plog "[ START ] baseline Zenoh Latency test"
LOG_FILE="$OUT_DIR/zenoh-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] baseline Zenoh for msg/s $s"

   nohup $BIN_DIR/$LAT_ZENOH -m $s >> $LOG_FILE 2> /dev/null &

   timeout $DURATION $BIN_DIR/$LAT_ZENOH -p -m $s > /dev/null 2>&1

   ps -ax | grep $LAT_ZENOH | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] baseline Zenoh for msg/s $s"
   sleep 1
   echo "Still running $LAT_ZENOH: $(ps -ax | grep $LAT_ZENOH | wc -l) - This should be 0"

   s=$(($s * 10))
done
plog "[ END ] baseline Zenoh Latency test"


# Static Source->Sink

plog "[ START ] Zenoh Flow Source->Sink Static Latency test"
LOG_FILE="$OUT_DIR/zf-src-snk-static-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] Zenoh Flow Source->Sink Static for msg/s $s"

   timeout $DURATION $BIN_DIR/$LAT_SRC_SNK_STATIC -m $s >> $LOG_FILE

   ps -ax | grep $LAT_SRC_SNK_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] Zenoh Flow Source->Sink Static for msg/s $s"
   sleep 1
   echo "Still running $LAT_SRC_SNK_STATIC: $(ps -ax | grep $LAT_SRC_SNK_STATIC | wc -l) - This should be 0"
   s=$(($s * 10))

done
plog "[ END ] Zenoh Flow Source->Sink Static Latency test"

# Static Source->Op

plog "[ START ] Zenoh Flow Source->Operator Static Latency test"
LOG_FILE="$OUT_DIR/zf-src-op-static-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] Zenoh Flow Source->Operator Static for msg/s $s"

   timeout $DURATION $BIN_DIR/$LAT_SRC_OP_STATIC -m $s >> $LOG_FILE

   ps -ax | grep $LAT_SRC_OP_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] Zenoh Flow Source->Operator Static for msg/s $s"
   sleep 1
   echo "Still running $LAT_SRC_OP_STATIC: $(ps -ax | grep $LAT_SRC_OP_STATIC | wc -l) - This should be 0"
   s=$(($s * 10))

done
plog "[ END ] Zenoh Flow Source->Operator Static Latency test"


# Dynamic Source->Sink

plog "[ START ] Zenoh Flow Source->Sink Dynamic Latency test"
LOG_FILE="$OUT_DIR/zf-src-snk-dynamic-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] Zenoh Flow Source->Sink Dynamic for msg/s $s"

   descriptor_file="descriptor-src-snk-$s.yaml"

   $BIN_DIR/$LAT_SRC_SNK_DYNAMIC -m $s -d $descriptor_file > /dev/null 2>&1

   nohup $BIN_DIR/$LAT_SRC_SNK_DYNAMIC -r -n "snk" -d $descriptor_file  >> $LOG_FILE 2> /dev/null &

   timeout $DURATION $BIN_DIR/$LAT_SRC_SNK_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1

   ps -ax | grep $LAT_SRC_SNK_DYNAMIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] Zenoh Flow Source->Sink Dynamic for msg/s $s"
   sleep 1
   echo "Still running $LAT_SRC_SNK_DYNAMIC: $(ps -ax | grep $LAT_SRC_SNK_DYNAMIC | wc -l) - This should be 0"
   rm $descriptor_file
   s=$(($s * 10))
done

sed -i -e 's/zenoh-flow-multi/zf-source-sink-multi/g' $LOG_FILE
plog "[ END ] Zenoh Flow Source->Sink Dynamic Latency test"

# Dynamic Source->Op

plog "[ START ] Zenoh Flow Source->Operator Dynamic Latency test"
LOG_FILE="$OUT_DIR/zf-src-op-dynamic-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] Zenoh Flow Source->Operator Dynamic for msg/s $s"

   descriptor_file="descriptor-src-op-$s.yaml"

   $BIN_DIR/$LAT_SRC_OP_DYNAMIC -m $s -d $descriptor_file > /dev/null 2>&1

   nohup $BIN_DIR/$LAT_SRC_OP_DYNAMIC -r -n "comp" -d $descriptor_file >> $LOG_FILE 2> /dev/null &
   nohup $BIN_DIR/$LAT_SRC_OP_DYNAMIC -r -n "snk" -d $descriptor_file > /dev/null 2>&1 &

   timeout $DURATION $BIN_DIR/$LAT_SRC_OP_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1

   ps -ax | grep $LAT_SRC_OP_DYNAMIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] Zenoh Flow Source->Operator Dynamic for msg/s $s"
   sleep 1
   echo "Still running $LAT_SRC_OP_DYNAMIC: $(ps -ax | grep $LAT_SRC_OP_DYNAMIC | wc -l) - This should be 0"
   rm $descriptor_file
   s=$(($s * 10))

done
plog "[ END ] Zenoh Flow Source->Operator Dynamic Latency test"




# Static Source->Op->Sink

plog "[ START ] Zenoh Flow Source->Operator->Sink Static Latency test"
LOG_FILE="$OUT_DIR/zf-src-op-sink-static-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] Zenoh Flow Source->Operator->Sink Static for msg/s $s"

   timeout $DURATION $BIN_DIR/$LAT_STATIC -m $s -p >> $LOG_FILE

   ps -ax | grep $LAT_STATIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] Zenoh Flow Source->Operator->Sink Static for msg/s $s"
   sleep 1
   echo "Still running $LAT_STATIC: $(ps -ax | grep $LAT_STATIC | wc -l) - This should be 0"
   rm $descriptor_file
   s=$(($s * 10))

done
plog "[ END ] Zenoh Flow Source->Operator->Sink Static Latency test"


plog "Bye!"


# Dynamic Source->Op->Sink

plog "[ START ] Zenoh Flow Source->Operator->Sink Dynamic Latency test"
LOG_FILE="$OUT_DIR/zf-src-op-sink-dynamic-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] Zenoh Flow Source->Operator->Sink Dynamic for msg/s $s"

   descriptor_file="descriptor-src-op-sink-$s.yaml"

   $BIN_DIR/$LAT_DYNAMIC -m $s -d $descriptor_file > /dev/null 2>&1

   nohup $BIN_DIR/$LAT_DYNAMIC -r -n "comp0" -d $descriptor_file > /dev/null 2>&1 &
   nohup $BIN_DIR/$LAT_DYNAMIC -r -n "snk" -d $descriptor_file >> $LOG_FILE 2> /dev/null &

   timeout $DURATION $BIN_DIR/$LAT_DYNAMIC -r -n "src" -d $descriptor_file > /dev/null 2>&1

   ps -ax | grep $LAT_DYNAMIC | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] Zenoh Flow Source->Operator->Sink Dynamic for msg/s $s"
   sleep 1
   echo "Still running $LAT_DYNAMIC: $(ps -ax | grep $LAT_DYNAMIC | wc -l) - This should be 0"
   rm $descriptor_file
   s=$(($s * 10))

done
plog "[ END ] Zenoh Flow Source->Operator->Sink Dynamic Latency test"



# Zenoh UDP

plog "[ START ] baseline Zenoh UDP Latency test"
LOG_FILE="$OUT_DIR/zenoh-udp-$TS.csv"
echo "layer,scenario,test,name,messages,pipeline,latency,unit" > $LOG_FILE
s=$INITIAL_MSGS
while [ $s -le $FINAL_MSGS ]
do
   plog "[ RUN ] baseline Zenoh for msg/s $s"

   nohup $BIN_DIR/$LAT_ZENOH -u -m $s >> $LOG_FILE 2> /dev/null &

   timeout $DURATION $BIN_DIR/$LAT_ZENOH -u -p -m $s > /dev/null 2>&1

   ps -ax | grep $LAT_ZENOH | awk {'print $1'} | xargs kill -9 > /dev/null  2>&1

   plog "[ DONE ] baseline Zenoh UDP for msg/s $s"
   sleep 1
   echo "Still running $LAT_ZENOH: $(ps -ax | grep $LAT_ZENOH | wc -l) - This should be 0"

   s=$(($s * 10))
done
plog "[ END ] baseline Zenoh UDP Latency test"


plog "Bye!"


# get TCP sizes:
# ss -m --info | grep -A1 -m1 $(ss -p | grep 2541023 | awk {'print $5'} ) | tail -1
#            receiving packet allocated memory  ,
# eg  skmem:(r0                                 ,
#            total memory of receiving buffer
#            rb2358342,
#            memory used for sending packet (sent to l3)
#            t0,
#            total memory allocated for sending packet
#            tb2626560,
#            socket cache
#            f4096,
#            memory allocated to sending packet (not sent to l3)
#            w0,
#            memory used for storing socket options
#            o0,
#            memory used for socket backlog queue
#            bl0,
#            packet dropped before demultiplexed into socket
#            d0) c

# windowd scale factor
#cubic wscale:7,7
# re-transmission timeout in ms
#rto:204
# avg rtt/men dev rtt in ms
#rtt:0.066/0.007
# ack timeout in ms
#ato:40
# max segment size
#mss:32768
# path mtu
#pmtu:65535
# ??
#rcvmss:536
# ??
#advmss:65483
# size of congestion window
# cwnd:10
# tcp congestion slow start threshold
#ssthresh:66
# sent bytes
# bytes_sent:1719
# acked bytes
# bytes_acked:1719
# received bytes
# bytes_received:87262
# sent segments
# segs_out:1210
# received segments
# segs_in:1212
# ??
# data_segs_out:348
# ??
# data_segs_in:862
# egress bps
# send 39718.8Mbps
# how long since last packet send in ms
# lastsnd:1860
# how long since last packet recv in ms
# lastrcv:636
# ms since last ack recv
# lastack:1636
# the pacing rate/max pacing rage
#pacing_rate 78692.4Mbps
# ??
#delivery_rate 18724.6Mbps
# ?
#delivered:349
# ??
#app_limited busy:36ms
# ??
#rcv_rtt:109032
# helper for tcp internal auto tuning socker recv buffer
# rcv_space:65650
# ??/
# rcv_ssthresh:65483
# ??
# minrtt:0.014