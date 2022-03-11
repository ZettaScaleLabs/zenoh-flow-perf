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

#include <rclcpp/rclcpp.hpp>
#include <argparse/argparse.hpp>
#include "sender/sender.hpp"
#include <chrono>
#include <thread>

void send(std::shared_ptr<ato::sender::Sender> sender) {
    std::this_thread::sleep_for(std::chrono::duration<int>(5));

    while (true) {
        sender->publish_message();
    }
}

int main (int argc, char* argv[]) {


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

    rclcpp::init(argc, argv);
    rclcpp::executors::SingleThreadedExecutor executor;


    auto sender = std::make_shared<ato::sender::Sender>(size);
    executor.add_node(sender);

    std::thread t1(send, sender);

    executor.spin();

    rclcpp::shutdown();

    return 0;
}