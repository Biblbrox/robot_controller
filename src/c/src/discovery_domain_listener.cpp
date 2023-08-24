#include <fastrtps/fastdds/dds/domain/DomainParticipant.hpp>
#include <fastrtps/fastdds/rtps/common/Types.h>

#include <rclcpp/executors.hpp>
#include <rclcpp/rclcpp/node.hpp>

#include "discovery_domain_listener.hpp"

using eprosima::fastrtps::rtps::Locator_t;
using eprosima::fastrtps::rtps::octet;

void DiscoveryDomainParticipantListener::on_participant_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    eprosima::fastrtps::rtps::ParticipantDiscoveryInfo &&info) {
  static_cast<void>(participant);
  if (info.status == eprosima::fastrtps::rtps::ParticipantDiscoveryInfo::
                         DISCOVERED_PARTICIPANT) {

    /* Process the case when a new DomainParticipant was found in the domain */
    /*std::cout << "New DomainParticipant '" << info.info.m_participantName
              << "' with ID '" << info.info.m_guid.entityId
              << "' and GuidPrefix '" << info.info.m_guid.guidPrefix
              << "' with locators '" << info.info.default_locators
              << "' with user data '" << info.info.m_userData[0]
              << "' discovered." << std::endl;

    std::cout << "Name: " << participant->guid() << "\n";*/

    ParticipantData participant_data;
    std::memcpy(participant_data.guid, info.info.m_guid.guidPrefix.value,
                12 * sizeof(octet));
    participant_data.port = info.info.default_locators.unicast.begin()->port;

    Locator_t locator = *info.info.default_locators.unicast.begin();
    if (locator.kind == LOCATOR_KIND_UDPv4) {
      participant_data.transport = UPDV4_TRANSPORT;
      std::memcpy(participant_data.endpoint_v4,
                  locator.address + 12 * sizeof(octet), 4 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_UDPv6) {
      participant_data.transport = UPDV6_TRANSPORT;
      std::memcpy(participant_data.endpoint_v4, locator.address,
                  16 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_TCPv4) {
      participant_data.transport = TCPV4_TRANSPORT;
      std::memcpy(participant_data.endpoint_v4,
                  locator.address + 12 * sizeof(octet), 4 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_TCPv6) {
      participant_data.transport = TCPV6_TRANSPORT;
      std::memcpy(participant_data.endpoint_v4, locator.address,
                  16 * sizeof(octet));
    } else if (locator.kind == LOCATOR_KIND_SHM) {
      participant_data.transport = SHM_TRANSPORT;
    }

    participant_data.participant = static_cast<void *>(participant);

    if (m_on_participant_discovery) {
      assert(m_on_participant_discovery_data != nullptr);
      m_on_participant_discovery(participant_data,
                                 m_on_participant_discovery_data);
    }
    // m_on_participant_discovery()
  } else if (info.status == eprosima::fastrtps::rtps::ParticipantDiscoveryInfo::
                                CHANGED_QOS_PARTICIPANT) {
    /* Process the case when a DomainParticipant changed its QOS */

  } else if (info.status == eprosima::fastrtps::rtps::ParticipantDiscoveryInfo::
                                REMOVED_PARTICIPANT) {
    /* Process the case when a DomainParticipant was removed from the domain */
    /*std::cout << "New DomainParticipant '" << info.info.m_participantName
              << "' with ID '" << info.info.m_guid.entityId
              << "' and GuidPrefix '" << info.info.m_guid.guidPrefix
              << "' left the domain." << std::endl;*/
  }
}
void DiscoveryDomainParticipantListener::on_subscriber_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    eprosima::fastrtps::rtps::ReaderDiscoveryInfo &&info) {
  static_cast<void>(participant);
  switch (info.status) {
  case eprosima::fastrtps::rtps::ReaderDiscoveryInfo::DISCOVERED_READER:
    /* Process the case when a new subscriber was found in the domain */
    /*std::cout << "New DataReader subscribed to topic '" <<
       info.info.topicName()
              << "' of type '" << info.info.typeName() << "' discovered\n";*/
    if (m_on_reader_discovery) {
      ReaderData reader_data;
      std::strcpy(reader_data.topic_name, info.info.topicName().c_str());
      std::strcpy(reader_data.type_name, info.info.typeName().c_str());
      m_on_reader_discovery(reader_data, m_on_reader_discovery_data);
    }
    break;
  case eprosima::fastrtps::rtps::ReaderDiscoveryInfo::CHANGED_QOS_READER:
    /* Process the case when a subscriber changed its QOS */
    break;
  case eprosima::fastrtps::rtps::ReaderDiscoveryInfo::REMOVED_READER:
    /* Process the case when a subscriber was removed from the domain */
    /*std::cout << "New DataReader subscribed to topic '" <<
       info.info.topicName()
              << "' of type '" << info.info.typeName() << "' left the
       domain.";*/
    break;
  }
}
void DiscoveryDomainParticipantListener::on_publisher_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    eprosima::fastrtps::rtps::WriterDiscoveryInfo &&info) {
  static_cast<void>(participant);
  switch (info.status) {
  case eprosima::fastrtps::rtps::WriterDiscoveryInfo::DISCOVERED_WRITER: {
    /* Process the case when a new publisher was found in the domain */
    /*std::cout << "New DataWriter publishing under topic '"
              << info.info.topicName() << "' of type '" << info.info.typeName()
              << "' discovered\n";*/
    if (m_on_writer_discovery) {
      WriterData writer_data;
      std::strcpy(writer_data.topic_name, info.info.topicName().c_str());
      std::strcpy(writer_data.type_name, info.info.typeName().c_str());
      m_on_writer_discovery(writer_data, m_on_writer_discovery_data);
    }
    break;
  }
  case eprosima::fastrtps::rtps::WriterDiscoveryInfo::CHANGED_QOS_WRITER:
    /* Process the case when a publisher changed its QOS */
    break;
  case eprosima::fastrtps::rtps::WriterDiscoveryInfo::REMOVED_WRITER:
    /* Process the case when a publisher was removed from the domain */
    /*std::cout << "New DataWriter publishing under topic '"
              << info.info.topicName() << "' of type '" << info.info.typeName()
              << "' left the domain.";*/
    break;
  }
}

void DiscoveryDomainParticipantListener::on_type_discovery(
    eprosima::fastdds::dds::DomainParticipant *participant,
    const eprosima::fastrtps::rtps::SampleIdentity &request_sample_id,
    const eprosima::fastrtps::string_255 &topic,
    const eprosima::fastrtps::types::TypeIdentifier *identifier,
    const eprosima::fastrtps::types::TypeObject *object,
    eprosima::fastrtps::types::DynamicType_ptr dyn_type) {

  static_cast<void>(participant);
  static_cast<void>(request_sample_id);
  static_cast<void>(topic);
  static_cast<void>(identifier);
  static_cast<void>(object);
  static_cast<void>(dyn_type);
  std::cout << "New data type of topic '" << topic << "' discovered."
            << std::endl;
}

void DiscoveryDomainParticipantListener::set_participant_discovery_callback(
    on_participant_discovery_callback_t callback, void *data) {
  m_on_participant_discovery = callback;
  m_on_participant_discovery_data = data;
}

void DiscoveryDomainParticipantListener::set_participant_discovery_data(
    void *data) {
  m_on_participant_discovery_data = data;
}
void DiscoveryDomainParticipantListener::set_reader_discovery_callback(
    on_reader_discovery_callback_t callback, void *data) {
  m_on_reader_discovery = callback;
  m_on_reader_discovery_data = data;
}
void DiscoveryDomainParticipantListener::set_writer_discovery_callback(
    on_writer_discovery_callback_t callback, void *data) {
  m_on_writer_discovery = callback;
  m_on_writer_discovery_data = data;
}
