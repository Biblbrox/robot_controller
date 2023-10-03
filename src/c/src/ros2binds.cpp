/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

#include <rclcpp/rclcpp.hpp>

#include "ros2binds.h"

void rclcpp_init(int argc, const char *const *argv) {
  rclcpp::init(argc, argv, rclcpp::InitOptions(), rclcpp::SignalHandlerOptions::None);
}
int rclcpp_shutdown() { return rclcpp::shutdown(); }
