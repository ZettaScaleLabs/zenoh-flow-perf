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
#include <iostream>

#include "eval_interfaces/msg/thr.hpp"
#include "receiver/receiver.hpp"


using ato::receiver::Receiver;

Receiver::Receiver(const uint64_t size, const std::string topic_name, const uint64_t pipeline_length) : rclcpp::Node("receiver_node", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->size = size;
    this->pipeline_length = pipeline_length;
    this->subscriber = this->create_subscription<eval_interfaces::msg::Thr>(topic_name, qos, std::bind(&Receiver::receiver_callback, this, std::placeholders::_1));
    this->recv.store(0);

    // RCLCPP_INFO(this->get_logger(), "Init Receiver with msg/s %d, pipeline lenght is %d, topic name is %s", this->msgs,  this->pipeline_length, topic_name.c_str());

}

void Receiver::print_stat(const uint64_t elapsed){

    uint64_t received = this->recv.exchange(0, std::memory_order::memory_order_relaxed);
    float interval = 1000000 / float(elapsed);
    uint64_t msgs = (uint64_t) ((float) received/interval);

    if (msgs > 0) {
        // framework,scenario,test,pipeline,payload,rate,value,unit
        std::cout << "ros2,multi,latency," << this->pipeline_length << "," << this->size << "," << msgs << "," << msgs << ",msgs" << std::endl << std::flush;
    }

}

void Receiver::receiver_callback(const eval_interfaces::msg::Thr::SharedPtr msg) {

    (void) msg;
    this->recv++;

}