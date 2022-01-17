#include <rclcpp/rclcpp.hpp>
#include <argparse/argparse.hpp>
#include "sender/sender.hpp"

int main (int argc, char* argv[]) {


    argparse::ArgumentParser program("sender");

    program.add_argument("msgs")
        .help("messages per second")
        .scan<'i', uint64_t>();

    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    uint64_t msgs = program.get<uint64_t>("msgs");

    rclcpp::init(argc, argv);
    rclcpp::executors::SingleThreadedExecutor executor;


    auto sender = std::make_shared<ato::sender::Sender>(msgs);
    executor.add_node(sender);

    executor.spin();

    rclcpp::shutdown();

    return 0;


}