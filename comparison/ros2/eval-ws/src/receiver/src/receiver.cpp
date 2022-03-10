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

#include <chrono>
#include <vector>
#include <iostream>

#include "eval_interfaces/msg/evaluation.hpp"
#include "receiver/receiver.hpp"


using ato::receiver::Receiver;

Receiver::Receiver(const uint64_t msgs, const std::string topic_name, const uint64_t pipeline_length) : rclcpp::Node("receiver_node", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->msgs = msgs;
    this->pipeline_length = pipeline_length;
    this->subscriber = this->create_subscription<eval_interfaces::msg::Evaluation>(topic_name, qos, std::bind(&Receiver::receiver_callback, this, std::placeholders::_1));
    this->publisher = this->create_publisher<eval_interfaces::msg::Evaluation>("pong", qos);

    // RCLCPP_INFO(this->get_logger(), "Init Receiver with msg/s %d, pipeline lenght is %d, topic name is %s", this->msgs,  this->pipeline_length, topic_name.c_str());

}


void Receiver::receiver_callback(const eval_interfaces::msg::Evaluation::SharedPtr msg) {

    auto ts = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::system_clock::now().time_since_epoch()).count();
    auto latency = ts - msg->emitter_ts;
    // RCLCPP_INFO(this->get_logger(), "Receiver!, %ul - %ul = %dus", msg->emitter_ts, ts, latency);

    // framework,scenario,test,pipeline,payload,rate,value,unit
    std::cout << "ros2,multi,latency," << this->pipeline_length << ",8," << this->msgs << "," << latency << ",us" << std::endl << std::flush;

    auto message = eval_interfaces::msg::Evaluation();
    message.emitter_ts = 0;

    //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
    this->publisher->publish(message);

}