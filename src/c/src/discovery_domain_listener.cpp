/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

#include <fastrtps/fastdds/dds/domain/DomainParticipant.hpp>
#include <fastrtps/fastdds/rtps/common/Types.h>

#include "discovery_domain_listener.hpp"
#include "network.hpp"

using eprosima::fastrtps::rtps::GuidPrefix_t;
using eprosima::fastrtps::rtps::Locator_t;
using eprosima::fastrtps::rtps::octet;

void DiscoveryDomainParticipantListener::on_participant_discovery(
        eprosima::fastdds::dds::DomainParticipant *participant,
        eprosima::fastrtps::rtps::ParticipantDiscoveryInfo &&info)
{
    static_cast<void>(participant);
    if (info.status == eprosima::fastrtps::rtps::ParticipantDiscoveryInfo::
                               DISCOVERED_PARTICIPANT) {
        ParticipantData participant_data;
        std::memcpy(participant_data.guid, info.info.m_guid.guidPrefix.value,
                    GuidPrefix_t::size * sizeof(octet));
        participant_data.endpoint.port = info.info.default_locators.unicast.begin()->port;

        Locator_t locator = *info.info.default_locators.unicast.begin();
        r2discoverer::parse_endpoint_fastdds(participant_data.endpoint, locator);
        participant_data.participant = static_cast<void *>(participant);

        if (m_on_participant_discovery) {
            assert(m_on_participant_discovery_data != nullptr);
            //std::cout << "cpp: m_on_participant_discovery\n";
            m_on_participant_discovery(participant_data,
                                       m_on_participant_discovery_data);
        }
    } else if (info.status == eprosima::fastrtps::rtps::ParticipantDiscoveryInfo::
                                      CHANGED_QOS_PARTICIPANT) {
        /* Process the case when a DomainParticipant changed its QOS */

    } else if (info.status == eprosima::fastrtps::rtps::ParticipantDiscoveryInfo::
                                      REMOVED_PARTICIPANT) {
        /* Process the case when a DomainParticipant was removed from the domain */
    }
}
void DiscoveryDomainParticipantListener::on_subscriber_discovery(
        eprosima::fastdds::dds::DomainParticipant *participant,
        eprosima::fastrtps::rtps::ReaderDiscoveryInfo &&info)
{
    static_cast<void>(participant);
    auto reader_info = [info]() {
        ReaderData reader_data;
        std::strcpy(reader_data.topic_name, info.info.topicName().c_str());
        std::strcpy(reader_data.type_name, info.info.typeName().c_str());
        auto guid_size = sizeof(octet) * GuidPrefix_t::size;
        std::memcpy(reader_data.guid_prefix, info.info.guid().guidPrefix.value,
                    guid_size);

        Locator_t locator = *info.info.remote_locators().unicast.begin();
        r2discoverer::parse_endpoint_fastdds(reader_data.endpoint, locator);

        return reader_data;
    };
    switch (info.status) {
        case eprosima::fastrtps::rtps::ReaderDiscoveryInfo::DISCOVERED_READER:
            /* Process the case when a new subscriber was found in the domain */
            if (m_on_reader_discovery) {
                assert(m_on_reader_discovery_data != nullptr);
                //std::cout << "cpp: on_subscriber_discovery\n";
                ReaderData reader_data = reader_info();
                m_on_reader_discovery(reader_data, m_on_reader_discovery_data);
            }
            break;
        case eprosima::fastrtps::rtps::ReaderDiscoveryInfo::CHANGED_QOS_READER:
            /* Process the case when a subscriber changed its QOS */
            break;
        case eprosima::fastrtps::rtps::ReaderDiscoveryInfo::REMOVED_READER:
            if (m_on_reader_remove) {
                assert(m_on_reader_remove_data != nullptr);
                ReaderData reader_data = reader_info();
                m_on_reader_remove(reader_data, m_on_reader_remove_data);
            }
            /* Process the case when a subscriber was removed from the domain */
            break;
    }
}
void DiscoveryDomainParticipantListener::on_publisher_discovery(
        eprosima::fastdds::dds::DomainParticipant *participant,
        eprosima::fastrtps::rtps::WriterDiscoveryInfo &&info)
{
    static_cast<void>(participant);
    auto writer_info = [info]() {
        WriterData writer_data;
        std::strcpy(writer_data.topic_name, info.info.topicName().c_str());
        std::strcpy(writer_data.type_name, info.info.typeName().c_str());
        auto guid_size = sizeof(octet) * GuidPrefix_t::size;
        std::memcpy(writer_data.guid_prefix, info.info.guid().guidPrefix.value,
                    guid_size);
        Locator_t locator = *info.info.remote_locators().unicast.begin();
        r2discoverer::parse_endpoint_fastdds(writer_data.endpoint, locator);
        return writer_data;
    };
    switch (info.status) {
        case eprosima::fastrtps::rtps::WriterDiscoveryInfo::DISCOVERED_WRITER: {
            /* Process the case when a new publisher was found in the domain */
            if (m_on_writer_discovery) {
                assert(m_on_writer_discovery_data != nullptr);
                //std::cout << "cpp: on_publisher_discovery\n";
                WriterData writer_data = writer_info();
                m_on_writer_discovery(writer_data, m_on_writer_discovery_data);
            }
            break;
        }
        case eprosima::fastrtps::rtps::WriterDiscoveryInfo::CHANGED_QOS_WRITER:
            /* Process the case when a publisher changed its QOS */
            break;
        case eprosima::fastrtps::rtps::WriterDiscoveryInfo::REMOVED_WRITER:
            if (m_on_writer_remove) {
                assert(m_on_writer_remove_data != nullptr);
                WriterData writer_data = writer_info();
                m_on_writer_remove(writer_data, m_on_writer_remove_data);
            }
            /* Process the case when a publisher was removed from the domain */
            break;
    }
}

void DiscoveryDomainParticipantListener::on_type_discovery(
        eprosima::fastdds::dds::DomainParticipant *participant,
        const eprosima::fastrtps::rtps::SampleIdentity &request_sample_id,
        const eprosima::fastrtps::string_255 &topic,
        const eprosima::fastrtps::types::TypeIdentifier *identifier,
        const eprosima::fastrtps::types::TypeObject *object,
        eprosima::fastrtps::types::DynamicType_ptr dyn_type)
{

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
        on_participant_discovery_callback_t callback, void *data)
{
    m_on_participant_discovery = callback;
    m_on_participant_discovery_data = data;
}

void DiscoveryDomainParticipantListener::set_reader_discovery_callback(
        on_reader_discovery_callback_t callback, void *data)
{
    m_on_reader_discovery = callback;
    m_on_reader_discovery_data = data;
}
void DiscoveryDomainParticipantListener::set_writer_discovery_callback(
        on_writer_discovery_callback_t callback, void *data)
{
    m_on_writer_discovery = callback;
    m_on_writer_discovery_data = data;
}
void DiscoveryDomainParticipantListener::set_participant_removed_callback(
        on_participant_remove_callback_t callback, void *data)
{
    m_on_participant_remove = callback;
    m_on_participant_remove_data = data;
}
void DiscoveryDomainParticipantListener::set_reader_removed_callback(
        on_reader_remove_callback_t callback, void *data)
{
    m_on_reader_remove = callback;
    m_on_reader_remove_data = data;
}
void DiscoveryDomainParticipantListener::set_writer_removed_callback(
        on_writer_remove_callback_t callback, void *data)
{
    m_on_writer_remove = callback;
    m_on_writer_remove_data = data;
}
