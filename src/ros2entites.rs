/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod ros2entities {
    use std::cmp::min;
    use std::string::String;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Clone, Serialize)]
    pub struct Settings {
        pub domain_id: u32,
        pub include_internals: bool,
        pub dds_topic_type: bool
    }

    impl Settings {
        pub fn new() -> Settings {
            return Settings { domain_id: 0, include_internals: false, dds_topic_type: false };
        }

        pub fn from_json(json: &str) -> Settings {
            let settings: Settings = serde_json::from_str(json).unwrap();
            return settings;
        }

        pub fn to_json(&self) -> String {
            let json = serde_json::to_string(&self).unwrap();
            return json;
        }

        pub fn save(&self, path: &str) {
            let json = self.to_json();
            std::fs::write(path, json).expect("Unable to write file");
        }
    }


    #[derive(Deserialize, Clone)]
    pub struct Ros2State {
        pub packages: Vec<Ros2Package>,
        pub nodes: Vec<Ros2Node>,
        pub topics: Vec<Ros2Topic>,
        pub include_internals: bool,
    }

    impl Ros2State {
        pub fn new(filter_internal: bool) -> Ros2State {
            return Ros2State {
                packages: Vec::new(),
                nodes: Vec::new(),
                topics: Vec::new(),
                include_internals: filter_internal,
            };
        }

        pub fn add_package(&mut self, package_: Ros2Package) {
            if !self.has_package(package_.clone().name) {
                self.packages.push(package_.clone());
            }
        }

        pub fn has_package(&self, package_name: String) -> bool {
            return self.packages.iter().any(|package| package.name == package_name);
        }

        pub fn has_node(&self, node_name: String) -> bool {
            return self.nodes.iter().any(|node| node.name == node_name);
        }

        pub fn has_publisher(&self, publisher_: Ros2Publisher) -> bool {
            let pred = |publisher: &&Ros2Publisher| { return publisher.topic_type == publisher_.topic_type && publisher.node_name == publisher_.node_name; };
            return !self.nodes.iter().position(|node| !node.publishers.iter().find(pred).is_none()).is_none();
        }

        pub fn has_subscriber(&self, subscriber_: Ros2Subscriber) -> bool {
            let pred = |subscriber: &&Ros2Subscriber| { return subscriber.topic_type == subscriber_.topic_type && subscriber.node_name == subscriber_.node_name; };
            return !self.nodes.iter().position(|node| !node.subscribers.iter().find(pred).is_none()).is_none();
        }

        /// Push node into store if it doesn't exists yet.
        /// Returns reference to pushed node or existing node if it exists already
        /// # Arguments
        ///
        /// * `node_`: node to push
        ///
        /// returns: &Ros2Node
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        pub fn add_node(&mut self, node_: Ros2Node) -> (bool, &Ros2Node) {
            let is_present = self.nodes.iter().any(|node| node.name == node_.name);
            let pushed_node: &Ros2Node;
            if !is_present {
                self.nodes.push(node_.clone());
            }
            pushed_node = self.nodes.iter().find(|&node_found| node_found.name == node_.clone().name).unwrap();

            return (!is_present, pushed_node);
        }

        pub fn add_topic(&mut self, topic_: Ros2Topic) {
            //if self.topics.iter().find(|&topic| topic.name == topic_.name) == None {
            self.topics.push(topic_);
            //}
        }

        pub fn remove_package(&mut self, package_: Ros2Package) {
            let index = self.packages.iter().position(|package| package.name == package_.name).unwrap();
            self.packages.remove(index);
        }

        pub fn remove_node(&mut self, node_: Ros2Node) {
            let index = self.nodes.iter().position(|node| node.name == node_.name).unwrap();
            let node = self.nodes.get(index).unwrap();
            // Remove topics with beholds this node
            self.topics.retain(|topic| topic.node_name != node.name);
            // Remove node itself
            self.nodes.remove(index);
        }

        pub fn remove_topic(&mut self, topic_: Ros2Topic) {}

        fn is_internal(&self, mut topic_name: String) -> bool {
            let names: Vec<&str> = vec!["parameter_events", "rosout"];
            if topic_name.starts_with("/") {
                topic_name = topic_name[1..].to_string();
            }
            return names.contains(&topic_name.as_str());
        }

        pub fn add_publisher(&mut self, publisher: Ros2Publisher) {
            let node_name = publisher.node_name.clone();
            let is_internal = self.is_internal(publisher.topic_name.clone());
            let publishers = if (is_internal && self.include_internals) || !is_internal {
                vec![publisher.clone()]
            } else {
                vec![]
            };
            if !node_name.starts_with("_") || self.include_internals {
                let mut node = self.nodes.iter_mut().find(|node| node.name == node_name);
                if node.is_none() {
                    let new_node = Ros2Node {
                        name: node_name.clone(),
                        package_name: "".to_string(),
                        subscribers: vec![],
                        publishers,
                        service_servers: vec![],
                        service_clients: vec![],
                        action_servers: vec![],
                        action_clients: vec![],
                        host: publisher.clone().host,
                        is_lifecycle: false,
                        state: Ros2NodeState::Unconfigured,
                    };
                    self.add_node(new_node);
                } else {
                    if !publishers.is_empty() {
                        node.as_mut().unwrap().publishers.push(publisher.clone());
                    }
                }
            }

            // Check for corresponding topic presence. If it does exists, just increase publishers count
            let topic: Option<&mut Ros2Topic> = self.topics.iter_mut().find(|topic| publisher.clone().topic_name == topic.name && publisher.clone().topic_type == topic.topic_type);
            if topic.is_none() {
                self.topics.push(Ros2Topic { name: publisher.clone().topic_name.clone(), node_name, topic_type: publisher.topic_type, subscribers_num: 0, publishers_num: 1 });
            } else {
                topic.unwrap().publishers_num += 1;
            }
        }

        pub fn add_subscriber(&mut self, subscriber: Ros2Subscriber) {
            let node_name = subscriber.node_name.clone();
            let is_internal = self.is_internal(subscriber.topic_name.clone());
            let subscribers: Vec<Ros2Subscriber> = if (is_internal && self.include_internals) || !is_internal {
                vec![subscriber.clone()]
            } else {
                vec![]
            };

            if !node_name.starts_with("_") || self.include_internals {
                let mut node = self.nodes.iter_mut().find(|node| node.name == node_name);
                if node.is_none() {
                    let new_node = Ros2Node {
                        name: node_name.clone(),
                        package_name: "".to_string(),
                        subscribers,
                        publishers: vec![],
                        service_servers: vec![],
                        service_clients: vec![],
                        action_servers: vec![],
                        action_clients: vec![],
                        host: subscriber.host,
                        is_lifecycle: false,
                        state: Ros2NodeState::Unconfigured,
                    };
                    self.add_node(new_node);
                } else {
                    node.as_mut().unwrap().subscribers.push(subscriber.clone());
                }
            }
            // Check for corresponding topic presence. If it does exists, just increase subscribers count
            let topic: Option<&mut Ros2Topic> = self.topics.iter_mut().find(|topic| subscriber.topic_name == topic.name && subscriber.topic_type == topic.topic_type);
            if topic.is_none() {
                self.topics.push(Ros2Topic { name: subscriber.topic_name.clone(), node_name, topic_type: subscriber.topic_type, subscribers_num: 0, publishers_num: 1 });
            } else {
                topic.unwrap().publishers_num += 1;
            }
        }

        /// Deletes publisher from state
        /// To delete publisher we have to find the node, which contains this publisher and remove the publisher from this node
        /// # Arguments
        ///
        /// * `publisher_`:
        ///
        /// returns: ()
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        pub fn remove_publisher(&mut self, publisher_: Ros2Publisher) {
            let pred = |publisher: &&Ros2Publisher| { return publisher.topic_type == publisher_.topic_type && publisher.node_name == publisher_.node_name; };
            let node_idx = match self.nodes.iter().position(|node| !node.publishers.iter().find(pred).is_none()) {
                Some(idx) => idx,
                None => return
            };
            let node: &mut Ros2Node = self.nodes.get_mut(node_idx).unwrap();

            let pub_pred = |publisher: &Ros2Publisher| { return publisher.topic_type == publisher_.topic_type && publisher.node_name == publisher_.node_name; };
            let pub_idx = node.publishers.iter().position(pub_pred).unwrap();
            node.publishers.remove(pub_idx);

            // Check for corresponding topic presence. If it does exists, decrease publishers count.
            // As it goes to zero, delete topic
            let topic_pos = self.topics.iter_mut().position(|topic| publisher_.topic_name == topic.name && publisher_.topic_type == topic.topic_type);
            if !topic_pos.is_none() {
                let topic: &mut Ros2Topic = self.topics.get_mut(topic_pos.unwrap()).unwrap();
                if topic.publishers_num == 0 && topic.subscribers_num == 0 {
                    self.topics.remove(topic_pos.unwrap());
                } else {
                    topic.publishers_num = min(topic.publishers_num, 0);
                }
            }
        }

        /// The same as remove_publisher but for subscriber
        ///
        /// # Arguments
        ///
        /// * `subscriber_`:
        ///
        /// returns: ()
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        pub fn remove_subscriber(&mut self, subscriber_: Ros2Subscriber) {
            let pred = |&subscriber: &&Ros2Subscriber| { return subscriber.topic_type == subscriber_.topic_type && subscriber.node_name == subscriber_.node_name; };
            let node_idx = match self.nodes.iter().position(|node| !node.subscribers.iter().find(pred).is_none()) {
                Some(idx) => idx,
                None => return /* TODO: Temporary solution. This case isn't appropriate because there is a change of broken graph state*/
            };
            let node = self.nodes.get_mut(node_idx).unwrap();

            let sub_pred = |subscriber: &Ros2Subscriber| { return subscriber.topic_type == subscriber_.topic_type && subscriber.node_name == subscriber_.node_name; };
            let sub_idx = node.subscribers.iter().position(sub_pred).unwrap();
            node.subscribers.remove(sub_idx);

            // Check for corresponding topic presence. If it does exists, decrease publishers count.
            // As it goes to zero, delete topic
            let topic_pos = self.topics.iter_mut().position(|topic| subscriber_.topic_name == topic.name && subscriber_.topic_type == topic.topic_type);
            if !topic_pos.is_none() {
                let topic: &mut Ros2Topic = self.topics.get_mut(topic_pos.unwrap()).unwrap();
                if topic.publishers_num == 0 && topic.subscribers_num == 0 {
                    self.topics.remove(topic_pos.unwrap());
                } else {
                    topic.subscribers_num = min(topic.subscribers_num, 0);
                }
            }
        }

        pub fn contains_node(&self, node_name: String) -> bool {
            let node = self.nodes.iter().find(|&node| node.name == node_name);
            return !node.is_none();
        }
    }


    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2Package {
        pub name: String,
        pub path: String,
        pub executables: Vec<Ros2Executable>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2Executable {
        pub name: String,
        pub package_name: String,
        pub path: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Host {
        pub ip: String,
        pub name: String,
    }

    impl Host {
        pub fn default() -> Host {
            return Host { ip: "127.0.0.1".to_string(), name: "localhost".to_string() };
        }
        pub fn new(ip: String, name: String) -> Host {
            return Host { ip, name };
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub enum Ros2NodeState {
        Unconfigured,
        Inactive,
        Active,
        Shutdown,
        // For non lifecycle nodes, which doesn't support lifecycle states
        NonLifecycle,
    }

    /// Ros2Node is the main struct that contains almost all information about the node and its publishers and subscribers
    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2Node {
        pub name: String,
        pub package_name: String,
        pub subscribers: Vec<Ros2Subscriber>,
        pub publishers: Vec<Ros2Publisher>,
        pub service_servers: Vec<Ros2ServiceServer>,
        pub service_clients: Vec<Ros2ServiceClient>,
        pub action_servers: Vec<Ros2ActionServer>,
        pub action_clients: Vec<Ros2ActionClient>,
        pub host: Host,
        pub is_lifecycle: bool,
        pub state: Ros2NodeState,
    }

    impl Ros2Node {
        pub fn create(name: String) -> Ros2Node {
            return Ros2Node {
                name,
                package_name: "".to_string(),
                subscribers: vec![],
                publishers: vec![],
                service_servers: vec![],
                service_clients: vec![],
                action_servers: vec![],
                action_clients: vec![],
                host: Host::default(),
                is_lifecycle: false,
                state: Ros2NodeState::Inactive,
            };
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct Ros2Context {
        pub guid: String,
        pub host: Host,
    }

    impl Ros2Context {
        pub fn new(guid: String, host: Host) -> Ros2Context {
            return Ros2Context { guid, host };
        }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2Subscriber {
        pub topic_name: String,
        pub node_name: String,
        pub guid: String,
        pub topic_type: String,
        pub host: Host,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2Publisher {
        pub topic_name: String,
        pub guid: String,
        pub node_name: String,
        pub topic_type: String,
        pub host: Host,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2ActionClient {
        pub name: String,
        pub node_name: String,
        pub topic_name: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2ActionServer {
        pub name: String,
        pub node_name: String,
        pub topic_name: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2ServiceServer {
        pub name: String,
        pub node_name: String,
        pub topic_name: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2ServiceClient {
        pub name: String,
        pub node_name: String,
        pub topic_name: String,
    }

    /// I will have done with these entities later...
    /// ....
    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2Client {
        pub name: String,
        pub node_name: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Ros2Topic {
        pub name: String,
        pub node_name: String,
        pub topic_type: String,
        pub subscribers_num: u64,
        pub publishers_num: u64,
    }
}

#[cfg(test)]
mod tests {
    use crate::ros2entites::ros2entities::{Host, Ros2Node, Ros2Publisher, Ros2State, Ros2Subscriber};

    #[test]
    fn add_node() {
        let mut state = Ros2State::new();
        assert_eq!(state.has_node("test_node".to_string()), false);
        let test_node = Ros2Node::create("test_node".to_string());
        state.add_node(test_node);
        assert_eq!(state.has_node("test_node".to_string()), true);
    }

    #[test]
    fn add_publisher() {
        let mut state = Ros2State::new();
        let test_publisher: Ros2Publisher = Ros2Publisher {
            topic_name: "test_name".to_string(),
            guid: "test_guid".to_string(),
            node_name: "test_node".to_string(),
            topic_type: "test_type".to_string(),
            host: Host::default(),
        };
        assert_eq!(state.has_publisher(test_publisher.clone()), false);
        state.add_publisher(test_publisher.clone());
        assert_eq!(state.has_publisher(test_publisher.clone()), true);
    }

    #[test]
    fn add_subscriber() {
        let mut state = Ros2State::new();
        let test_subscriber: Ros2Subscriber = Ros2Subscriber {
            topic_name: "test_name".to_string(),
            guid: "test_guid".to_string(),
            node_name: "test_node".to_string(),
            topic_type: "test_type".to_string(),
            host: Host::default(),
        };
        assert_eq!(state.has_subscriber(test_subscriber.clone()), false);
        state.add_subscriber(test_subscriber.clone());
        assert_eq!(state.has_subscriber(test_subscriber.clone()), true);
    }

    #[test]
    fn remove_publisher() {
        let mut state = Ros2State::new();
        let test_publisher: Ros2Publisher = Ros2Publisher {
            topic_name: "test_name".to_string(),
            guid: "test_guid".to_string(),
            node_name: "test_node".to_string(),
            topic_type: "test_type".to_string(),
            host: Host::default(),
        };
        assert_eq!(state.has_publisher(test_publisher.clone()), false);
        state.add_publisher(test_publisher.clone());
        assert_eq!(state.has_publisher(test_publisher.clone()), true);
        state.remove_publisher(test_publisher.clone());
        assert_eq!(state.has_publisher(test_publisher.clone()), false);
    }

    #[test]
    fn remove_subscriber() {
        let mut state = Ros2State::new();
        let test_subscriber: Ros2Subscriber = Ros2Subscriber {
            topic_name: "test_name".to_string(),
            guid: "test_guid".to_string(),
            node_name: "test_node".to_string(),
            topic_type: "test_type".to_string(),
            host: Host::default(),
        };
        assert_eq!(state.has_subscriber(test_subscriber.clone()), false);
        state.add_subscriber(test_subscriber.clone());
        assert_eq!(state.has_subscriber(test_subscriber.clone()), true);
        state.remove_subscriber(test_subscriber.clone());
        assert_eq!(state.has_subscriber(test_subscriber.clone()), false);
    }
}