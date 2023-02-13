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

#include <iostream>
#include <cstdio>
#include <fstream>
#include <rclcpp/rclcpp.hpp>
#include <chrono>
#include <vector>

#include "std_msgs/msg/string.hpp"
#include "std_msgs/msg/float32.hpp"
#include "std_msgs/msg/float64.hpp"
#include "std_msgs/msg/int32.hpp"
#include "std_msgs/msg/int64.hpp"
#include "std_msgs/msg/header.hpp"
#include "geometry_msgs/msg/twist_with_covariance_stamped.hpp"
#include "geometry_msgs/msg/twist_with_covariance.hpp"
#include "geometry_msgs/msg/wrench_stamped.hpp"
#include "geometry_msgs/msg/twist.hpp"
#include "geometry_msgs/msg/wrench.hpp"
#include "geometry_msgs/msg/vector3.hpp"
#include "geometry_msgs/msg/vector3_stamped.hpp"
#include "geometry_msgs/msg/pose.hpp"
#include "sensor_msgs/msg/image.hpp"
#include "sensor_msgs/msg/point_field.hpp"
#include "sensor_msgs/msg/point_cloud2.hpp"
#include "sensor_msgs/msg/laser_scan.hpp"

namespace ato {
    namespace nodes {

        class Arequipa : public rclcpp::Node {

            public:
                explicit Arequipa(const std::string file_name);
                ~Arequipa();

            private:
                std::ofstream outfile;
                rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscriber;
                void receiver_callback(const std_msgs::msg::String::SharedPtr msg);
        };


        class Barcelona : public rclcpp::Node {

            public:
                explicit Barcelona();
                virtual ~Barcelona() {};

            private:
                rclcpp::Publisher<geometry_msgs::msg::WrenchStamped>::SharedPtr publisher;
                rclcpp::Subscription<geometry_msgs::msg::TwistWithCovarianceStamped>::SharedPtr subscriber;
                void receiver_callback(const geometry_msgs::msg::TwistWithCovarianceStamped::SharedPtr msg);
                void publish_message(const geometry_msgs::msg::WrenchStamped msg);
        };


        class Cordoba : public rclcpp::Node {

            public:
                explicit Cordoba();
                virtual ~Cordoba() {}

            private:
                rclcpp::Publisher<std_msgs::msg::Float32>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;
                std_msgs::msg::Float32 data;
                void publish_message();

        };

        class Delhi : public rclcpp::Node {

            public:
                explicit Delhi();
                virtual ~Delhi() {}

            private:
                rclcpp::Publisher<sensor_msgs::msg::Image>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;
                sensor_msgs::msg::Image data;
                void publish_message();

        };


        class Freeport : public rclcpp::Node {

            public:
                explicit Freeport();
                virtual ~Freeport() {}

            private:
                rclcpp::Publisher<std_msgs::msg::Int64>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;
                std_msgs::msg::Int64 data;
                void publish_message();

        };


        class Geneva : public rclcpp::Node {

            public:
                explicit Geneva();
                virtual ~Geneva() {}

            private:
                rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscriber_parana;
                rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscriber_danube;
                rclcpp::Subscription<geometry_msgs::msg::Pose>::SharedPtr subscriber_tagus;
                rclcpp::Subscription<geometry_msgs::msg::Twist>::SharedPtr subscriber_congo;

                rclcpp::Publisher<std_msgs::msg::String>::SharedPtr publisher;

                std_msgs::msg::String parana_val;
                std_msgs::msg::String danube_val;
                geometry_msgs::msg::Pose tagus_val;
                geometry_msgs::msg::Twist congo_val;

                std_msgs::msg::String data;
                void publish_message();

                void receiver_parana(const std_msgs::msg::String::SharedPtr msg);
                void receiver_danube(const std_msgs::msg::String::SharedPtr msg);
                void receiver_tagus(const geometry_msgs::msg::Pose::SharedPtr msg);
                void receiver_congo(const geometry_msgs::msg::Twist::SharedPtr msg);

        };


        class Georgetown : public rclcpp::Node {

            public:
                explicit Georgetown();
                virtual ~Georgetown() {}

            private:
                rclcpp::Subscription<geometry_msgs::msg::Vector3Stamped>::SharedPtr subscriber_murray;
                rclcpp::Subscription<geometry_msgs::msg::WrenchStamped>::SharedPtr subscriber_lena;
                rclcpp::Publisher<std_msgs::msg::Float64>::SharedPtr publisher;

                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;

                geometry_msgs::msg::Vector3Stamped murray_val;
                geometry_msgs::msg::WrenchStamped lena_val;
                std_msgs::msg::Float64 data;
                void publish_message();

                void receiver_murray(const geometry_msgs::msg::Vector3Stamped::SharedPtr msg);
                void receiver_lena(const geometry_msgs::msg::WrenchStamped::SharedPtr msg);

        };


        class Hamburg : public rclcpp::Node {

            public:
                explicit Hamburg();
                virtual ~Hamburg() {}

            private:
                rclcpp::Subscription<std_msgs::msg::Float32>::SharedPtr subscriber_tigris;
                rclcpp::Subscription<std_msgs::msg::Int64>::SharedPtr subscriber_ganges;
                rclcpp::Subscription<std_msgs::msg::Int32>::SharedPtr subscriber_nile;
                rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscriber_danube;

                rclcpp::Publisher<std_msgs::msg::String>::SharedPtr publisher;

                std_msgs::msg::Float32 tigris_val;
                std_msgs::msg::Int64 ganges_val;
                std_msgs::msg::Int32 nile_val;
                std_msgs::msg::String danube_val;

                std_msgs::msg::String data;
                void publish_message();

                void receiver_tigris(const std_msgs::msg::Float32::SharedPtr msg);
                void receiver_ganges(const std_msgs::msg::Int64::SharedPtr msg);
                void receiver_nile(const std_msgs::msg::Int32::SharedPtr msg);
                void receiver_danube(const std_msgs::msg::String::SharedPtr msg);

        };


        class Hebron : public rclcpp::Node {

            public:
                explicit Hebron();
                virtual ~Hebron() {}

            private:
                rclcpp::Publisher<geometry_msgs::msg::Quaternion>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;
                geometry_msgs::msg::Quaternion data;
                void publish_message();

        };


        class Kingston : public rclcpp::Node {

            public:
                explicit Kingston();
                virtual ~Kingston() {}

            private:
                rclcpp::Publisher<geometry_msgs::msg::Vector3>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;
                geometry_msgs::msg::Vector3 data;
                void publish_message();

        };


        class Lyon : public rclcpp::Node {

            public:
                explicit Lyon();
                virtual ~Lyon() {}

            private:
                rclcpp::Subscription<std_msgs::msg::Float32>::SharedPtr subscriber_amazon;
                rclcpp::Publisher<std_msgs::msg::Float32>::SharedPtr publisher;

                void publish_message(const std_msgs::msg::Float32::SharedPtr msg);
                void receiver_amazon(const std_msgs::msg::Float32::SharedPtr msg);
        };


        class Madellin : public rclcpp::Node {

            public:
                explicit Madellin();
                virtual ~Madellin() {}

            private:
                rclcpp::Publisher<std_msgs::msg::Int32>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;
                std_msgs::msg::Int32 data;
                void publish_message();

        };

     class Mandalay : public rclcpp::Node {

            public:
                explicit Mandalay();
                virtual ~Mandalay() {}

            private:
                rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscriber_danube;
                rclcpp::Subscription<geometry_msgs::msg::Quaternion>::SharedPtr subscriber_chenab;
                rclcpp::Subscription<sensor_msgs::msg::PointCloud2>::SharedPtr subscriber_salween;
                rclcpp::Subscription<sensor_msgs::msg::LaserScan>::SharedPtr subscriber_godavari;
                rclcpp::Subscription<sensor_msgs::msg::PointCloud2>::SharedPtr subscriber_loire;
                rclcpp::Subscription<geometry_msgs::msg::Vector3>::SharedPtr subscriber_yamuna;

                rclcpp::Publisher<sensor_msgs::msg::PointCloud2>::SharedPtr publisher_brazos;
                rclcpp::Publisher<geometry_msgs::msg::Pose>::SharedPtr publisher_tagus;
                rclcpp::Publisher<sensor_msgs::msg::Image>::SharedPtr publisher_missouri;

                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;

                std_msgs::msg::String danube_val;
                geometry_msgs::msg::Quaternion chenab_val;
                sensor_msgs::msg::PointCloud2 salween_val;
                sensor_msgs::msg::LaserScan godavari_val;
                sensor_msgs::msg::PointCloud2 loire_val;
                geometry_msgs::msg::Vector3 yamuna_val;

                geometry_msgs::msg::Pose tagus_data;
                sensor_msgs::msg::Image missouri_data;

                void publish_message();

                void receiver_danube(const std_msgs::msg::String::SharedPtr msg);
                void receiver_chenab(const geometry_msgs::msg::Quaternion::SharedPtr msg);
                void receiver_salween(const sensor_msgs::msg::PointCloud2::SharedPtr msg);
                void receiver_godavari(const sensor_msgs::msg::LaserScan::SharedPtr msg);
                void receiver_loire(const sensor_msgs::msg::PointCloud2::SharedPtr msg);
                void receiver_yamuna(const geometry_msgs::msg::Vector3::SharedPtr msg);

        };


    class Monaco : public rclcpp::Node {

            public:
                explicit Monaco();
                virtual ~Monaco() {}

            private:
                rclcpp::Subscription<geometry_msgs::msg::Twist>::SharedPtr subscriber_congo;
                rclcpp::Publisher<std_msgs::msg::Float32>::SharedPtr publisher;

                geometry_msgs::msg::Twist congo_val;
                std_msgs::msg::Float32 data;

                void publish_message();
                void receiver_congo(const geometry_msgs::msg::Twist::SharedPtr msg);
        };

    class Osaka : public rclcpp::Node {

            public:
                explicit Osaka();
                virtual ~Osaka() {}

            private:

                rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscriber_parana;
                rclcpp::Subscription<sensor_msgs::msg::Image>::SharedPtr subscriber_columbia;
                rclcpp::Subscription<sensor_msgs::msg::Image>::SharedPtr subscriber_colorado;

                rclcpp::Publisher<sensor_msgs::msg::PointCloud2>::SharedPtr publisher_salween;
                rclcpp::Publisher<sensor_msgs::msg::LaserScan>::SharedPtr publisher_godavari;

                sensor_msgs::msg::LaserScan godavari_data;
                sensor_msgs::msg::PointCloud2 salween_data;

                sensor_msgs::msg::Image columbia_val;
                sensor_msgs::msg::Image colorado_val;
                std_msgs::msg::String parana_val;

                void publish_message();

                void receiver_parana(const std_msgs::msg::String::SharedPtr msg);
                void receiver_columbia(const sensor_msgs::msg::Image::SharedPtr msg);
                void receiver_colorado(const sensor_msgs::msg::Image::SharedPtr msg);

        };


    class Ponce : public rclcpp::Node {

            public:
                explicit Ponce();
                virtual ~Ponce() {}

            private:
                rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscriber_danube;
                rclcpp::Subscription<geometry_msgs::msg::Pose>::SharedPtr subscriber_tagus;
                rclcpp::Subscription<sensor_msgs::msg::Image>::SharedPtr subscriber_missouri;
                rclcpp::Subscription<sensor_msgs::msg::PointCloud2>::SharedPtr subscriber_loire;
                rclcpp::Subscription<geometry_msgs::msg::Vector3>::SharedPtr subscriber_yamuna;
                rclcpp::Subscription<std_msgs::msg::Float32>::SharedPtr subscriber_ohio;
                rclcpp::Subscription<std_msgs::msg::Float64>::SharedPtr subscriber_volga;
                rclcpp::Subscription<sensor_msgs::msg::PointCloud2>::SharedPtr subscriber_brazos;


                rclcpp::Publisher<geometry_msgs::msg::Twist>::SharedPtr publisher_congo;
                rclcpp::Publisher<geometry_msgs::msg::TwistWithCovarianceStamped>::SharedPtr publisher_mekong;

                std_msgs::msg::String danube_val;
                geometry_msgs::msg::Pose tagus_val;
                sensor_msgs::msg::Image missouri_val;
                sensor_msgs::msg::PointCloud2 loire_val;
                geometry_msgs::msg::Vector3 yamuna_val;
                std_msgs::msg::Float32 ohio_val;
                std_msgs::msg::Float64 volga_val;
                sensor_msgs::msg::PointCloud2 brazos_val;

                geometry_msgs::msg::Twist congo_data;
                geometry_msgs::msg::TwistWithCovarianceStamped mekong_data;

                void publish_message();

                void receiver_danube(const std_msgs::msg::String::SharedPtr msg);
                void receiver_tagus(const geometry_msgs::msg::Pose::SharedPtr msg);
                void receiver_missouri(const sensor_msgs::msg::Image::SharedPtr msg);
                void receiver_loire(const sensor_msgs::msg::PointCloud2::SharedPtr msg);
                void receiver_yamuna(const geometry_msgs::msg::Vector3::SharedPtr msg);
                void receiver_ohio(const std_msgs::msg::Float32::SharedPtr msg);
                void receiver_volga(const std_msgs::msg::Float64::SharedPtr msg);
                void receiver_brazos(const sensor_msgs::msg::PointCloud2::SharedPtr msg);

        };


        class Portsmouth : public rclcpp::Node {

            public:
                explicit Portsmouth();
                virtual ~Portsmouth() {}

            private:
                rclcpp::Publisher<std_msgs::msg::String>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                std::chrono::duration<double> pace;
                std_msgs::msg::String data;
                void publish_message();

        };

    class Rotterdam : public rclcpp::Node {

            public:
                explicit Rotterdam();
                virtual ~Rotterdam() {}

            private:
                rclcpp::Subscription<geometry_msgs::msg::TwistWithCovarianceStamped>::SharedPtr subscriber_mekong;
                rclcpp::Publisher<geometry_msgs::msg::Vector3Stamped>::SharedPtr publisher;

                geometry_msgs::msg::TwistWithCovarianceStamped mekong_val;
                geometry_msgs::msg::Vector3Stamped data;

                void publish_message();
                void receiver_mekong(const geometry_msgs::msg::TwistWithCovarianceStamped::SharedPtr msg);
        };



    class Taipei : public rclcpp::Node {

            public:
                explicit Taipei();
                virtual ~Taipei() {}

            private:
                rclcpp::Subscription<sensor_msgs::msg::Image>::SharedPtr subscriber_columbia;
                rclcpp::Publisher<sensor_msgs::msg::Image>::SharedPtr publisher;


                sensor_msgs::msg::Image data;

                void publish_message();
                void receiver_columbia(const sensor_msgs::msg::Image::SharedPtr msg);
        };

    class Tripoli : public rclcpp::Node {

            public:
                explicit Tripoli();
                virtual ~Tripoli() {}

            private:
                rclcpp::Subscription<sensor_msgs::msg::Image>::SharedPtr subscriber_columbia;
                rclcpp::Subscription<sensor_msgs::msg::LaserScan>::SharedPtr subscriber_godavari;
                rclcpp::Publisher<sensor_msgs::msg::PointCloud2>::SharedPtr publisher;

                sensor_msgs::msg::LaserScan godavari_val;
                sensor_msgs::msg::Image columbia_val;
                sensor_msgs::msg::PointCloud2 data;

                void publish_message();
                void receiver_columbia(const sensor_msgs::msg::Image::SharedPtr msg);
                void receiver_godavari(const sensor_msgs::msg::LaserScan::SharedPtr msg);
        };

    }

}