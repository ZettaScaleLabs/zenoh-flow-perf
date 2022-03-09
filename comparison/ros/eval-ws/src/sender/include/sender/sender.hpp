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

#include <chrono>
#include <thread>

namespace ato {
    namespace ros_sender {
        class Sender {

            private:
                ros::NodeHandle nh;
                ros::Publisher publisher;
                ros::Subscriber subscriber;
                std::chrono::duration<double> pace;
                void receiver_callback(const eval_interfaces::Evaluation::ConstPtr& msg);


            public:
                explicit Sender(const uint64_t msgs, ros::NodeHandle &nh);
                virtual ~Sender() {}
                void publish_message();



        };
    }
}