#!/usr/bin/env sh

CURRENT_DIR=$(readlink -f $(dirname $(dirname $0)))
colcon build --cmake-args -DCMAKE_BUILD_TYPE=Release
cp ${CURRENT_DIR}/build/nodegraph/libnodegraph.so ${CURRENT_DIR}/lib/nodegraph/libnodegraph.so
cp ${CURRENT_DIR}/build/nodegraph/libnodegraph.so ${CURRENT_DIR}/../../target/debug
