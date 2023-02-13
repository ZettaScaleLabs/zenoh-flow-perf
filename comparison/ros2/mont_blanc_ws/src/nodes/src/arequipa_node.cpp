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
#include "nodes.hpp"

int main (int argc, char* argv[]) {

    argparse::ArgumentParser program("arequipa");

    program.add_argument("file")
        .help("file_name");
        // .scan<'s', std::string>();


    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        // std::exit(1);
    }

    std::string file_name = program.get<std::string>("file");

    rclcpp::init(argc, argv);
    rclcpp::executors::SingleThreadedExecutor executor;


    auto arequipa = std::make_shared<ato::nodes::Arequipa>(file_name);
    executor.add_node(arequipa);

    executor.spin();

    rclcpp::shutdown();

    return 0;


}