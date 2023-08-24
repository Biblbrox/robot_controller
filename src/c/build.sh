#!/usr/bin/env sh

CURRENT_DIR=$(readlink -f $(dirname $(dirname $0)))
colcon build
cp ${CURRENT_DIR}/build/nodegraph/libnodegraph.so ${CURRENT_DIR}/lib/nodegraph/libnodegraph.so