#!/usr/bin/env sh

DIR="$( cd "$( dirname "$0" )" && pwd )"
bindgen ${DIR}/bindings.h > ${DIR}/../discovery_server_impl.rs