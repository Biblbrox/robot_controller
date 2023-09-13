#include "network.hpp"

namespace r2discoverer {

using eprosima::fastrtps::rtps::octet;

void parse_endpoint_fastdds(FastDDSEndpoint &endpoint, const eprosima::fastrtps::rtps::Locator_t& locator)
{
    if (locator.kind == LOCATOR_KIND_UDPv4) {
        endpoint.transport = UPDV4_TRANSPORT;
        std::memcpy(endpoint.endpoint_v4, locator.address + 12 * sizeof(octet), 4 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_UDPv6) {
        endpoint.transport = UPDV6_TRANSPORT;
        std::memcpy(endpoint.endpoint_v6, locator.address, 16 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_TCPv4) {
        endpoint.transport = TCPV4_TRANSPORT;
        std::memcpy(endpoint.endpoint_v4, locator.address + 12 * sizeof(octet), 4 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_TCPv6) {
        endpoint.transport = TCPV6_TRANSPORT;
        std::memcpy(endpoint.endpoint_v6, locator.address, 16 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_SHM) {
        endpoint.transport = SHM_TRANSPORT;
    }
    //endpoint.endpoint_v4[4] = '\0';
    //endpoint.endpoint_v6[16] = '\0';
}
}