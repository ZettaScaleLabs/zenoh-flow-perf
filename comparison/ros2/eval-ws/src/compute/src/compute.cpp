
#include <chrono>
#include <vector>
#include "compute/compute.hpp"
#include "eval_interfaces/msg/evaluation.hpp"

using ato::compute::Compute;

Compute::Compute(const std::string listen_topic, const std::string publish_topic) : rclcpp::Node("compute_node", rclcpp::NodeOptions().use_intra_process_comms(true)) {


    this->publisher = this->create_publisher<eval_interfaces::msg::Evaluation>(publish_topic, 1);
    this->subscriber = this->create_subscription<eval_interfaces::msg::Evaluation>(listen_topic, 1, std::bind(&Compute::receiver_callback, this, std::placeholders::_1));

    RCLCPP_INFO(this->get_logger(), "Init Compute with listener %s and publisher %s", listen_topic.c_str(), publish_topic.c_str());

}


void Compute::receiver_callback(const eval_interfaces::msg::Evaluation::SharedPtr msg) {
     this->publish_message(msg);
}


void Compute::publish_message(const eval_interfaces::msg::Evaluation::SharedPtr msg) {
    this->publisher->publish(*msg);
}
