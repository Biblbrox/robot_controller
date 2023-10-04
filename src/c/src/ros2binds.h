/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

void rclcpp_init(int argc, char const *const argv[]);
int rclcpp_shutdown();

#ifdef __cplusplus
}
#endif