pub mod discovery_server {
    use std::ffi::OsStr;
    use std::sync::{Arc, mpsc, Mutex};
    use std::sync::atomic::AtomicPtr;
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;
    use bitflags::{bitflags, Flags};
    use dns_lookup::lookup_addr;
    use log::{debug, warn};
    use serde::de::Unexpected::Option;
    use single_value_channel::channel_starting_with;
    use tls_parser::parse_content_and_signature;
    use crate::discovery_server_impl::{ParticipantData, ReaderData, stop_discovery_server_impl, WriterData};
    use crate::fastdds_server::fastdds_server::{FastDDSDiscoverer, FastDDSEntity, FastDDSEvent, PartFunc, ReadFunc, WriteFunc};
    use crate::network::network::{hostname_ip, parse_endpoint};
    use crate::ros2_server::ros2_server::{Ros2Discoverer, Ros2DiscovererParams};
    use crate::ros2entites::ros2entities::{Ros2Context, Ros2NodeState, Ros2Package, Ros2Publisher, Ros2State, Ros2Subscriber, Ros2Topic};

    #[derive(Clone)]
    pub struct DiscoveryFlags(u32);

    bitflags! {
        impl DiscoveryFlags: u32 {
            const EnableFastdds = 0b00000001;
            const EnableROS2 = 0b00000010;
            const IncludeInternals = 0b00000100;
        }
    }

    fn default_handler_participant(_participant_data: ParticipantData) {}

    fn default_handler_reader(_reader_data: ReaderData) {}

    fn default_handler_writer(_writer_data: WriterData) {}

    pub struct FastddsParams {
        pub on_participant_discovery: PartFunc,
        pub on_reader_discovery: ReadFunc,
        pub on_writer_discovery: WriteFunc,
        pub on_participant_removed: PartFunc,
        pub on_reader_removed: ReadFunc,
        pub on_writer_removed: WriteFunc,
    }

    impl FastddsParams {
        pub fn new() -> FastddsParams {
            FastddsParams {
                on_participant_discovery: Box::new(default_handler_participant),
                on_reader_discovery: Box::new(default_handler_reader),
                on_writer_discovery: Box::new(default_handler_writer),
                on_participant_removed: Box::new(default_handler_participant),
                on_reader_removed: Box::new(default_handler_reader),
                on_writer_removed: Box::new(default_handler_writer),
            }
        }
    }

    pub struct ROS2Params {}

    impl ROS2Params {}


    pub struct DiscoveryServer {
        pub domain_id: u32,
        pub discovery_flags: DiscoveryFlags,
        pub fastdds_params: FastddsParams,
        pub on_state_update: fn(Arc<Ros2State>),

        pub fastdds_discoverer: Box<FastDDSDiscoverer>,
        pub ros2_discoverer: Box<Ros2Discoverer>,
        pub state: Arc<Mutex<Ros2State>>,

        state_tx: single_value_channel::Updater<std::option::Option<Ros2State>>,
    }

    fn create_discovery_server(domain_id: u32, discovery_flags: DiscoveryFlags, state_tx: single_value_channel::Updater<std::option::Option<Ros2State>>) -> DiscoveryServer {
        DiscoveryServer {
            domain_id,
            discovery_flags: discovery_flags.clone(),
            fastdds_params: FastddsParams::new(),
            on_state_update: |state: Arc<Ros2State>| {},
            fastdds_discoverer: Box::new(FastDDSDiscoverer::new(domain_id)),
            ros2_discoverer: Box::new(Ros2Discoverer::new(Ros2DiscovererParams { domain_id })),
            state: Arc::new(Mutex::new(Ros2State::new(discovery_flags.contains(DiscoveryFlags::IncludeInternals)))),
            state_tx,
        }
    }

    impl Default for DiscoveryServer {
        fn default() -> DiscoveryServer {
            let (rx, tx) = single_value_channel::channel();
            create_discovery_server(0, DiscoveryFlags::EnableROS2 | DiscoveryFlags::EnableFastdds, tx)
        }
    }

    impl DiscoveryServer {
        pub fn new(domain_id: u32, discovery_flags: DiscoveryFlags) -> (single_value_channel::Receiver<std::option::Option<Ros2State>>, DiscoveryServer) {
            let (state_rx, state_tx) = single_value_channel::channel::<Ros2State>();
            return (state_rx, create_discovery_server(domain_id, discovery_flags, state_tx));
        }

        /// Handle discovered publisher
        /// This function adds publisher into Ros2State field
        /// Note: some information, like node_name isn't available without ros2discoverer
        /// In the case if ros2discoverer disabled, the node_name field filled with "unknown" value.
        /// # Arguments
        ///
        /// * `publisher`:
        ///
        /// returns: ()
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        fn handle_discovered_publisher(&self, mut publisher: Ros2Publisher) {
            //if !self.discovery_flags.contains(DiscoveryFlags::IncludeInternals) && self.is_internal(publisher.clone().topic_name) {
            //    return;
            //}

            publisher.node_name = match self.node_name_by_gid(publisher.topic_name.clone(), publisher.guid.clone()) {
                Ok(node_name) => node_name,
                Err(_error_str) => "unknown".to_string()
            };
            if publisher.host.ip != "SHM" {
                publisher.host.name = hostname_ip(publisher.clone().host.ip);
            }
            self.show_pub_info(publisher.clone());

            if !publisher.node_name.is_empty() && publisher.node_name != "_NODE_NAME_UNKNOWN_" {
                self.state.lock().unwrap().add_publisher(publisher.clone());

                let res = self.state_tx.update(Some(self.state.lock().unwrap().clone()));
                match res {
                    Ok(()) => return,
                    Err(e) => warn!("Unable to send state in handle_discovered_publisher")
                }
            }
        }
        fn handle_discovered_subscriber(&self, mut subscriber: Ros2Subscriber) {
            //if !self.discovery_flags.contains(DiscoveryFlags::IncludeInternals) && self.is_internal(subscriber.clone().topic_name) {
            //    return;
            //}
            subscriber.node_name = match self.node_name_by_gid(subscriber.topic_name.clone(), subscriber.guid.clone()) {
                Ok(node_name) => node_name,
                Err(_error_str) => "unknown".to_string()
            };
            if subscriber.host.ip != "SHM" {
                subscriber.host.name = hostname_ip(subscriber.clone().host.ip);
            }
            self.show_sub_info(subscriber.clone());
            if !subscriber.node_name.is_empty() && subscriber.node_name != "_NODE_NAME_UNKNOWN_" {
                self.state.lock().unwrap().add_subscriber(subscriber.clone());
                let res = self.state_tx.update(Some(self.state.lock().unwrap().clone()));
                match res {
                    Ok(()) => return,
                    Err(e) => warn!("Unable to send state in handle_discovered_subscriber")
                }
            }
        }

        fn handle_discovered_context(&self, context: Ros2Context) {}

        fn show_sub_info(&self, subscriber: Ros2Subscriber) {
            debug!("------------------------On reader discovery-----------------------");
            debug!("Node name: {}", subscriber.node_name);
            debug!("Topic name: {}", subscriber.topic_name);
            debug!("Topic type: {}", subscriber.topic_type);
            debug!("Topic guid: {}", subscriber.guid);
            debug!("Endpoint IP: {}", subscriber.host.ip);
            debug!("Endpoint name: {}", subscriber.host.name);
            debug!("------------------------------------------------------------------");
        }

        fn show_pub_info(&self, publisher: Ros2Publisher) {
            debug!("------------------------On writer removed-----------------------");
            debug!("Node name: {}", publisher.node_name);
            debug!("Topic name: {}", publisher.topic_name);
            debug!("Topic type: {}", publisher.topic_type);
            debug!("Topic guid: {}", publisher.guid);
            debug!("Endpoint IP: {}", publisher.host.ip);
            debug!("Endpoint name: {}", publisher.host.name);
            debug!("------------------------------------------------------------------");
        }

        fn handle_removed_publisher(&self, mut publisher: Ros2Publisher) {
            publisher.node_name = match self.node_name_by_gid(publisher.topic_name.clone(), publisher.guid.clone()) {
                Ok(node_name) => node_name,
                Err(_error_str) => "unknown".to_string()
            };
            if publisher.host.ip != "SHM" {
                publisher.host.name = hostname_ip(publisher.clone().host.ip);
            }
            self.show_pub_info(publisher.clone());
            if !publisher.node_name.is_empty() && publisher.node_name != "_NODE_NAME_UNKNOWN_" {
                self.state.lock().unwrap().remove_publisher(publisher.clone());
                let res = self.state_tx.update(Some(self.state.lock().unwrap().clone()));
                match res {
                    Ok(()) => return,
                    Err(e) => warn!("Unable to send state in handle_removed_publisher")
                }
            }
        }

        fn handle_removed_subscriber(&self, mut subscriber: Ros2Subscriber) {
            subscriber.node_name = match self.node_name_by_gid(subscriber.topic_name.clone(), subscriber.guid.clone()) {
                Ok(node_name) => node_name,
                Err(_error_str) => "unknown".to_string()
            };
            if subscriber.host.ip != "SHM" {
                subscriber.host.name = hostname_ip(subscriber.clone().host.ip);
            }
            self.show_sub_info(subscriber.clone());
            if !subscriber.node_name.is_empty() && subscriber.node_name != "_NODE_NAME_UNKNOWN_" {
                self.state.lock().unwrap().remove_subscriber(subscriber.clone());
                let res = self.state_tx.update(Some(self.state.lock().unwrap().clone()));
                match res {
                    Ok(()) => return,
                    Err(e) => warn!("Unable to send state in handle_removed_subscriber")
                }
            }
        }

        fn handle_removed_context(&self, context: Ros2Context) {}

        pub fn run(&mut self)
        {
            debug!("run_discovery_server");

            // Discover packages
            if self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                // We have to explore package once at booting or by request
                let packages = self.ros2_discoverer.explore_packages();
                for package in packages {
                    self.state.lock().unwrap().add_package(package);
                }
            }

            // FastDDS callbacks
            // init fastdds discovery server
            if self.discovery_flags.contains(DiscoveryFlags::EnableFastdds) {
                self.fastdds_discoverer.run();
            }

            let rx = &self.fastdds_discoverer.rx;
            let handle_event_pub = |publisher: Ros2Publisher, event_type: FastDDSEvent| {
                match event_type {
                    FastDDSEvent::PublisherDiscovered => self.handle_discovered_publisher(publisher),
                    FastDDSEvent::PublisherRemoved => self.handle_removed_publisher(publisher),
                    _ => {}
                };
            };
            let handle_event_sub = |subscriber: Ros2Subscriber, event_type: FastDDSEvent| {
                match event_type {
                    FastDDSEvent::SubscriberDiscovered => self.handle_discovered_subscriber(subscriber),
                    FastDDSEvent::SubscriberRemoved => self.handle_removed_subscriber(subscriber),
                    _ => {}
                }
            };
            let handle_event_context = |context: Ros2Context, event_type: FastDDSEvent| {
                match event_type {
                    FastDDSEvent::ContextDiscovered => self.handle_discovered_context(context),
                    FastDDSEvent::ContextRemoved => self.handle_removed_context(context),
                    _ => {}
                }
            };

            for (event_type, data) in rx {
                match data {
                    FastDDSEntity::Publisher(publisher) => handle_event_pub(publisher, event_type),
                    FastDDSEntity::Subscriber(subscriber) => handle_event_sub(subscriber, event_type),
                    FastDDSEntity::Context(context) => handle_event_context(context, event_type)
                }
            }
        }

        pub fn stop(&self) {
            debug!("stop_discovery_server");
            unsafe {
                stop_discovery_server_impl(self.domain_id);
            }
        }

        pub fn is_running(&self) -> bool {
            debug!("is_discovery_running");
            if self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return true;
            }

            if self.discovery_flags.contains(DiscoveryFlags::EnableFastdds) && self.fastdds_discoverer.running {
                return true;
            }

            return false;
        }

        pub fn lifecycle_state(&self, node_name: String) -> Result<Ros2NodeState, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            return Ok(self.ros2_discoverer.lifecycle_state(node_name));
        }

        pub fn lifecycled_node_names(&self) -> Result<Vec<String>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            return Ok(self.ros2_discoverer.lifecycled_node_names());
        }

        pub fn is_node_lifecycle(&self, node_name: String) -> Result<bool, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }
            return Ok(self.ros2_discoverer.is_node_lifecycle(node_name));
        }

        ///  This function may be well applied only for lifecycle nodes.
        ///  If node doesn't support it, `killall` command will be used without any guarantee of success
        /// # Arguments
        ///
        /// * `node_name`: Name of node
        ///
        /// returns: String
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        pub fn shutdown_node(&self, node_name: String) -> Result<String, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }
            return Ok(self.ros2_discoverer.shutdown_node(node_name));
        }

        pub fn run_sample_node(&self) -> Result<String, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            return Ok(self.ros2_discoverer.run_sample_node());
        }

        pub fn is_node_running(&self, node_name: String) -> bool {
            return self.ros2_node_names().unwrap().contains(&node_name);
        }

        pub fn run_node<I, S>(&self, node_name: String, args: I) -> Result<String, String>
            where
                I: IntoIterator<Item=S>,
                S: AsRef<OsStr>
        {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.run_node(node_name, args))
        }

        pub fn explore_packages(&self) -> Result<Vec<Ros2Package>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.explore_packages())
        }

        pub fn ros2_topic_names(&self) -> Result<Vec<String>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.ros2_topic_names())
        }

        pub fn ros2_package_names(&self) -> Result<Vec<String>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.ros2_package_names())
        }

        pub fn explore_topics(&self) -> Result<Vec<Ros2Topic>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.explore_topics())
        }

        pub fn topic_info(&self, topic_name: String) -> Result<String, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }
            Ok(self.ros2_discoverer.topic_info(topic_name))
        }

        pub fn package_path(&self, package_name: String) -> Result<String, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.package_path(package_name))
        }

        /// Find package prefix from for specified package
        ///
        /// # Arguments
        ///
        /// * `package_name`: package name
        ///
        /// returns: String
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        pub fn package_prefix(&self, package_name: String) -> Result<String, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.package_prefix(package_name))
        }

        pub fn ros2_node_names(&self) -> Result<Vec<String>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }

            Ok(self.ros2_discoverer.ros2_node_names())
        }

        pub fn ros2_subscriber_names(&self) -> Result<Vec<String>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }
            Ok(self.ros2_discoverer.ros2_subscriber_names())
        }

        pub fn node_name_by_gid(&self, topic_name: String, gid_search: String) -> Result<String, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }
            Ok(self.ros2_discoverer.node_name_by_gid(topic_name, gid_search))
        }

        pub fn ros2_executable_names(&self, package_name: String) -> Result<Vec<String>, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }
            Ok(self.ros2_discoverer.ros2_executable_names(package_name))
        }

        pub fn node_info(&self, node_name: String) -> Result<String, String> {
            if !self.discovery_flags.contains(DiscoveryFlags::EnableROS2) {
                return Err("You must enable ros2 support and run discovery server to perform this action".to_string());
            }
            Ok(self.ros2_discoverer.node_info(node_name))
        }
    }
}