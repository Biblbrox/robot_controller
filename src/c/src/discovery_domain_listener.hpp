/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

#pragma once

#include <fastrtps/fastdds/dds/domain/DomainParticipantFactory.hpp>
#include <fastrtps/fastdds/dds/domain/DomainParticipantListener.hpp>
#include <fastrtps/fastdds/dds/domain/qos/DomainParticipantQos.hpp>
#include <fastrtps/fastdds/dds/publisher/DataWriter.hpp>
#include <fastrtps/fastdds/dds/publisher/DataWriterListener.hpp>
#include <fastrtps/fastdds/dds/publisher/Publisher.hpp>
#include <fastrtps/fastdds/dds/topic/TypeSupport.hpp>

#include "discovery_server.h"

class DiscoveryDomainParticipantListener
    : public eprosima::fastdds::dds::DomainParticipantListener {
    /* Custom Callback on_participant_discovery */
    void on_participant_discovery(
            eprosima::fastdds::dds::DomainParticipant *participant,
            eprosima::fastrtps::rtps::ParticipantDiscoveryInfo &&info) override;

    /* Custom Callback on_subscriber_discovery */
    void on_subscriber_discovery(
            eprosima::fastdds::dds::DomainParticipant *participant,
            eprosima::fastrtps::rtps::ReaderDiscoveryInfo &&info) override;

    /* Custom Callback on_publisher_discovery */
    void on_publisher_discovery(
            eprosima::fastdds::dds::DomainParticipant *participant,
            eprosima::fastrtps::rtps::WriterDiscoveryInfo &&info) override;

    /* Custom Callback on_type_discovery */
    void on_type_discovery(
            eprosima::fastdds::dds::DomainParticipant *participant,
            const eprosima::fastrtps::rtps::SampleIdentity &request_sample_id,
            const eprosima::fastrtps::string_255 &topic,
            const eprosima::fastrtps::types::TypeIdentifier *identifier,
            const eprosima::fastrtps::types::TypeObject *object,
            eprosima::fastrtps::types::DynamicType_ptr dyn_type) override;

public:
    void set_participant_discovery_callback(
            on_participant_discovery_callback_t callback, void *data);
    void set_reader_discovery_callback(on_reader_discovery_callback_t callback,
                                       void *data);
    void set_writer_discovery_callback(on_writer_discovery_callback_t callback,
                                       void *data);

    void
    set_participant_removed_callback(on_participant_remove_callback_t callback,
                                     void *data);
    void set_reader_removed_callback(on_reader_remove_callback_t callback,
                                     void *data);
    void set_writer_removed_callback(on_writer_remove_callback_t callback,
                                     void *data);

private:
    // On discovery data
    on_participant_discovery_callback_t m_on_participant_discovery;
    on_writer_discovery_callback_t m_on_writer_discovery;
    on_reader_discovery_callback_t m_on_reader_discovery;
    void *m_on_participant_discovery_data;
    void *m_on_writer_discovery_data;
    void *m_on_reader_discovery_data;

    // On remove data
    on_participant_remove_callback_t m_on_participant_remove;
    on_writer_remove_callback_t m_on_writer_remove;
    on_reader_remove_callback_t m_on_reader_remove;
    void *m_on_participant_remove_data;
    void *m_on_writer_remove_data;
    void *m_on_reader_remove_data;
};