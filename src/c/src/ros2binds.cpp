#include <rclcpp/rclcpp.hpp>

#include "ros2binds.h"

void rclcpp_init(int argc, const char *const *argv) {
  rclcpp::init(argc, argv, rclcpp::InitOptions(), rclcpp::SignalHandlerOptions::None);
}
int rclcpp_shutdown() { return rclcpp::shutdown(); }
