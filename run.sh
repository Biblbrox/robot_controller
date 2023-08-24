#!/usr/bin/env sh

. /opt/ros/humble/setup.sh
export LD_LIBRARY_PATH=./src/c/lib/nodegraph:$LD_LIBRARY_PATH
export ROS_DISCOVERY_SERVER="127.0.0.1:11811"
#export RMW_FASTRTPS_USE_QOS_FROM_XML=1
export FASTRTPS_DEFAULT_PROFILES_FILE=super_client_configuration_file.xml
echo "Stoping ros2 daemon..."
ros2 daemon stop
echo "Starting ros2 daemon..."
ros2 daemon start
cargo run
