/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod fastdds_server {
    use ::std::os::raw::c_void;
    use std::mem::transmute;
    use flume::SendError;
    use std::thread;
    use log::debug;
    use crate::discovery_server_impl::{DiscoveryServerParams, ParticipantData, ReaderData, register_on_participant_discovery_data, register_on_participant_removed_data, register_on_reader_discovery_data, register_on_reader_removed_data, register_on_writer_discovery_data, register_on_writer_removed_data, run_discovery_server_impl, WriterData};
    use crate::fastdds_server::fastdds_server::FastDDSEntity::Context;
    use crate::network::network::{fastdds_to_ros2, hex_str_from_uc, parse_endpoint, string_from_c};
    use crate::ros2entites::ros2entities::{Host, Ros2Context, Ros2Publisher, Ros2Subscriber};

    pub type PartFunc = Box<dyn Fn(ParticipantData)>;
    pub type ReadFunc = Box<dyn Fn(ReaderData)>;
    pub type WriteFunc = Box<dyn Fn(WriterData)>;

    pub enum FastDDSEntity {
        Publisher(Ros2Publisher),
        Subscriber(Ros2Subscriber),
        Context(Ros2Context),
    }

    pub enum FastDDSEvent {
        PublisherDiscovered,
        SubscriberDiscovered,
        ContextDiscovered,
        PublisherRemoved,
        SubscriberRemoved,
        ContextRemoved,
    }

    pub struct FastDDSDiscoverer {
        pub domain_id: u32,
        pub on_participant_discovery: PartFunc,
        pub on_reader_discovery: ReadFunc,
        pub on_writer_discovery: WriteFunc,
        pub on_participant_removed: PartFunc,
        pub on_reader_removed: ReadFunc,
        pub on_writer_removed: WriteFunc,
        pub running: bool,
        pub rx: flume::Receiver<(FastDDSEvent, FastDDSEntity)>,
        //pub tx: mpsc::Sender<FastDDSEntity>,
    }

    impl FastDDSDiscoverer {
        pub fn new(domain_id: u32) -> FastDDSDiscoverer {
            let (tx, rx) = flume::unbounded::<(FastDDSEvent, FastDDSEntity)>();

            let tx_participant_discovery = tx.clone();
            let on_participant_discovery = Box::new(move |participant_data: ParticipantData| {
                // Called when new ROS2 context appears in network
                let endpoint = parse_endpoint(participant_data.endpoint);
                let guid = hex_str_from_uc(participant_data.guid);

                let context: Ros2Context = Ros2Context::new(guid, Host::new(endpoint, "unknown".to_string()));
                let entity: FastDDSEntity = Context(context);

                match tx_participant_discovery.send((FastDDSEvent::ContextDiscovered, entity)) {
                    Ok(()) => {}
                    Err(SendError(e)) => panic!("Unable to send data about discovered context")
                };
            });

            let tx_reader_discovery = tx.clone();
            let on_reader_discovery = Box::new(move |reader_data: ReaderData| {
                let mut topic_name = fastdds_to_ros2(string_from_c(reader_data.topic_name));
                let topic_type = string_from_c(reader_data.type_name);
                if !topic_name.starts_with("/") {
                    topic_name = "/".to_string() + topic_name.as_str();
                }
                let guid = hex_str_from_uc(reader_data.guid_prefix);
                let endpoint = parse_endpoint(reader_data.endpoint);

                let subscriber = Ros2Subscriber {
                    topic_name,
                    guid,
                    node_name: "unknown".to_string(),
                    topic_type,
                    host: Host::new(endpoint, "unknown".to_string()),
                };
                let entity: FastDDSEntity = FastDDSEntity::Subscriber(subscriber);
                match tx_reader_discovery.send((FastDDSEvent::SubscriberDiscovered, entity)) {
                    Ok(()) => {}
                    Err(e) => panic!("Unable to send data about discovered reader")
                };
            });

            let tx_writer_discovery = tx.clone();
            let on_writer_discovery = Box::new(move |writer_data: WriterData| {
                let mut topic_name = fastdds_to_ros2(string_from_c(writer_data.topic_name));
                let topic_type = string_from_c(writer_data.type_name);
                if !topic_name.starts_with("/") {
                    topic_name = "/".to_string() + topic_name.as_str();
                }
                let guid = hex_str_from_uc(writer_data.guid_prefix);
                let endpoint = parse_endpoint(writer_data.endpoint);

                let publisher = Ros2Publisher {
                    topic_name: topic_name.clone(),
                    guid,
                    node_name: "unknown".to_string(),
                    topic_type,
                    host: Host::new(endpoint, "unknown".to_string()),
                };

                let entity: FastDDSEntity = FastDDSEntity::Publisher(publisher);
                match tx_writer_discovery.send((FastDDSEvent::PublisherDiscovered, entity)) {
                    Ok(()) => {}
                    Err(e) => panic!("Unable to send data about discovered writer. Error {e}")
                };
            });


            let tx_participant_removed = tx.clone();
            let on_participant_removed = Box::new(move |participant_data: ParticipantData| {
                let endpoint = parse_endpoint(participant_data.endpoint);
                let guid = hex_str_from_uc(participant_data.guid);
                let context: Ros2Context = Ros2Context::new(guid, Host::new(endpoint, "unknown".to_string()));
                let entity: FastDDSEntity = Context(context);
                match tx_participant_removed.send((FastDDSEvent::ContextRemoved, entity)) {
                    Ok(()) => {}
                    Err(e) => panic!("Unable to send data about removed participant")
                };
            });
            let tx_reader_removed = tx.clone();
            let on_reader_removed = Box::new(move |reader_data: ReaderData| {
                let mut topic_name = fastdds_to_ros2(string_from_c(reader_data.topic_name));
                let topic_type = string_from_c(reader_data.type_name);
                if !topic_name.starts_with("/") {
                    topic_name = "/".to_string() + topic_name.as_str();
                }
                let guid = hex_str_from_uc(reader_data.guid_prefix);
                let endpoint = parse_endpoint(reader_data.endpoint);

                let subscriber = Ros2Subscriber {
                    topic_name,
                    guid,
                    node_name: "unknown".to_string(),
                    topic_type,
                    host: Host::new(endpoint, "unknown".to_string()),
                };
                let entity: FastDDSEntity = FastDDSEntity::Subscriber(subscriber);
                match tx_reader_removed.send((FastDDSEvent::SubscriberRemoved, entity)) {
                    Ok(()) => {}
                    Err(e) => panic!("Unable to send data about removed reader")
                };
            });
            let tx_writer_removed = tx.clone();
            let on_writer_removed = Box::new(move |writer_data: WriterData| {
                let mut topic_name = fastdds_to_ros2(string_from_c(writer_data.topic_name));
                let topic_type = string_from_c(writer_data.type_name);
                if !topic_name.starts_with("/") {
                    topic_name = "/".to_string() + topic_name.as_str();
                }
                let guid = hex_str_from_uc(writer_data.guid_prefix);
                let endpoint = parse_endpoint(writer_data.endpoint);

                let publisher = Ros2Publisher {
                    topic_name,
                    guid,
                    node_name: "unknown".to_string(),
                    topic_type,
                    host: Host::new(endpoint, "unknown".to_string()),
                };
                let entity: FastDDSEntity = FastDDSEntity::Publisher(publisher);
                match tx_writer_removed.send((FastDDSEvent::PublisherRemoved, entity)) {
                    Ok(()) => {}
                    Err(e) => panic!("Unable to send data about removed writer")
                };
            });

            FastDDSDiscoverer {
                domain_id,
                on_participant_discovery,
                on_reader_discovery,
                on_writer_discovery,
                on_participant_removed,
                on_reader_removed,
                on_writer_removed,
                running: false,
                rx,
            }
        }

        pub fn run(&mut self)
        {
            unsafe extern "C" fn wrapper_participant_discovery<F: Fn(ParticipantData)>(participant_data: ParticipantData, ctx: *mut c_void) {
                let callback_raw: *mut Box<dyn Fn(ParticipantData)> = transmute(ctx);
                (*(*callback_raw))(participant_data);
            }
            unsafe extern "C" fn wrapper_reader_discovery<F: Fn(ReaderData)>(reader_data: ReaderData, ctx: *mut c_void) {
                let callback_raw: *mut Box<dyn Fn(ReaderData)> = transmute(ctx);
                (*(*callback_raw))(reader_data);
            }
            unsafe extern "C" fn wrapper_writer_discovery<F: Fn(WriterData)>(writer_data: WriterData, ctx: *mut c_void) {
                let callback_raw: *mut Box<dyn Fn(WriterData)> = transmute(ctx);
                (*(*callback_raw))(writer_data);
            }
            unsafe extern "C" fn wrapper_participant_removed<F: Fn(ParticipantData)>(participant_data: ParticipantData, ctx: *mut c_void) {
                let callback_raw: *mut Box<dyn Fn(ParticipantData)> = transmute(ctx);
                (*(*callback_raw))(participant_data);
            }
            unsafe extern "C" fn wrapper_reader_removed<F: Fn(ReaderData)>(reader_data: ReaderData, ctx: *mut c_void) {
                let callback_raw: *mut Box<dyn Fn(ReaderData)> = transmute(ctx);
                (*(*callback_raw))(reader_data);
            }
            unsafe extern "C" fn wrapper_writer_removed<F: Fn(WriterData)>(writer_data: WriterData, ctx: *mut c_void) {
                let callback_raw: *mut Box<dyn Fn(WriterData)> = transmute(ctx);
                (*(*callback_raw))(writer_data);
            }

            let ptr_participant_dsc = Box::into_raw(Box::new(self.on_participant_discovery.as_mut()));
            let ptr_writer_dsc = Box::into_raw(Box::new(self.on_writer_discovery.as_mut()));
            let ptr_reader_dsc = Box::into_raw(Box::new(self.on_reader_discovery.as_mut()));

            let ptr_participant_rem = Box::into_raw(Box::new(self.on_participant_removed.as_mut()));
            let ptr_writer_rem = Box::into_raw(Box::new(self.on_writer_removed.as_mut()));
            let ptr_reader_rem = Box::into_raw(Box::new(self.on_reader_removed.as_mut()));

            unsafe {
                register_on_participant_discovery_data(ptr_participant_dsc as *mut c_void);
                register_on_writer_discovery_data(ptr_writer_dsc as *mut c_void);
                register_on_reader_discovery_data(ptr_reader_dsc as *mut c_void);

                register_on_participant_removed_data(ptr_participant_rem as *mut c_void);
                register_on_writer_removed_data(ptr_writer_rem as *mut c_void);
                register_on_reader_removed_data(ptr_reader_rem as *mut c_void);

                type ParticipantFunc = fn(ParticipantData);
                type ReaderFunc = fn(ReaderData);
                type WriterFunc = fn(WriterData);

                let discovery_params = DiscoveryServerParams {
                    participant_discovery_callback: Some(wrapper_participant_discovery::<ParticipantFunc>),
                    reader_discovery_callback: Some(wrapper_reader_discovery::<ReaderFunc>),
                    writer_discovery_callback: Some(wrapper_writer_discovery::<WriterFunc>),
                    participant_removed_callback: Some(wrapper_participant_removed::<ParticipantFunc>),
                    reader_removed_callback: Some(wrapper_reader_removed::<ReaderFunc>),
                    writer_removed_callback: Some(wrapper_writer_removed::<WriterFunc>),
                };

                let domain_id = self.domain_id;
                thread::spawn(move || {
                    run_discovery_server_impl(domain_id, discovery_params);
                });

                self.running = true;
            }
        }
    }
}