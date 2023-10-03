/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod ros2 {
    use std::env::Args;
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int};
    use crate::discovery_server_impl::{rclcpp_init, rclcpp_shutdown};

    pub fn init(args: Args) {
        let args = args.map(|arg| CString::new(arg).unwrap()).collect::<Vec<CString>>();
        // convert the strings to raw pointers
        let c_args = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<*const c_char>>();
        unsafe { rclcpp_init(args.len() as c_int, c_args.as_ptr()); }
    }

    pub fn shutdown() -> bool {
        unsafe { return rclcpp_shutdown() == 1; }
    }
}