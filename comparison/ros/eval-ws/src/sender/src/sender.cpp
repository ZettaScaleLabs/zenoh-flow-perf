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
#include "sender/sender.hpp"
#include <chrono>
#include <thread>


using ato::ros_sender::Sender;

Sender::Sender(const uint64_t msgs, ros::NodeHandle &nh) {
    this->nh = nh;

    this->pace = std::chrono::duration<double>(1.0/double(msgs));
    this->publisher = this->nh.advertise<eval_interfaces::Evaluation>("out_0", 1024);
    this->subscriber = this->nh.subscribe("pong", 1000, &Sender::receiver_callback, this);

    ROS_INFO("Init sender msg/s %ld", msgs);
}

void Sender::publish_message() {

        eval_interfaces::Evaluation msg;

        auto ts = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::system_clock::now().time_since_epoch());
        msg.emitter_ts = ts.count();

        // ROS_INFO("Publish! %zu ", msg.emitter_ts);
        this->publisher.publish(msg);


}

void Sender::receiver_callback(const eval_interfaces::Evaluation::ConstPtr& msg){
    (void) msg;
    std::this_thread::sleep_for(this->pace);
    this->publish_message();
}