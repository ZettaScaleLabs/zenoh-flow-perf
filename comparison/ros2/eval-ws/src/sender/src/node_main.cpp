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

#include <rclcpp/rclcpp.hpp>
#include <argparse/argparse.hpp>
#include "sender/sender.hpp"
#include <chrono>
#include <thread>

void send_single(std::shared_ptr<ato::sender::Sender> sender) {
    std::this_thread::sleep_for(std::chrono::duration<int>(5));
    sender->publish_message();
}

int main (int argc, char* argv[]) {


    argparse::ArgumentParser program("sender");

    program.add_argument("msgs")
        .help("messages per second")
        .scan<'i', uint64_t>();

    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    uint64_t msgs = program.get<uint64_t>("msgs");

    rclcpp::init(argc, argv);
    rclcpp::executors::SingleThreadedExecutor executor;


    auto sender = std::make_shared<ato::sender::Sender>(msgs);
    executor.add_node(sender);

    std::thread t1(send_single, sender);

    executor.spin();

    rclcpp::shutdown();

    return 0;
}