#pragma once

/**
 * This file contains function for extracting information from the ros2 node
 * graph.
 */

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

enum rmw_transport {
  SHM_TRANSPORT = 1,
  UPDV4_TRANSPORT,
  UPDV6_TRANSPORT,
  TCPV4_TRANSPORT,
  TCPV6_TRANSPORT
};

typedef struct {
  enum rmw_transport transport;
  void *participant;
  uint32_t port;
  unsigned char guid[12];
  union {
    unsigned char endpoint_v4[16]; // Empty in case of SHM transport
    unsigned char endpoint_v6[39]; // Empty in case of SHM transport
  };
} ParticipantData;

typedef struct {
  char topic_name[255];
  char type_name[255];
} WriterData;

typedef struct {
  char topic_name[255];
  char type_name[255];
} ReaderData;

typedef void (*on_participant_discovery_callback_t)(
    ParticipantData participant_data, void *user_data);
typedef void (*on_reader_discovery_callback_t)(ReaderData reader_data,
                                               void *user_data);
typedef void (*on_writer_discovery_callback_t)(WriterData writer_data,
                                               void *user_data);

/*void on_participant_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    eprosima::fastrtps::rtps::ParticipantDiscoveryInfo &&info) override;

void on_subscriber_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    eprosima::fastrtps::rtps::ReaderDiscoveryInfo &&info) override;

void on_publisher_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    eprosima::fastrtps::rtps::WriterDiscoveryInfo &&info) override;

void on_type_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    const eprosima::fastrtps::rtps::SampleIdentity &request_sample_id,
    const eprosima::fastrtps::string_255 &topic,
    const eprosima::fastrtps::types::TypeIdentifier *identifier,
    const eprosima::fastrtps::types::TypeObject *object,
    eprosima::fastrtps::types::DynamicType_ptr dyn_type) override;*/

/**
 * Run discovery server in separate thread.
 * Note: this is non-blocking operation.
 * @param domain_id
 * @param callback
 * @return
 */
int run_discovery_server_impl(
    uint32_t domain_id,
    on_participant_discovery_callback_t participant_callback,
    on_reader_discovery_callback_t reader_callback,
    on_writer_discovery_callback_t writer_callback);

int stop_discovery_server_impl(uint32_t domain_id);
int is_discovery_running_impl(uint32_t domain_id);
void register_on_participant_data(void *data);
void register_on_reader_data(void *data);
void register_on_writer_data(void *data);

#ifdef __cplusplus
}
#endif