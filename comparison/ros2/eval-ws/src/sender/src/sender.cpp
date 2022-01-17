
#include <chrono>
#include <vector>
#include "sender/sender.hpp"
#include "eval_interfaces/msg/evaluation.hpp"

using ato::sender::Sender;

Sender::Sender(const uint64_t msgs) : rclcpp::Node("sender_node", rclcpp::NodeOptions().use_intra_process_comms(true)) {

    auto pace = std::chrono::duration<double>(1.0/double(msgs));
    this->publisher = this->create_publisher<eval_interfaces::msg::Evaluation>("out_0", 1);
    this->timer = this->create_wall_timer(pace, std::bind(&Sender::publish_message, this));

    RCLCPP_INFO(this->get_logger(), "Init Sender msg/s %d pace is %f", msgs, pace);

}


void Sender::publish_message() {
    auto message = eval_interfaces::msg::Evaluation();
    auto ts = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::system_clock::now().time_since_epoch());
    message.emitter_ts = ts.count();

    //RCLCPP_INFO(this->get_logger(), "Publish!, %ul", message.emitter_ts);
    this->publisher->publish(message);

}