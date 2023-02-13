
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

#include <random>
#include <vector>
#include <algorithm>
#include <limits>
#include <string>

#include "std_msgs/msg/string.hpp"
#include "std_msgs/msg/float32.hpp"
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
    namespace utils {

        std::string random_string(size_t length)
        {
        auto randchar = []() -> char
            {
            const char charset[] =
                "0123456789"
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                "abcdefghijklmnopqrstuvwxyz"
                "`~!@#$%^&*()_+"
                "[]\\;',./{}|:\"<>?";
            const size_t max_index = (sizeof(charset) - 1);
            return charset[rand() % max_index];
            };
        std::string str(length, 0);
        std::generate_n(str.begin(), length, randchar);
        return str;
        }

        template<typename T,typename = typename std::enable_if<std::is_arithmetic<T>::value, T>::type>
        T random_number(T min = std::numeric_limits<T>::min(), T max = std::numeric_limits<T>::max())
        {
        std::random_device rd;
        std::mt19937 gen(rd());
        std::uniform_real_distribution<> dist(min, max);

        return dist(gen);
        }

        template<typename T,typename = typename std::enable_if<std::is_arithmetic<T>::value, T>::type>
        std::vector<T> random_number_vector(
        size_t length,
        T min = std::numeric_limits<T>::min(),
        T max = std::numeric_limits<T>::max())
        {
        auto randnum = [&]() -> T
            {
            return utils::random_number<T>(min, max);
            };

        std::vector<T> out(length);
        std::generate(out.begin(), out.end(), randnum);

        return out;
        }

        // MSG GEN =========================================================================================
        std_msgs::msg::Header random_header(size_t len = 16)
        {
        std_msgs::msg::Header header_msg;
        header_msg.stamp.sec = utils::random_number<int32_t>();
        header_msg.stamp.nanosec = utils::random_number<int32_t>();
        header_msg.frame_id = random_string(len);

        return header_msg;
        }

        geometry_msgs::msg::Quaternion random_quaternion()
        {
        geometry_msgs::msg::Quaternion q_msg;

        q_msg.x = utils::random_number<double>();
        q_msg.y = utils::random_number<double>();
        q_msg.z = utils::random_number<double>();
        q_msg.w = utils::random_number<double>();

        return q_msg;
        }

        geometry_msgs::msg::Point random_point()
        {
        geometry_msgs::msg::Point p_msg;

        p_msg.x = utils::random_number<double>();
        p_msg.y = utils::random_number<double>();
        p_msg.z = utils::random_number<double>();

        return p_msg;
        }

        geometry_msgs::msg::Pose random_pose()
        {
        geometry_msgs::msg::Pose pose_msg;

        pose_msg.position = utils::random_point();
        pose_msg.orientation = utils::random_quaternion();

        return pose_msg;
        }

        geometry_msgs::msg::Vector3 random_vector3()
        {
        geometry_msgs::msg::Vector3 vec_msg;

        vec_msg.x = utils::random_number<double>();
        vec_msg.y = utils::random_number<double>();
        vec_msg.z = utils::random_number<double>();

        return vec_msg;
        }

        geometry_msgs::msg::Vector3Stamped random_vector3stamped()
        {
        geometry_msgs::msg::Vector3Stamped vec_msg;

        vec_msg.header = utils::random_header();
        vec_msg.vector = utils::random_vector3();

        return vec_msg;
        }

        geometry_msgs::msg::Twist random_twist()
        {
        geometry_msgs::msg::Twist twist_msg;

        twist_msg.linear = utils::random_vector3();
        twist_msg.angular = utils::random_vector3();

        return twist_msg;
        }

        geometry_msgs::msg::TwistWithCovariance random_twistwithcovariance(size_t len = 36)
        {
        geometry_msgs::msg::TwistWithCovariance twist_msg;

        std::array<double,36> arr;
        auto covariance = utils::random_number_vector<double>(len);
        std::copy_n(covariance.begin(), 36, arr.begin());

        twist_msg.twist = utils::random_twist();
        twist_msg.covariance = arr;

        return twist_msg;
        }

        geometry_msgs::msg::TwistWithCovarianceStamped random_twistwithcovariancestamped(size_t len = 36)
        {
        geometry_msgs::msg::TwistWithCovarianceStamped twist_msg;

        twist_msg.header = utils::random_header();
        twist_msg.twist = utils::random_twistwithcovariance(len);

        return twist_msg;
        }

        geometry_msgs::msg::Wrench random_wrench()
        {
        geometry_msgs::msg::Wrench wrench_msg;

        wrench_msg.force = utils::random_vector3();
        wrench_msg.torque = utils::random_vector3();

        return wrench_msg;
        }

        geometry_msgs::msg::WrenchStamped random_wrenchstamped()
        {
        geometry_msgs::msg::WrenchStamped wrench_msg;

        wrench_msg.header = utils::random_header();
        wrench_msg.wrench = utils::random_wrench();

        return wrench_msg;
        }

        sensor_msgs::msg::Image random_image(size_t len = 0)  // 1920 * 1080 * 3
        {
        sensor_msgs::msg::Image image_msg;

        image_msg.header = utils::random_header();

        image_msg.height = utils::random_number<uint32_t>();
        image_msg.width = utils::random_number<uint32_t>();

        image_msg.encoding = utils::random_string(32);
        image_msg.is_bigendian = utils::random_number<int>(0, 1);
        image_msg.step = utils::random_number<uint32_t>();

        image_msg.data = utils::random_number_vector<uint8_t>(len);

        return image_msg;
        }

        sensor_msgs::msg::PointField random_pointfield(size_t len = 32)
        {
        sensor_msgs::msg::PointField pt_msg;

        pt_msg.name = utils::random_string(len);
        pt_msg.offset = utils::random_number<uint32_t>();
        pt_msg.datatype = utils::random_number<uint8_t>();
        pt_msg.count = utils::random_number<uint32_t>(0, 1);

        return pt_msg;
        }

        sensor_msgs::msg::PointCloud2 random_pointcloud(size_t len = 0)  // 4 * 4 * 4 * 1280 * 960
        {
        sensor_msgs::msg::PointCloud2 pc_msg;

        pc_msg.header = utils::random_header();

        pc_msg.height = utils::random_number<uint32_t>();
        pc_msg.width = utils::random_number<uint32_t>();

        std::vector<sensor_msgs::msg::PointField> pts(3);
        for (int i = 0; i < 3; ++i) {
            pts.push_back(random_pointfield());
        }

        pc_msg.fields = pts;

        pc_msg.is_bigendian = utils::random_number<int>(0, 1);
        pc_msg.point_step = utils::random_number<uint32_t>();
        pc_msg.row_step = utils::random_number<uint32_t>();

        pc_msg.data = utils::random_number_vector<uint8_t>(len);

        pc_msg.is_dense = utils::random_number<int>(0, 1);

        return pc_msg;
        }

        sensor_msgs::msg::LaserScan random_laserscan(size_t len = 1024)
        {
        sensor_msgs::msg::LaserScan scan_msg;

        scan_msg.header = utils::random_header();

        scan_msg.angle_min = utils::random_number<float>();
        scan_msg.angle_max = utils::random_number<float>();
        scan_msg.angle_increment = utils::random_number<float>();
        scan_msg.time_increment = utils::random_number<float>();
        scan_msg.scan_time = utils::random_number<float>();
        scan_msg.range_min = utils::random_number<float>();
        scan_msg.range_max = utils::random_number<float>();

        scan_msg.ranges = utils::random_number_vector<float>(len);
        scan_msg.intensities = utils::random_number_vector<float>(len);

        return scan_msg;
        }
    }
}
