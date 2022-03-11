//
// Copyright (c) 2017, 2022 ZettaScale Technology.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale zenoh team, <zenoh@zettascale.tech>
//


#include "ros/ros.h"
#include "eval_interfaces/Thr.h"
#include "sender/sender.hpp"
#include <argparse/argparse.hpp>
#include <chrono>
#include <thread>

void send(ato::ros_sender::Sender sender) {
    /// Wait for discovery...
    std::this_thread::sleep_for(std::chrono::duration<int>(5));

    while (true) {
        sender.publish_message();
    }

}


int main(int argc, char **argv) {

    argparse::ArgumentParser program("sender");

    program.add_argument("size")
        .help("message size")
        .scan<'i', uint64_t>();

    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    uint64_t size = program.get<uint64_t>("size");

    ros::init(argc, argv, "sender_thr");

    ros::NodeHandle nh;

    auto sender = ato::ros_sender::Sender(size, nh);

    std::thread t1(send, sender);

    while(nh.ok()) {
        ros::spin();
    }

   return 0;
}