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
#include <vector>
#include "compute/compute.hpp"
#include "eval_interfaces/msg/thr.hpp"

using ato::compute::Compute;

Compute::Compute(const std::string listen_topic, const std::string publish_topic) : rclcpp::Node("compute_node", rclcpp::NodeOptions().use_intra_process_comms(false)) {

    auto qos_pub = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    auto qos_sub = rclcpp::QoS(rclcpp::KeepAll()).reliable();


    this->publisher = this->create_publisher<eval_interfaces::msg::Thr>(publish_topic, qos_pub);
    this->subscriber = this->create_subscription<eval_interfaces::msg::Thr>(listen_topic, qos_sub, std::bind(&Compute::receiver_callback, this, std::placeholders::_1));

    RCLCPP_INFO(this->get_logger(), "Init Compute with listener %s and publisher %s", listen_topic.c_str(), publish_topic.c_str());

}


void Compute::receiver_callback(const eval_interfaces::msg::Thr::SharedPtr msg) {
     this->publish_message(msg);
}


void Compute::publish_message(const eval_interfaces::msg::Thr::SharedPtr msg) {
    this->publisher->publish(*msg);
}
