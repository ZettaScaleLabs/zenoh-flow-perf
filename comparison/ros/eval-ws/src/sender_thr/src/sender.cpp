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
#include "sender/sender.hpp"

#define QUEUE_LENGTH 1

using ato::ros_sender::Sender;

Sender::Sender(const uint64_t size, ros::NodeHandle &nh) {
    this->nh = nh;
    this->publisher = this->nh.advertise<eval_interfaces::Thr>("out_0", QUEUE_LENGTH);
    this->data = std::vector<uint8_t>();
    this->data.reserve(size);

    for (uint64_t i=0; i<size; i++) {
        this->data.push_back(0);
    }

    ROS_INFO("Init sender size %ld", size);
}

void Sender::publish_message() {

        eval_interfaces::Thr msg;

        msg.payload = this->data;

        // ROS_INFO("Publish! %zu ", msg.emitter_ts);
        this->publisher.publish(msg);


}
