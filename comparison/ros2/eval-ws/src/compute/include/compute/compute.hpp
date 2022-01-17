#include <rclcpp/rclcpp.hpp>
#include <chrono>
#include <vector>
#include "eval_interfaces/msg/evaluation.hpp"

namespace ato {
    namespace compute {
        class Compute : public rclcpp::Node {

            public:
                explicit Compute(const std::string listen_topic, const std::string publish_topic);
                virtual ~Compute() {}

            private:
                rclcpp::Publisher<eval_interfaces::msg::Evaluation>::SharedPtr publisher;
                rclcpp::Subscription<eval_interfaces::msg::Evaluation>::SharedPtr subscriber;
                void receiver_callback(const eval_interfaces::msg::Evaluation::SharedPtr msg);
                void publish_message(const eval_interfaces::msg::Evaluation::SharedPtr msg);

        };
    }
}