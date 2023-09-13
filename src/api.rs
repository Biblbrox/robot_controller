pub mod api {
    use std::sync::{Arc};
    use log::{error, warn};
    use serde_json::json;
    use tokio::sync::Mutex;
    use crate::protocol::JsonProtocol;
    use crate::ros2_server::ros2_server::Ros2Discoverer;
    use crate::ros2entites::ros2entities::{Ros2State, Ros2Topic};

    #[derive(Clone)]
    pub struct Api {
        pub ros2discoverer: Box<Ros2Discoverer>,
    }

    impl Api {
        pub fn new(ros2discoverer: Box<Ros2Discoverer>) -> Api {
            return Api {
                ros2discoverer
            };
        }

        /**
        Generate json for ros2 state object
         */
        pub async fn ros2_state_json(&self, state: Arc<Mutex<Ros2State>>) -> String {
            let state_obj: &Ros2State = &state.lock().await.to_owned();

            let json_str = json!({
                "packages": state_obj.packages,
                "nodes": state_obj.nodes,
                "topics": state_obj.topics
            });

            return json_str.to_string();
        }

        pub async fn handle_request(&self, request: String, current_state: Arc<Mutex<Ros2State>>) -> String {
            // Parse request as json formatted str
            let mut parsed = JsonProtocol::new();
            match parsed.parse_request(request.as_str()) {
                Ok(()) => (),
                Err(msg) => error!("{}", msg)
            }
            let command = parsed.command.clone();
            let response: String = match command.as_str() {
                "state" => self.state_command(&parsed, current_state).await,
                "shutdown" => self.shutdown_node_command(&parsed, current_state),
                "configure" => self.configure_node_command(&parsed, current_state),
                "launch" => self.launch_node_command(&parsed, current_state),
                "cleanup" => self.cleanup_node_command(&parsed, current_state),
                "rename_topic" => "".to_string(),
                _ => "Unknown request".to_string()
            };

            return response;
        }

        /// This function renames topic to another name. In order to do this topic's node need to be
        /// restarted. Due this rename_topic_command works well for lifecycle nodes. It can be used with
        /// regular nodes without any guarantee of success. It returns json string contains if command
        /// was completed with success.
        /// # Arguments
        ///
        /// * `request`: request data
        /// * `_current_state`: current info about ros2 entities
        ///
        /// returns: String
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        pub async fn rename_topic_command(&self, request: &JsonProtocol, current_state: Arc<Mutex<Ros2State>>) -> String {
            let new_topic_name = if request.arguments.contains_key("new_topic_name") {
                request.arguments.get("node_name").unwrap().to_string()
            } else {
                r#"{"result": "failure", "msg": "You must provide new_topic_name argument for command rename_topic"}"#.to_string()
            };

            let old_topic_name = if request.arguments.contains_key("old_topic_name") {
                request.arguments.get("node_name").unwrap().to_string()
            } else {
                r#"{"result": "failure", "msg": "You must provide old_topic_name argument for command rename_topic"}"#.to_string()
            };

            // Find node name for topic
            let topic: Ros2Topic = current_state.lock().await.topics.iter().find(|&topic| topic.name == new_topic_name).unwrap().clone();
            let node_name = topic.node_name;
            // Check if node is running. If yes then kill it
            if self.ros2discoverer.is_node_running(node_name.clone()) {
                self.ros2discoverer.shutdown_node(node_name.clone());
            }

            // Restart node with altered topic name
            let res = self.ros2discoverer.run_node(node_name.clone(), ["--ros-args", "-r", old_topic_name.as_str(), new_topic_name.as_str()]);
            return res;
        }

        pub fn start_node_command(&self, request: &JsonProtocol, _current_state: Arc<Mutex<Ros2State>>) -> String {
            let mut error: bool = false;
            let mut error_msg: String = "".to_string();
            let package_name: String;
            if request.arguments.contains_key("package_name") {
                package_name = request.arguments.get("package_name").unwrap().to_string()
            } else {
                error = true;
                error_msg += "You must provide package_name argument for command start_node\n";
                //error_msg += r#"{"result": "failure", "msg": ""}\n"#;
            };

            let node_name: String;
            if request.arguments.contains_key("node_name") {
                node_name = request.arguments.get("node_name").unwrap().to_string();
            } else {
                error = true;
                error_msg += "You must provide node_name argument for command start_node\n";
                //error_msg += r#"{"result": "failure", "msg": "You must provide node_name argument for command start_node"}\n"#;
            };

            let host: String;
            if request.arguments.contains_key("host") {
                host = request.arguments.get("host").unwrap().to_string()
            } else {
                error = true;
                error_msg += "You must provide host argument for command start_node\n";
                //error_msg += r#"{"result": "failure", "msg": "You must provide host argument for command start_node"}\n"#;
            };

            return if error {
                r#"{"result": "failure", "msg": "#.to_string() + error_msg.as_str() + r#"}\n"#
            } else {
                r#"{"result": "success", "msg": ""}\n"#.to_string()
            };
        }

        pub async fn state_command(&self, _request: &JsonProtocol, current_state: Arc<Mutex<Ros2State>>) -> String {
            return self.ros2_state_json(current_state.clone()).await;
        }

        pub fn shutdown_node_command(&self, request: &JsonProtocol, _current_state: Arc<Mutex<Ros2State>>) -> String {
            // Extract node name from request
            let node_name = if request.arguments.contains_key("node_name") {
                request.arguments.get("node_name").unwrap().to_string()
            } else {
                r#"{"result": "failure", "msg": "You must provide node_name argument for command kill_node"}"#.to_string()
            };

            if node_name.is_empty() {
                warn!("Node {} doesn't running. So, it is impossible to be killed", node_name);
                return "".to_string();
            }

            return self.ros2discoverer.shutdown_node(node_name);
        }

        pub fn configure_node_command(&self, request: &JsonProtocol, _current_state: Arc<Mutex<Ros2State>>) -> String {
            // TODO: Implement
            return "".to_string();
        }

        pub fn launch_node_command(&self, request: &JsonProtocol, _current_state: Arc<Mutex<Ros2State>>) -> String {
            // TODO: Implement
            return "".to_string();
        }

        pub fn cleanup_node_command(&self, request: &JsonProtocol, _current_state: Arc<Mutex<Ros2State>>) -> String {
            // TODO: Implement
            return "".to_string();
        }
    }
}