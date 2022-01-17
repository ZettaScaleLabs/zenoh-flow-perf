#include <rclcpp/rclcpp.hpp>
#include <argparse/argparse.hpp>
#include "compute/compute.hpp"

int main (int argc, char* argv[]) {

    argparse::ArgumentParser program("compute");

    program.add_argument("input")
        .help("input topic");
        // .scan<'s', std::string>();

    program.add_argument("output")
        .help("output topic");
        // .scan<'s', std::string>();

    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    std::string input = program.get<std::string>("input");
    std::string output = program.get<std::string>("output");

    rclcpp::init(argc, argv);
    rclcpp::executors::SingleThreadedExecutor executor;


    auto compute = std::make_shared<ato::compute::Compute>(input, output);
    executor.add_node(compute);

    executor.spin();

    rclcpp::shutdown();

    return 0;


}