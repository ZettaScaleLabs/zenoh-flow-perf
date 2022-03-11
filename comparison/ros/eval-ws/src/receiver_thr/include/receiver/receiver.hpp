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
#include "ros/ros.h"
#include "eval_interfaces/Thr.h"
#include <atomic>

namespace ato {
    namespace ros_receiver {
        class Receiver {

            public:
                explicit Receiver(const uint64_t size, const std::string topic_name, const uint64_t pipeline_length, ros::NodeHandle &nh);
                virtual ~Receiver() {};
                void print_stat(const uint64_t elapsed);

            private:
                ros::NodeHandle nh;
                ros::Subscriber subscriber;
                uint64_t size;
                uint64_t pipeline_length;
                std::atomic<uint64_t> recv;
                void receiver_callback(const eval_interfaces::Thr::ConstPtr& msg);

        };
    }
}