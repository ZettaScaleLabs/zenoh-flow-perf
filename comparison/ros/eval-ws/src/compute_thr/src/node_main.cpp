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
#include "eval_interfaces/Evaluation.h"
#include "compute/compute.hpp"
#include <argparse/argparse.hpp>
#include <chrono>
#include <thread>

int main(int argc, char **argv) {

    argparse::ArgumentParser program("compute");

    program.add_argument("input")
        .help("input topic");
        // .scan<'s', std::string>();

    program.add_argument("output")
        .help("output topic");
        // .scan<'s', std::string>();

    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    std::string input = program.get<std::string>("input");
    std::string output = program.get<std::string>("output");


    ros::init(argc, argv, "compute_thr");

    ros::NodeHandle nh;

    auto sender = ato::ros_compute::Compute(input, output, nh);

    while(nh.ok()) {
        ros::spin();
    }

   return 0;
}