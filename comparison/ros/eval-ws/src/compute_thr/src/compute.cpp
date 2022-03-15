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
#include "eval_interfaces/Thr.h"
#include "compute/compute.hpp"
#include <chrono>
#include <thread>

#define QUEUE_LENGTH 10

using ato::ros_compute::Compute;

Compute::Compute(const std::string listen_topic, const std::string publish_topic, ros::NodeHandle &nh) {
    this->nh = nh;
    this->publish_topic = publish_topic;
    this->listen_topic = listen_topic;
    this->publisher = this->nh.advertise<eval_interfaces::Thr>(publish_topic, QUEUE_LENGTH);
    this->subscriber = this->nh.subscribe(listen_topic, QUEUE_LENGTH, &Compute::receiver_callback, this);

    ROS_INFO("Init Compute with listener: %s and subscriber %s", listen_topic.c_str(), publish_topic.c_str());
}

void Compute::publish_message(const eval_interfaces::Thr::ConstPtr& msg) {
    this->publisher.publish(msg);
}


void Compute::receiver_callback(const eval_interfaces::Thr::ConstPtr& msg){
    this->publish_message(msg);
}