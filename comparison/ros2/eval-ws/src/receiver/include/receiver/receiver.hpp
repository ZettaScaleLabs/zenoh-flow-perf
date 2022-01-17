#include <rclcpp/rclcpp.hpp>
#include <chrono>
#include <vector>
#include "eval_interfaces/msg/evaluation.hpp"

namespace ato {
    namespace receiver {
        class Receiver : public rclcpp::Node {

            public:
                explicit Receiver(const uint64_t msgs, const std::string topic_name, const uint64_t pipeline_length);
                virtual ~Receiver() {};

            private:
                rclcpp::Subscription<eval_interfaces::msg::Evaluation>::SharedPtr subscriber;
                uint64_t msgs;
                uint64_t pipeline_length;
                void receiver_callback(const eval_interfaces::msg::Evaluation::SharedPtr msg);

        };
    }
}