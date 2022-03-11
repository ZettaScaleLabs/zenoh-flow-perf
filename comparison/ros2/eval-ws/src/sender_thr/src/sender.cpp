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
#include <thread>
#include "sender/sender.hpp"
#include "eval_interfaces/msg/thr.hpp"


using ato::sender::Sender;

Sender::Sender(const uint64_t size) : rclcpp::Node("sender_node", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->publisher = this->create_publisher<eval_interfaces::msg::Thr>("out_0", qos);
    this->data = std::vector<uint8_t>();
    this->data.reserve(size);

    for (uint64_t i=0; i<size; i++) {
        this->data.push_back(0);
    }

    RCLCPP_INFO(this->get_logger(), "Init Sender with size %ld", size);

}


void Sender::publish_message() {

        auto message = eval_interfaces::msg::Thr();
        message.payload = this->data;
        this->publisher->publish(message);

}