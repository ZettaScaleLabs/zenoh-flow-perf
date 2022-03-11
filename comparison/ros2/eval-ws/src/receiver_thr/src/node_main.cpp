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

#include <chrono>
#include <thread>

#include <rclcpp/rclcpp.hpp>
#include <argparse/argparse.hpp>
#include "receiver/receiver.hpp"

void print_statistics(std::shared_ptr<ato::receiver::Receiver> receiver) {

    while (true) {
        std::chrono::steady_clock::time_point begin = std::chrono::steady_clock::now();
        std::this_thread::sleep_for(std::chrono::duration<int>(1));
        std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();

        uint64_t elapsed = std::chrono::duration_cast<std::chrono::microseconds>(end - begin).count();
        receiver->print_stat(elapsed);
    }

}

int main (int argc, char* argv[]) {


    argparse::ArgumentParser program("receiver");

    program.add_argument("size")
        .help("message size")
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

    uint64_t size = program.get<uint64_t>("size");
    std::string input = program.get<std::string>("input");
    uint64_t length = program.get<uint64_t>("length");


    rclcpp::init(argc, argv);
    rclcpp::executors::SingleThreadedExecutor executor;


    auto receiver = std::make_shared<ato::receiver::Receiver>(size, input, length);

    std::thread t1(print_statistics, receiver);

    executor.add_node(receiver);

    executor.spin();

    rclcpp::shutdown();

    return 0;

}