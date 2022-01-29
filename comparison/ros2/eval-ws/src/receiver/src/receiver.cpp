
#include <chrono>
#include <vector>
#include <iostream>

#include "eval_interfaces/msg/evaluation.hpp"
#include "receiver/receiver.hpp"


using ato::receiver::Receiver;

Receiver::Receiver(const uint64_t msgs, const std::string topic_name, const uint64_t pipeline_length) : rclcpp::Node("receiver_node", rclcpp::NodeOptions().use_intra_process_comms(false)) {
    auto qos = rclcpp::QoS(rclcpp::KeepAll()).reliable();
    this->msgs = msgs;
    this->pipeline_length = pipeline_length;
    this->subscriber = this->create_subscription<eval_interfaces::msg::Evaluation>(topic_name, qos, std::bind(&Receiver::receiver_callback, this, std::placeholders::_1));

    // RCLCPP_INFO(this->get_logger(), "Init Receiver with msg/s %d, pipeline lenght is %d, topic name is %s", this->msgs,  this->pipeline_length, topic_name.c_str());

}


void Receiver::receiver_callback(const eval_interfaces::msg::Evaluation::SharedPtr msg) {

    auto ts = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::system_clock::now().time_since_epoch()).count();
    auto latency = ts - msg->emitter_ts;
    // RCLCPP_INFO(this->get_logger(), "Receiver!, %ul - %ul = %dus", msg->emitter_ts, ts, latency);


    // layer,scenario name,test kind, test name, payload size, msg/s, pipeline size, latency, unit
    std::cout << "ros2,scenario,latency,pipeline," << this->msgs << "," << this->pipeline_length << "," << latency << ",us" << std::endl << std::flush;

}