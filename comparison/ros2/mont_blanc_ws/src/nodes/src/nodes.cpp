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

#include "nodes.hpp"
#include "utils.hpp"
#include <algorithm>
#include <stdexcept>
#include <random>

using ato::nodes::Arequipa;
using ato::nodes::Barcelona;
using ato::nodes::Cordoba;
using ato::nodes::Delhi;
using ato::nodes::Freeport;
using ato::nodes::Geneva;
using ato::nodes::Georgetown;
using ato::nodes::Hamburg;
using ato::nodes::Hebron;
using ato::nodes::Kingston;
using ato::nodes::Lyon;
using ato::nodes::Madellin;
using ato::nodes::Mandalay;
using ato::nodes::Monaco;
using ato::nodes::Osaka;
using ato::nodes::Ponce;
using ato::nodes::Portsmouth;
using ato::nodes::Rotterdam;
using ato::nodes::Taipei;
using ato::nodes::Tripoli;


// Arequipa

Arequipa::Arequipa(const std::string file_name) : rclcpp::Node("arqequipa", rclcpp::NodeOptions().use_intra_process_comms(false)) {

    auto qos_sub = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->subscriber = this->create_subscription<std_msgs::msg::String>("/arkansas", qos_sub, std::bind(&Arequipa::receiver_callback, this, std::placeholders::_1));
    this->outfile.open(file_name);
}

Arequipa::~Arequipa() {
    this->outfile.close();
}

void Arequipa::receiver_callback(const std_msgs::msg::String::SharedPtr msg) {
    this->outfile << msg->data << std::endl;

}


// Barcellona

Barcelona::Barcelona() : rclcpp::Node("arqequipa", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos_pub = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    auto qos_sub = rclcpp::QoS(rclcpp::KeepAll()).reliable();


    this->publisher = this->create_publisher<geometry_msgs::msg::WrenchStamped>("/lena", qos_pub);
    this->subscriber = this->create_subscription<geometry_msgs::msg::TwistWithCovarianceStamped>("/mekong", qos_sub, std::bind(&Barcelona::receiver_callback, this, std::placeholders::_1));
}

void Barcelona::receiver_callback(const geometry_msgs::msg::TwistWithCovarianceStamped::SharedPtr msg) {
    geometry_msgs::msg::WrenchStamped new_msg = geometry_msgs::msg::WrenchStamped();
    new_msg.header = msg->header;
    new_msg.wrench.force = msg->twist.twist.linear;
    new_msg.wrench.torque = msg->twist.twist.angular;

    this->publish_message(new_msg);

}


void Barcelona::publish_message(const geometry_msgs::msg::WrenchStamped msg) {
    this->publisher->publish(msg);
}

// Cordoba

Cordoba::Cordoba() : rclcpp::Node("cordoba", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->pace = std::chrono::duration<double>(0.1); // 100ms
    this->publisher = this->create_publisher<std_msgs::msg::Float32>("/amazon", qos);
    this->timer = this->create_wall_timer(pace, std::bind(&Cordoba::publish_message, this));
    this->data = std_msgs::msg::Float32();
    this->data.data = ato::utils::random_number<double>();
    RCLCPP_INFO(this->get_logger(), "Init Cordoba pace is %f", pace.count());

}

void Cordoba::publish_message() {

        //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
        this->publisher->publish(this->data);


}

// Delhi

Delhi::Delhi() : rclcpp::Node("delhi", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->pace = std::chrono::duration<double>(1.0); // 1s
    this->publisher = this->create_publisher<sensor_msgs::msg::Image>("/columbia", qos);
    this->timer = this->create_wall_timer(pace, std::bind(&Delhi::publish_message, this));
    this->data = ato::utils::random_image();
    RCLCPP_INFO(this->get_logger(), "Init Delhi pace is %f", pace.count());

}

void Delhi::publish_message() {

        //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
        this->publisher->publish(this->data);
}


// Freeport

Freeport::Freeport() : rclcpp::Node("freeport", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->pace = std::chrono::duration<double>(0.05); // 50ms
    this->publisher = this->create_publisher<std_msgs::msg::Int64>("/ganges", qos);
    this->timer = this->create_wall_timer(pace, std::bind(&Freeport::publish_message, this));
    this->data = std_msgs::msg::Int64();
    this->data.data = ato::utils::random_number<int64_t>();
    RCLCPP_INFO(this->get_logger(), "Init Freeport pace is %f", pace.count());

}

void Freeport::publish_message() {

        //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
        this->publisher->publish(this->data);
}

// Geneva

Geneva::Geneva() : rclcpp::Node("geneva", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<std_msgs::msg::String>("/arkansas", qos);
    this->subscriber_parana = this->create_subscription<std_msgs::msg::String>("/parana", qos, std::bind(&Geneva::receiver_parana, this, std::placeholders::_1));
    this->subscriber_danube = this->create_subscription<std_msgs::msg::String>("/danube", qos, std::bind(&Geneva::receiver_danube, this, std::placeholders::_1));
    this->subscriber_tagus = this->create_subscription<geometry_msgs::msg::Pose>("/tagus", qos, std::bind(&Geneva::receiver_tagus, this, std::placeholders::_1));
    this->subscriber_congo = this->create_subscription<geometry_msgs::msg::Twist>("/congo", qos, std::bind(&Geneva::receiver_congo, this, std::placeholders::_1));

    this->data = std_msgs::msg::String();
    this->data.data = "geneva/arkansas";

}

void Geneva::publish_message() {
    std_msgs::msg::String msg  = std_msgs::msg::String();
    msg.data = this->data.data + ":" +this->parana_val.data;
    this->publisher->publish(msg);
}

void Geneva::receiver_parana(const std_msgs::msg::String::SharedPtr msg) {
    this->parana_val = *msg;
    this->publish_message();
}
void Geneva::receiver_danube(const std_msgs::msg::String::SharedPtr msg) {
    this->danube_val = *msg;
}
void Geneva::receiver_tagus(const geometry_msgs::msg::Pose::SharedPtr msg) {
    this->tagus_val = *msg;
}
void Geneva::receiver_congo(const geometry_msgs::msg::Twist::SharedPtr msg) {
    this->congo_val = *msg;
}

// Georgetown

Georgetown::Georgetown() : rclcpp::Node("georgetown", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<std_msgs::msg::Float64>("/volga", qos);
    this->subscriber_murray = this->create_subscription<geometry_msgs::msg::Vector3Stamped>("/murray", qos, std::bind(&Georgetown::receiver_murray, this, std::placeholders::_1));
    this->subscriber_lena = this->create_subscription<geometry_msgs::msg::WrenchStamped>("/lena", qos, std::bind(&Georgetown::receiver_lena, this, std::placeholders::_1));

    this->pace = std::chrono::duration<double>(0.05); // 50ms
    this->timer = this->create_wall_timer(pace, std::bind(&Georgetown::publish_message, this));

    this->data = std_msgs::msg::Float64();
    this->data.data = ato::utils::random_number<double>();

}

void Georgetown::publish_message() {
        this->publisher->publish(this->data);
}

void Georgetown::receiver_murray(const geometry_msgs::msg::Vector3Stamped::SharedPtr msg) {
    this->murray_val = *msg;
}
void Georgetown::receiver_lena(const geometry_msgs::msg::WrenchStamped::SharedPtr msg) {
    this->lena_val = *msg;
}

// Hamburg


Hamburg::Hamburg() : rclcpp::Node("hamburg", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<std_msgs::msg::String>("/parana", qos);
    this->subscriber_tigris = this->create_subscription<std_msgs::msg::Float32>("/tigris", qos, std::bind(&Hamburg::receiver_tigris, this, std::placeholders::_1));
    this->subscriber_ganges = this->create_subscription<std_msgs::msg::Int64>("/ganges", qos, std::bind(&Hamburg::receiver_ganges, this, std::placeholders::_1));
    this->subscriber_nile = this->create_subscription<std_msgs::msg::Int32>("/nile", qos, std::bind(&Hamburg::receiver_nile, this, std::placeholders::_1));
    this->subscriber_danube = this->create_subscription<std_msgs::msg::String>("/danube", qos, std::bind(&Hamburg::receiver_danube, this, std::placeholders::_1));

    this->data = std_msgs::msg::String();
    this->data.data = "hamburg/parana";

}

void Hamburg::publish_message() {
        this->publisher->publish(this->data);
}

void Hamburg::receiver_tigris(const std_msgs::msg::Float32::SharedPtr msg) {
    this->tigris_val = *msg;
}
void Hamburg::receiver_ganges(const std_msgs::msg::Int64::SharedPtr msg) {
    this->ganges_val = *msg;
}
void Hamburg::receiver_nile(const std_msgs::msg::Int32::SharedPtr msg) {
    this->nile_val = *msg;
}
void Hamburg::receiver_danube(const std_msgs::msg::String::SharedPtr msg) {
    this->danube_val = *msg;
    this->publish_message();
}

// Hebron


Hebron::Hebron() : rclcpp::Node("hebron", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->pace = std::chrono::duration<double>(0.1); // 100ms
    this->publisher = this->create_publisher<geometry_msgs::msg::Quaternion>("/chenab", qos);
    this->timer = this->create_wall_timer(pace, std::bind(&Hebron::publish_message, this));
    this->data = ato::utils::random_quaternion();
    RCLCPP_INFO(this->get_logger(), "Init Hebron pace is %f", pace.count());

}

void Hebron::publish_message() {

        //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
        this->publisher->publish(this->data);
}

// Kingston

Kingston::Kingston() : rclcpp::Node("kingston", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->pace = std::chrono::duration<double>(0.1); // 100ms
    this->publisher = this->create_publisher<geometry_msgs::msg::Vector3>("/chenab", qos);
    this->timer = this->create_wall_timer(pace, std::bind(&Kingston::publish_message, this));
    this->data = ato::utils::random_vector3();
    RCLCPP_INFO(this->get_logger(), "Init Kingston pace is %f", pace.count());

}

void Kingston::publish_message() {

        //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
        this->publisher->publish(this->data);
}

// Lyon

Lyon::Lyon() : rclcpp::Node("lyon", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<std_msgs::msg::Float32>("/tigris", qos);
    this->subscriber_amazon = this->create_subscription<std_msgs::msg::Float32>("/amazon", qos, std::bind(&Lyon::receiver_amazon, this, std::placeholders::_1));

}

void Lyon::publish_message(const std_msgs::msg::Float32::SharedPtr msg) {
    this->publisher->publish(*msg);
}

void Lyon::receiver_amazon(const std_msgs::msg::Float32::SharedPtr msg) {
    this->publish_message(msg);
}

// Madellin


Madellin::Madellin() : rclcpp::Node("madellin", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->pace = std::chrono::duration<double>(0.01); // 10ms
    this->publisher = this->create_publisher<std_msgs::msg::Int32>("/nile", qos);
    this->timer = this->create_wall_timer(pace, std::bind(&Madellin::publish_message, this));
    this->data = std_msgs::msg::Int32();
    this->data.data = ato::utils::random_number<int32_t>();
    RCLCPP_INFO(this->get_logger(), "Init Madellin pace is %f", pace.count());

}

void Madellin::publish_message() {

        //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
        this->publisher->publish(this->data);
}


// Mandalay

Mandalay::Mandalay() : rclcpp::Node("mandalay", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher_brazos = this->create_publisher<sensor_msgs::msg::PointCloud2>("/brazos", qos);
    this->publisher_tagus = this->create_publisher<geometry_msgs::msg::Pose>("/tagus", qos);
    this->publisher_missouri = this->create_publisher<sensor_msgs::msg::Image>("/missouri", qos);

    this->subscriber_danube = this->create_subscription<std_msgs::msg::String>("/danube", qos, std::bind(&Mandalay::receiver_danube, this, std::placeholders::_1));
    this->subscriber_chenab = this->create_subscription<geometry_msgs::msg::Quaternion>("/chenab", qos, std::bind(&Mandalay::receiver_chenab, this, std::placeholders::_1));
    this->subscriber_salween = this->create_subscription<sensor_msgs::msg::PointCloud2>("/salween", qos, std::bind(&Mandalay::receiver_salween, this, std::placeholders::_1));
    this->subscriber_godavari = this->create_subscription<sensor_msgs::msg::LaserScan>("/godavari", qos, std::bind(&Mandalay::receiver_godavari, this, std::placeholders::_1));
    this->subscriber_loire = this->create_subscription<sensor_msgs::msg::PointCloud2>("/loire", qos, std::bind(&Mandalay::receiver_loire, this, std::placeholders::_1));
    this->subscriber_yamuna = this->create_subscription<geometry_msgs::msg::Vector3>("/yamuna", qos, std::bind(&Mandalay::receiver_yamuna, this, std::placeholders::_1));

    this->pace = std::chrono::duration<double>(0.1); // 100ms
    this->timer = this->create_wall_timer(pace, std::bind(&Mandalay::publish_message, this));

    this->tagus_data = ato::utils::random_pose();
    this->missouri_data = ato::utils::random_image();
    this->loire_val = ato::utils::random_pointcloud();

}

void Mandalay::publish_message() {
        this->publisher_brazos->publish(this->loire_val);
        this->publisher_tagus->publish(this->tagus_data);
        this->publisher_missouri->publish(this->missouri_data);
}

void Mandalay::receiver_danube(const std_msgs::msg::String::SharedPtr msg) {
    this->danube_val = *msg;
}
void Mandalay::receiver_chenab(const geometry_msgs::msg::Quaternion::SharedPtr msg) {
    this->chenab_val = *msg;
}
void Mandalay::receiver_salween(const sensor_msgs::msg::PointCloud2::SharedPtr msg) {
    this->salween_val = *msg;
}
void Mandalay::receiver_godavari(const sensor_msgs::msg::LaserScan::SharedPtr msg){
    this->godavari_val = *msg;
}
void Mandalay::receiver_loire(const sensor_msgs::msg::PointCloud2::SharedPtr msg){
    this->loire_val = *msg;
}
void Mandalay::receiver_yamuna(const geometry_msgs::msg::Vector3::SharedPtr msg){
    this->yamuna_val = *msg;
}

// Monaco

Monaco::Monaco() : rclcpp::Node("monaco", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<std_msgs::msg::Float32>("/ohio", qos);
    this->subscriber_congo = this->create_subscription<geometry_msgs::msg::Twist>("/congo", qos, std::bind(&Monaco::receiver_congo, this, std::placeholders::_1));
    this->data =  std_msgs::msg::Float32();
    this->data.data = ato::utils::random_number<float>();
}

void Monaco::publish_message() {
    this->publisher->publish(this->data);
}

void Monaco::receiver_congo(const geometry_msgs::msg::Twist::SharedPtr msg) {
    this->congo_val = *msg;
    this->publish_message();
}

// Osaka


Osaka::Osaka() : rclcpp::Node("osaka", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher_salween = this->create_publisher<sensor_msgs::msg::PointCloud2>("/salween", qos);
    this->publisher_godavari = this->create_publisher<sensor_msgs::msg::LaserScan>("/godavari", qos);


    this->subscriber_parana = this->create_subscription<std_msgs::msg::String>("/parana", qos, std::bind(&Osaka::receiver_parana, this, std::placeholders::_1));
    this->subscriber_columbia = this->create_subscription<sensor_msgs::msg::Image>("/columbia", qos, std::bind(&Osaka::receiver_columbia, this, std::placeholders::_1));
    this->subscriber_colorado = this->create_subscription<sensor_msgs::msg::Image>("/colorado", qos, std::bind(&Osaka::receiver_colorado, this, std::placeholders::_1));


    this->godavari_data = ato::utils::random_laserscan();
    this->salween_data = ato::utils::random_pointcloud();

}

void Osaka::publish_message() {
        this->publisher_salween->publish(this->salween_data);
        this->publisher_godavari->publish(this->godavari_data);

}

void Osaka::receiver_parana(const std_msgs::msg::String::SharedPtr msg) {
    this->parana_val = *msg;
}
void Osaka::receiver_columbia(const sensor_msgs::msg::Image::SharedPtr msg) {
    this->columbia_val = *msg;
}
void Osaka::receiver_colorado(const sensor_msgs::msg::Image::SharedPtr msg) {
    this->colorado_val = *msg;
    this->publish_message();
}

// Ponce

Ponce::Ponce() : rclcpp::Node("ponce", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher_congo = this->create_publisher<geometry_msgs::msg::Twist>("/congo", qos);
    this->publisher_mekong = this->create_publisher<geometry_msgs::msg::TwistWithCovarianceStamped>("/mekong", qos);


    this->subscriber_danube = this->create_subscription<std_msgs::msg::String>("/danube", qos, std::bind(&Ponce::receiver_danube, this, std::placeholders::_1));
    this->subscriber_tagus = this->create_subscription<geometry_msgs::msg::Pose>("/tagus", qos, std::bind(&Ponce::receiver_tagus, this, std::placeholders::_1));
    this->subscriber_missouri = this->create_subscription<sensor_msgs::msg::Image>("/missouri", qos, std::bind(&Ponce::receiver_missouri, this, std::placeholders::_1));
    this->subscriber_loire = this->create_subscription<sensor_msgs::msg::PointCloud2>("/loire", qos, std::bind(&Ponce::receiver_loire, this, std::placeholders::_1));
    this->subscriber_yamuna = this->create_subscription<geometry_msgs::msg::Vector3>("/yamuna", qos, std::bind(&Ponce::receiver_yamuna, this, std::placeholders::_1));
    this->subscriber_ohio = this->create_subscription<std_msgs::msg::Float32>("/ohio", qos, std::bind(&Ponce::receiver_ohio, this, std::placeholders::_1));
    this->subscriber_volga = this->create_subscription<std_msgs::msg::Float64>("/volga", qos, std::bind(&Ponce::receiver_volga, this, std::placeholders::_1));
    this->subscriber_brazos = this->create_subscription<sensor_msgs::msg::PointCloud2>("/brazos", qos, std::bind(&Ponce::receiver_brazos, this, std::placeholders::_1));


    this->congo_data = ato::utils::random_twist();
    this->mekong_data = ato::utils::random_twistwithcovariancestamped();

}

void Ponce::publish_message() {
        this->publisher_congo->publish(this->congo_data);
        this->publisher_mekong->publish(this->mekong_data);
}

void Ponce::receiver_danube(const std_msgs::msg::String::SharedPtr msg){
this->danube_val = *msg;
}
void Ponce::receiver_tagus(const geometry_msgs::msg::Pose::SharedPtr msg){
this->tagus_val = *msg;
}
void Ponce::receiver_missouri(const sensor_msgs::msg::Image::SharedPtr msg){
this->missouri_val = *msg;
}
void Ponce::receiver_loire(const sensor_msgs::msg::PointCloud2::SharedPtr msg){
this->loire_val = *msg;
}
void Ponce::receiver_yamuna(const geometry_msgs::msg::Vector3::SharedPtr msg){
this->yamuna_val = *msg;
}
void Ponce::receiver_ohio(const std_msgs::msg::Float32::SharedPtr msg){
this->ohio_val = *msg;
}
void Ponce::receiver_volga(const std_msgs::msg::Float64::SharedPtr msg){
this->volga_val = *msg;
}
void Ponce::receiver_brazos(const sensor_msgs::msg::PointCloud2::SharedPtr msg){
    this->brazos_val = *msg;
    this->publish_message();
}


// Portsmouth

Portsmouth::Portsmouth() : rclcpp::Node("portsmouth", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->pace = std::chrono::duration<double>(0.2); // 200ms
    this->publisher = this->create_publisher<std_msgs::msg::String>("/danube", qos);
    this->timer = this->create_wall_timer(pace, std::bind(&Portsmouth::publish_message, this));
    this->data = std_msgs::msg::String();
    this->data.data = "portsmouth/danube";
    RCLCPP_INFO(this->get_logger(), "Init Portsmouth pace is %f", pace.count());

}

void Portsmouth::publish_message() {

        //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
        this->publisher->publish(this->data);
}

// Rotterdam

Rotterdam::Rotterdam() : rclcpp::Node("rotterdam", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<geometry_msgs::msg::Vector3Stamped>("/murray", qos);
    this->subscriber_mekong = this->create_subscription<geometry_msgs::msg::TwistWithCovarianceStamped>("/mekong", qos, std::bind(&Rotterdam::receiver_mekong, this, std::placeholders::_1));
    this->data = ato::utils::random_vector3stamped();
}

void Rotterdam::publish_message() {
    this->publisher->publish(this->data);
}

void Rotterdam::receiver_mekong(const geometry_msgs::msg::TwistWithCovarianceStamped::SharedPtr msg) {
    this->mekong_val = *msg;
    this->publish_message();
}


// Taipei

Taipei::Taipei() : rclcpp::Node("taipei", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<sensor_msgs::msg::Image>("/colorado", qos);
    this->subscriber_columbia = this->create_subscription<sensor_msgs::msg::Image>("/columbia", qos, std::bind(&Taipei::receiver_columbia, this, std::placeholders::_1));
}

void Taipei::publish_message() {
    this->publisher->publish(this->data);
}

void Taipei::receiver_columbia(const sensor_msgs::msg::Image::SharedPtr msg) {
    this->data = *msg;
    this->publish_message();
}


// Tripoli

Tripoli::Tripoli() : rclcpp::Node("tripoli", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();

    this->publisher = this->create_publisher<sensor_msgs::msg::PointCloud2>("/loire", qos);
    this->subscriber_columbia = this->create_subscription<sensor_msgs::msg::Image>("/columbia", qos, std::bind(&Tripoli::receiver_columbia, this, std::placeholders::_1));
    this->subscriber_godavari = this->create_subscription<sensor_msgs::msg::LaserScan>("/godavari", qos, std::bind(&Tripoli::receiver_godavari, this, std::placeholders::_1));
    this->data = ato::utils::random_pointcloud();
}

void Tripoli::publish_message() {
    this->publisher->publish(this->data);
}

void Tripoli::receiver_columbia(const sensor_msgs::msg::Image::SharedPtr msg) {
    this->columbia_val = *msg;
}

void Tripoli::receiver_godavari(const sensor_msgs::msg::LaserScan::SharedPtr msg) {
    this->godavari_val = *msg;
    this->publish_message();
}