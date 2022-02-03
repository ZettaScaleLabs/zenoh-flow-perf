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
#include <chrono>
#include <vector>
#include "eval_interfaces/msg/evaluation.hpp"

namespace ato {
    namespace compute {
        class Compute : public rclcpp::Node {

            public:
                explicit Compute(const std::string listen_topic, const std::string publish_topic);
                virtual ~Compute() {}

            private:
                rclcpp::Publisher<eval_interfaces::msg::Evaluation>::SharedPtr publisher;
                rclcpp::Subscription<eval_interfaces::msg::Evaluation>::SharedPtr subscriber;
                void receiver_callback(const eval_interfaces::msg::Evaluation::SharedPtr msg);
                void publish_message(const eval_interfaces::msg::Evaluation::SharedPtr msg);

        };
    }
}