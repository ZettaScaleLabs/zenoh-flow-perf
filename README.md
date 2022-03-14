# Zenoh Flow Performance tests

This repository contains the script and binaries for zenoh flow performance figures gathering and comparison.

## Prerequisites

1. Install [rust](https://www.rust-lang.org/it)
2. Install all the dependencies `cmake bison`, [argparse](https://github.com/p-ranav/argparse)
3. Install [ROS](http://wiki.ros.org/noetic/Installation)
4. Install [ROS2](https://docs.ros.org/en/galactic/Installation.html), the [CycloneDDS RMW](https://docs.ros.org/en/galactic/Installation/DDS-Implementations/Working-with-Eclipse-CycloneDDS.html), and [colcon](https://docs.ros.org/en/galactic/Tutorials/Colcon-Tutorial.html)
5. Install [CycloneDDS](https://github.com/eclipse-cyclonedds/cyclonedds)
6. Install the parsing requirements `pip3 install -r requirements.txt`

## Build the tests

If all the dependencies are installed you can build all the tests with `make`.

If you want to clean the build you can use `make clean`.

### rust tests

```bash
$ git clone git@github.com:atolab/zenoh-flow-perf.git
$ cd zenoh-flow-perf
$ RUSTFLAGS='-C target-cpu=native'  cargo build --release --all-targets
```

### ROS2 tests

```bash
$ cd zenoh-flow-perf/comparison/ros2/eval-ws
$ source /opt/ros/galactic/setup.bash
$ colcon build
```

### ROS tests

```bash
$ cd zenoh-flow-perf/comparison/ros/eval-ws
$ source /opt/ros/noetic/setup.bash
$ catkin_make
```



### CycloneDDS tests

```bash
$ cd zenoh-flow-perf/comparison/cyclonedds/ping-pong
$ mkdir build
$ cmake ..
$ make
```


## Run the tests

The script `run-breakdown-tests.sh` is provided for convenience, this script is able to run the different tests, just look at the usage.

```bash
$ ./run-breakdown-tests.sh
Usage: ./run-breakdown-tests.sh
        -f flume
        -l link
        -s static
        -d dynamic
        -z zenoh
        -c CycloneDDS
        -r ROS2
        -R ROS
```


## Plot the result

Similarly the `parse.py` script is able to provide different graph and filtering on the tests.


```bash
$ ./parse.py
usage: parse.py [-h] [-k {latency,throughput}] -d DATA [-p {single,multi,all}] [-t {stat,time,ecdf,pdf}] [-s {log,lin}] [-m MSGS]
                [-l LENGTH] [-o OUTPUT] [-r RESAMPLE]
```