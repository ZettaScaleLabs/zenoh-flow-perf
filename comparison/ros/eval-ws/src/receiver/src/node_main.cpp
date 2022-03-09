//
// Copyright (c) 2017, 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//


#include "ros/ros.h"
#include "receiver/receiver.hpp"
#include <argparse/argparse.hpp>


int main(int argc, char **argv) {

    argparse::ArgumentParser program("receiver");

    program.add_argument("msgs")
        .help("messages per second")
        .scan<'i', uint64_t>();

     program.add_argument("length")
        .help("length of pipeline")
        .scan<'i', uint64_t>();

    program.add_argument("input")
        .help("input topic");



    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    uint64_t msgs = program.get<uint64_t>("msgs");
    std::string input = program.get<std::string>("input");
    uint64_t length = program.get<uint64_t>("length");

    ros::init(argc, argv, "receiver");

    ros::NodeHandle nh;

    auto sender = ato::ros_receiver::Receiver(msgs, input, length, nh);


    while(nh.ok()) {
        ros::spin();
    }

   return 0;
}