#pragma once

#include <cstddef>

#include <fastdds/rtps/common/Locator.h>
#include "discovery_server.h"

namespace r2discoverer {
void parse_endpoint_fastdds(FastDDSEndpoint& endpoint, const eprosima::fastrtps::rtps::Locator_t& locator);
}