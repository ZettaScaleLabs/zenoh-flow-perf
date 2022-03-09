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

#include "ros/ros.h"
#include "eval_interfaces/Evaluation.h"
#include "receiver/receiver.hpp"
#include <chrono>
#include <thread>


using ato::ros_receiver::Receiver;



Receiver::Receiver(const uint64_t msgs, const std::string topic_name, const uint64_t pipeline_length, ros::NodeHandle &nh) {
    this->nh = nh;
    this->msgs = msgs;
    this->pipeline_length = pipeline_length;
    this->subscriber = this->nh.subscribe(topic_name,1024, &Receiver::receiver_callback, this);
    this->publisher = this->nh.advertise<eval_interfaces::Evaluation>("pong", 1024);

    // RCLCPP_INFO(this->get_logger(), "Init Receiver with msg/s %d, pipeline lenght is %d, topic name is %s", this->msgs,  this->pipeline_length, topic_name.c_str());

}


void Receiver::receiver_callback(const eval_interfaces::Evaluation::ConstPtr& msg) {

    auto ts = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::system_clock::now().time_since_epoch()).count();
    auto latency = ts - msg->emitter_ts;

    // framework,scenario,test,pipeline,payload,rate,value,unit
    std::cout << "ros,multi,latency," << this->pipeline_length << ",8," << this->msgs << "," << this->msgs << "," << latency << ",us" << std::endl << std::flush;

    eval_interfaces::Evaluation message;
    message.emitter_ts = 0;

    //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
    this->publisher.publish(message);

}