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
#include "receiver/receiver.hpp"
#include <atomic>


using ato::ros_receiver::Receiver;



Receiver::Receiver(const uint64_t size, const std::string topic_name, const uint64_t pipeline_length, ros::NodeHandle &nh) {
    this->nh = nh;
    this->size = size;
    this->pipeline_length = pipeline_length;
    this->subscriber = this->nh.subscribe(topic_name,1024, &Receiver::receiver_callback, this);
    this->recv.store(0);


    // RCLCPP_INFO(this->get_logger(), "Init Receiver with msg/s %d, pipeline lenght is %d, topic name is %s", this->msgs,  this->pipeline_length, topic_name.c_str());

}

void Receiver::print_stat(const uint64_t elapsed){

    uint64_t received = this->recv.exchange(0, std::memory_order::memory_order_relaxed);
    float interval = 1000000 / float(elapsed);
    uint64_t msgs = (uint64_t) ((float) received/interval);

    if (msgs > 0) {
        // framework,scenario,test,pipeline,payload,rate,value,unit
        std::cout << "ros,multi,latency," << this->pipeline_length << "," << this->size << "," << msgs << "," << msgs << ",msgs" << std::endl << std::flush;
    }

}

void Receiver::receiver_callback(const eval_interfaces::Thr::ConstPtr& msg) {

    (void) msg;
    this->recv++;
}