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
#include "eval_interfaces/Evaluation.h"

namespace ato {
    namespace ros_receiver {
        class Receiver {

            public:
                explicit Receiver(const uint64_t msgs, const std::string topic_name, const uint64_t pipeline_length, ros::NodeHandle &nh);
                virtual ~Receiver() {};

            private:
                ros::NodeHandle nh;
                ros::Publisher publisher;
                ros::Subscriber subscriber;
                uint64_t msgs;
                uint64_t pipeline_length;
                void receiver_callback(const eval_interfaces::Evaluation::ConstPtr& msg);

        };
    }
}