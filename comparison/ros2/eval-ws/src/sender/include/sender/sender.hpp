#include <rclcpp/rclcpp.hpp>
#include <chrono>
#include <vector>
#include "eval_interfaces/msg/evaluation.hpp"

namespace ato {
    namespace sender {
        class Sender : public rclcpp::Node {

            public:
                explicit Sender(const uint64_t msgs);
                virtual ~Sender() {}

            private:
                rclcpp::Publisher<eval_interfaces::msg::Evaluation>::SharedPtr publisher;
                rclcpp::TimerBase::SharedPtr timer;
                void publish_message();

        };
    }
}