# Description
Server part of ros2 dashboard project. The project itself contains two parts. First is the main part, which is written with Rust. The second is the implementation
of FastDDS api written via C++-17 with C bindings for Rust. 

For now, the parameters of ROS2 environment are hardcoded:
 - ROS_DOMAIN_ID=1
 - ROS_DISCOVERY_SERVER="0.0.0.0:11811"
 - FASTRTPS_DEFAULT_PROFILES_FILE=super_client_configuration_file.xml

The last parameter is necessary for right work of ROS2 cli tools.

You can read about the project more in my note: 

**CAUTION**: This is only prototype of my idea, without any guaranties for a stable work.
