#include <fastrtps/fastdds/dds/domain/DomainParticipant.hpp>
#include <fastrtps/fastdds/dds/domain/DomainParticipantFactory.hpp>
#include <fastrtps/fastdds/dds/domain/qos/DomainParticipantQos.hpp>
#include <fastrtps/fastrtps/rtps/participant/RTPSParticipant.h>
#include <fastrtps/rtps/attributes/RTPSParticipantAttributes.h>
#include <rclcpp/rclcpp/context.hpp>

#include <cstring>
#include <istream>

#include "discovery_domain_listener.hpp"
#include "discovery_server.h"

using namespace eprosima::fastdds::dds;
using namespace eprosima::fastrtps::rtps;

using eprosima::fastrtps::Duration_t;
using eprosima::fastrtps::rtps::GUID_t;
using namespace std::this_thread;// sleep_for, sleep_until
using namespace std::chrono;// nanoseconds, system_clock, seconds

static DomainParticipant *server = nullptr;
static void *user_data_on_discovery_participant = nullptr;
static void *user_data_on_discovery_reader = nullptr;
static void *user_data_on_discovery_writer = nullptr;

static void *user_data_on_remove_participant = nullptr;
static void *user_data_on_remove_reader = nullptr;
static void *user_data_on_remove_writer = nullptr;


static const char *const server_guid = "44.53.00.5f.45.50.52.4f.53.49.4d.41";
static const octet server_guid_octets[12] = {
    0x44, 0x53, 0x00, 0x5f, 0x45, 0x50, 0x52, 0x4f, 0x53, 0x49, 0x4d, 0x41
};

static DomainParticipantQos create_server_attributes(uint32_t server_port,
                                                     uint32_t locator_port,
                                                     const char *const guid)
{
    DomainParticipantQos server_qos = PARTICIPANT_QOS_DEFAULT;

    // Set participant as SERVER
    server_qos.wire_protocol().builtin.discovery_config.discoveryProtocol =
            DiscoveryProtocol_t::BACKUP;

    // Set SERVER's GUID prefix
    std::istringstream(guid) >> server_qos.wire_protocol().prefix;

    // Set SERVER's listening locator for PDP
    Locator_t locator;
    //IPLocator::setIPv4(locator, 127, 0, 0, 1);
    IPLocator::setIPv4(locator, 0, 0, 0, 0);
    locator.port = server_port;
    server_qos.wire_protocol().builtin.metatrafficUnicastLocatorList.push_back(
            locator);

    /* Add a remote serve to which this server will connect */
    // Set remote SERVER's GUID prefix
    RemoteServerAttributes remote_server_att;
    remote_server_att.ReadguidPrefix(guid);

    // Set remote SERVER's listening locator for PDP
    Locator_t remote_locator;
    //IPLocator::setIPv4(remote_locator, 127, 0, 0, 1);
    IPLocator::setIPv4(remote_locator, 0, 0, 0, 0);
    remote_locator.port = locator_port;
    remote_server_att.metatrafficUnicastLocatorList.push_back(remote_locator);

    // Add remote SERVER to SERVER's list of SERVERs
    server_qos.wire_protocol()
            .builtin.discovery_config.m_DiscoveryServers.push_back(remote_server_att);

    return server_qos;
}

int run_discovery_server_impl(uint32_t domain_id,
                              DiscoveryServerParams params)
{
    printf("run_discovery_server_impl\n");
    // Get default participant QoS
    auto server_qos = create_server_attributes(11811, 11812, server_guid);

    // Create SERVER
    auto *listener = new DiscoveryDomainParticipantListener();

    // Set discovery callbacks
    listener->set_participant_discovery_callback(
            params.participant_discovery_callback, user_data_on_discovery_participant);
    listener->set_reader_discovery_callback(params.reader_discovery_callback,
                                            user_data_on_discovery_reader);
    listener->set_writer_discovery_callback(params.writer_discovery_callback,
                                            user_data_on_discovery_writer);

    // Set callbacks on remove
    listener->set_participant_removed_callback(
            params.participant_removed_callback, user_data_on_remove_participant);
    listener->set_reader_removed_callback(params.reader_removed_callback,
                                          user_data_on_remove_reader);
    listener->set_writer_removed_callback(params.writer_removed_callback, user_data_on_remove_writer);

    if (server && server->is_enabled()) {
        server->close();
        // server->delete_contained_entities();
    }

    printf("Creating server participant with domain id %d\n", domain_id);
    assert(DomainParticipantFactory::get_instance() != nullptr);
    server = DomainParticipantFactory::get_instance()->create_participant(
            domain_id, server_qos, listener);
    // DomainParticipantFactory::get_instance()->delete_participant(server);
    if (nullptr == server) {
        fprintf(stderr, "Unable to create server participant\n");
        return -1;
    }
    printf("Participant created");

    return 0;
}

static void kill_server_callback(ParticipantData participant_data, void *user_data)
{
    printf("Kill server callback\n");
    auto guid = participant_data.guid;
    size_t guid_size = 12 * sizeof(octet);
    if (memcmp(guid, server_guid_octets, guid_size) == 0) {
        auto participant =
                static_cast<DomainParticipant *>(participant_data.participant);
        auto ret = DomainParticipantFactory::get_instance()->delete_participant(
                participant);
        if (ret != ReturnCode_t::RETCODE_OK) {
            fprintf(stderr, "Unable to kill the server via callback\n");
        }
    }
}

int stop_discovery_server_impl(uint32_t domain_id)
{
    // Delete server participant
    auto instance = DomainParticipantFactory::get_instance();
    if (server != nullptr) {
        ReturnCode_t err = instance->delete_participant(server);
        if (err != ReturnCode_t::RETCODE_OK) {
            fprintf(stderr, "Unable to delete server participant");
            return -1;
        }
    } else {
        printf("Server participant is null. Trying to recover state from existing "
               "process\n");

        // Create server to delete another server. TODO: I don't know better way to
        // do this.
        auto *listener = new DiscoveryDomainParticipantListener();

        listener->set_participant_discovery_callback(
                kill_server_callback, user_data_on_discovery_participant);

        printf("Creating server participant with domain id %d\n", domain_id);

        auto server_qos = create_server_attributes(
                11814, 11815, "45.53.00.5f.45.50.51.4f.53.49.4d.42");
        // auto server_qos = create_server_attributes(11813, 11814, server_guid);
        server = instance->create_participant(domain_id, server_qos, listener);

        if (nullptr == server) {
            fprintf(stderr, "Unable to create server participant\n");
            return -1;
        } else {
            printf("Recover participant was created successfully\n");
        }

        std::this_thread::sleep_for(std::chrono::seconds(20));

        // system("killall ")
    }

    return 0;
}

int is_discovery_running_impl(uint32_t domain_id)
{
    if (server != nullptr)
        return 1;

    if (run_discovery_server_impl(domain_id, DiscoveryServerParams()) == -1)
        return 1;
    // TODO: fix this function. It is temporary solution (created 15.08.2023)
    stop_discovery_server_impl(domain_id);
    // delete server;
    return 0;
}

void register_on_participant_discovery_data(void *data)
{
    user_data_on_discovery_participant = data;
}

void register_on_reader_discovery_data(void *data)
{
    user_data_on_discovery_reader = data;
}

void register_on_writer_discovery_data(void *data)
{
    user_data_on_discovery_writer = data;
}
void register_on_participant_removed_data(void *data)
{
    user_data_on_remove_participant = data;
}
void register_on_reader_removed_data(void *data)
{
    user_data_on_remove_reader = data;
}
void register_on_writer_removed_data(void *data)
{
    user_data_on_remove_writer = data;
}
