pub mod ros2_server {
    use std::ffi::OsStr;
    use std::io;
    use std::process::Command;
    use std::time::Duration;
    use grep_matcher::Matcher;
    use grep_regex::RegexMatcher;

    use std::marker::PhantomData;
    use tokio::runtime::Runtime;
    use tokio::task::JoinHandle;
    use tokio::time;
    use crate::ros2entites::ros2entities::{Host, Ros2ActionClient, Ros2ActionServer, Ros2Executable, Ros2Node, Ros2NodeState, Ros2Package, Ros2Publisher, Ros2ServiceClient, Ros2ServiceServer, Ros2Subscriber, Ros2Topic, Settings};
    use grep_searcher::{Searcher, SearcherBuilder, Sink, SinkContext, SinkMatch};
    use grep_searcher::sinks::UTF8;
    use log::warn;
    use tls_parser::nom::ToUsize;

    #[derive(Clone)]
    pub struct Ros2DiscovererParams {
        pub domain_id: u32,
    }

    #[derive(Clone)]
    pub struct Ros2Discoverer {
        params: Ros2DiscovererParams,
    }

    struct StringSink<W>(W);

    impl<W: io::Write> Sink for StringSink<W> {
        type Error = io::Error;

        fn matched(
            &mut self,
            _: &Searcher,
            mat: &SinkMatch,
        ) -> Result<bool, io::Error> {
            self.0.write_all(mat.bytes())?;
            Ok(true)
        }

        fn context(
            &mut self,
            _: &Searcher,
            ctx: &SinkContext,
        ) -> Result<bool, io::Error> {
            self.0.write_all(ctx.bytes())?;
            Ok(true)
        }

        fn context_break(
            &mut self,
            _: &Searcher,
        ) -> Result<bool, io::Error> {
            self.0.write_all(b"--\n")?;
            Ok(true)
        }
    }

    impl Ros2Discoverer {
        pub fn new(params: Ros2DiscovererParams) -> Ros2Discoverer {
            return Ros2Discoverer { params };
        }

        pub fn lifecycle_state(&self, node_name: String) -> Ros2NodeState {
            /*let mut lifecycle_get_command = Command::new("ros2")
                .arg("lifecycle")
                .arg("get")
                .arg(node_name)
                .spawn().unwrap();

            let timeout = Duration::from_secs(1);
            let status_code = match lifecycle_get_command.wait_timeout(timeout).unwrap() {
                Some(status) => status.code(),
                None => {
                    // child hasn't exited yet
                    lifecycle_get_command.kill().unwrap();
                    lifecycle_get_command.wait().unwrap().code()
                }
            };

            let state_str: String = match String::from_utf8(lifecycle_get_command.stdout.unwrap()) {
                Ok(v) => v.split_whitespace().next().unwrap().to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            let state: Ros2NodeState = match state_str.as_str() {
                "unconfigured" => Ros2NodeState::Unconfigured,
                "inactive" => Ros2NodeState::Inactive,
                "active" => Ros2NodeState::Active,
                "shutdown" => Ros2NodeState::Shutdown,
                _ => Ros2NodeState::NonLifecycle
            };

            return state;*/

            return Ros2NodeState::Active;
        }

        pub fn lifecycled_node_names(&self) -> Vec<String> {
            let data_bytes = Command::new("ros2")
                .arg("lifecycle")
                .arg("nodes")
                .output()
                .expect("failed to execute process");
            let lifecycle_nodes_str: String = match String::from_utf8(data_bytes.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let lifecycle_nodes: Vec<String> = lifecycle_nodes_str.lines().map(|node| node.to_string()).collect();
            return lifecycle_nodes;
        }

        pub fn is_node_lifecycle(&self, node_name: String) -> bool {
            return self.lifecycled_node_names().contains(&node_name);
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
        pub fn shutdown_node(&self, node_name: String) -> String {
            return if self.is_node_lifecycle(node_name.clone()) { // Perform lifecycle scenario
                let err_msg_on_shutdown = format!("Unable to set shutdown state for node {}", node_name);
                // Set shutdown state
                let output = Command::new("ros2").arg("lifecycle").arg("set").arg(node_name).arg("shutdown").output().expect(err_msg_on_shutdown.as_str());

                let response = if output.status.success() {
                    r#"{"result": "failure", "msg": "#.to_string() + err_msg_on_shutdown.as_str() + r#"}"#
                } else {
                    r#"{"result": "success"}"#.to_string()
                };

                response
            } else { // Perform 'killall' scenario
                let err_msg = format!("Unable to kill node {}", node_name);
                let output = Command::new("killall").arg(node_name.clone()).output().unwrap();
                let response = if output.status.success() {
                    r#"{"result": "failure", "msg": "#.to_string() + err_msg.as_str() + r#"}"#
                } else {
                    r#"{"result": "success"}"#.to_string()
                };
                response
            };
        }

        pub fn run_sample_node(&self) -> String {
            let node_name = "turtle_teleop_key";
            let _output = Command::new("ros2")
                .arg("run")
                .arg("turtlesim")
                .arg(node_name)
                .spawn();

            return node_name.to_string();
        }

        pub fn is_node_running(&self, node_name: String) -> bool {
            return self.ros2_node_names().contains(&node_name);
        }

        pub fn run_node<I, S>(&self, node_name: String, args: I) -> String
            where
                I: IntoIterator<Item=S>,
                S: AsRef<OsStr>
        {
            let _output = Command::new("ros2")
                .arg("run")
                .arg(node_name.clone())
                .args(args)
                .spawn();

            return node_name.to_string();
        }

        pub fn explore_packages(&self) -> Vec<Ros2Package> {
            let package_names = self.ros2_package_names();
            let mut packages: Vec<Ros2Package> = Vec::new();

            let executables_info_bytes = Command::new("ros2")
                .arg("pkg")
                .arg("executables")
                .arg("--full-path")
                .output()
                .expect("failed to execute process");
            let executables_info: Vec<String> = match String::from_utf8(executables_info_bytes.stdout) {
                Ok(v) => v.lines().map(|line| line.to_string()).collect(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            for package_name in package_names.iter() {
                // Find executables for current package
                let executable_info_lines: Vec<&str> = executables_info.iter().filter(|line| line.contains(package_name)).map(|line| line.as_str()).collect();
                if executable_info_lines.is_empty() {
                    continue;
                }

                let mut executables: Vec<Ros2Executable> = Vec::new();
                for executable_line in executable_info_lines {
                    let name = executable_line.split('/').collect::<Vec<&str>>().last().unwrap().to_string();
                    let path = executable_line.to_string();
                    executables.push(Ros2Executable { name, package_name: package_name.to_string(), path });
                }

                let package = Ros2Package { name: package_name.to_string(), path: self.package_path(package_name.to_string()), executables };
                packages.push(package);
            };

            return packages;
        }

        pub fn ros2_topic_names(&self) -> Vec<String> {
            let topics_bytes_str = Command::new("ros2")
                .arg("topic")
                .arg("list")
                .output()
                .expect("Failed to retrieve topic list from ros2");

            let topics_str = match String::from_utf8(topics_bytes_str.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let topics_names = topics_str.lines().map(String::from).collect();

            return topics_names;
        }

        pub fn ros2_package_names(&self) -> Vec<String> {
            let node_bytes_str = Command::new("ros2")
                .arg("pkg")
                .arg("list")
                .output()
                .expect("failed to execute process");

            let nodes_str = match String::from_utf8(node_bytes_str.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let node_names = nodes_str.lines().map(String::from).collect();

            return node_names;
        }

        pub fn explore_topics(&self) -> Vec<Ros2Topic> {
            let topic_names = self.ros2_topic_names();
            let mut topics = Vec::new();
            for name in topic_names {
                let info = self.topic_info(name.clone());
                let node_name_pattern = "Node name";
                if !info.contains(node_name_pattern) {
                    continue;
                }
                let node_name_line = info.lines().find(|line| line.contains(node_name_pattern)).unwrap();
                let node_name = node_name_line.split(": ").collect::<Vec<&str>>()[1];

                let topic_type_pattern = "Topic type";
                if !info.contains(topic_type_pattern) {
                    continue;
                }
                let type_line = info.lines().find(|line| line.contains(topic_type_pattern)).unwrap();
                let topic_type = type_line.split(": ").collect::<Vec<&str>>()[1];

                topics.push(Ros2Topic { name, node_name: node_name.to_string(), topic_type: topic_type.to_string(), subscribers_num: 0, publishers_num: 0 });
            }

            return topics;
        }

        pub fn topic_info(&self, topic_name: String) -> String {
            let data_bytes = Command::new("ros2")
                .arg("topic")
                .arg("info")
                .arg(topic_name.clone())
                .arg("--verbose")
                .output()
                .expect(format!("Failed to obtain info for topic {}", topic_name).as_str());
            let info: String = match String::from_utf8(data_bytes.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            return info;
        }


        pub fn package_path(&self, package_name: String) -> String {
            let prefix = self.package_prefix(package_name);
            return prefix;
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
        pub fn package_prefix(&self, package_name: String) -> String {
            let data_bytes = Command::new("ros2")
                .arg("pkg")
                .arg("prefix")
                .arg(package_name)
                .arg("--share")
                .output()
                .expect("failed to execute process");
            //String::from_utf8(node_bytes_str.stdout);
            let prefix_str: String = match String::from_utf8(data_bytes.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            return prefix_str;
        }

        pub fn ros2_node_names(&self) -> Vec<String> {
            let node_bytes_str = Command::new("ros2")
                .arg("node")
                .arg("list")
                .output()
                .expect("failed to execute process");

            let nodes_str = match String::from_utf8(node_bytes_str.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let node_names = nodes_str.lines().map(String::from).collect();

            return node_names;
        }

        pub fn ros2_subscriber_names(&self) -> Vec<String> {
            let node_bytes_str = Command::new("ros2")
                .arg("pkg")
                .arg("list")
                .output()
                .expect("failed to execute process");

            let nodes_str = match String::from_utf8(node_bytes_str.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let node_names = nodes_str.lines().map(String::from).collect();

            return node_names;
        }

        pub fn node_name_by_gid(&self, mut topic_name: String, gid_search: String) -> String {
            if !topic_name.starts_with('/') {
                topic_name = "/".to_string() + topic_name.as_str();
            }
            let data_bytes = Command::new("ros2")
                .arg("topic")
                .arg("info")
                .arg(topic_name.clone())
                .arg("--verbose")
                .output()
                .expect("failed to execute process");

            if !data_bytes.stderr.is_empty() {
                let data_str = match String::from_utf8(data_bytes.stderr) {
                    Ok(v) => v.to_string(),
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                warn!("Error occur with ROS2 CLI command: {data_str}");
            }

            let data_str = match String::from_utf8(data_bytes.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            let mut node_name: String = "".to_string();
            for line in data_str.lines() {
                if line.starts_with("Node name:") {
                    let mut iter = line.split(": ");
                    iter.next();
                    node_name = iter.next().unwrap().to_string();
                }

                if line.starts_with("GID") {
                    let mut iter = line.split(": ");
                    iter.next();
                    let gid = iter.next().unwrap();
                    if gid.starts_with(gid_search.as_str()) {
                        return node_name;
                    }
                }
            }

            return "".to_string();
        }

        pub fn ros2_executable_names(&self, package_name: String) -> Vec<String> {
            let node_bytes_str = Command::new("ros2")
                .arg("pkg")
                .arg("executables")
                .arg(package_name)
                .output()
                .expect("failed to execute process");

            let nodes_str = match String::from_utf8(node_bytes_str.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let node_names = nodes_str.lines().map(String::from).collect();

            return node_names;
        }

        pub fn node_info(&self, node_name: String) -> String {
            let data_bytes = Command::new("ros2")
                .arg("node")
                .arg("info")
                .arg(node_name)
                .output()
                .expect("failed to execute process");
            let info: String = match String::from_utf8(data_bytes.stdout) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            return info;
        }

        pub fn explore_node(&self, node_name: String) -> Result<Ros2Node, String> {
            let node_info: String = self.node_info(node_name.clone());
            if node_info.starts_with("Unable") {
                return Err(node_info);
            }

            let re = regex::Regex::new(r"Subscribers|Publishers|Service Servers|Service Clients|Action Servers|Action Clients").unwrap();
            let parts: Vec<String> = re.split(node_info.as_str()).map(|x| x.to_string())
                .collect();
            let subscribers_info: Vec<String> = parts[1].lines().skip(1).map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect();
            let publishers_info: Vec<String> = parts[2].lines().skip(1).map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect();
            let service_servers_info: Vec<String> = parts[3].lines().skip(1).map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect();
            let service_clients_info: Vec<String> = parts[4].lines().skip(1).map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect();
            let action_servers_info: Vec<String> = parts[5].lines().skip(1).map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect();
            let action_clients_info: Vec<String> = parts[6].lines().skip(1).map(|line| line.trim().to_string()).filter(|line| !line.is_empty()).collect();

            let mut subscribers: Vec<Ros2Subscriber> = Vec::new();
            let mut publishers: Vec<Ros2Publisher> = Vec::new();
            let mut service_servers: Vec<Ros2ServiceServer> = Vec::new();
            let mut service_clients: Vec<Ros2ServiceClient> = Vec::new();
            let mut action_servers: Vec<Ros2ActionServer> = Vec::new();
            let mut action_clients: Vec<Ros2ActionClient> = Vec::new();

            for subscriber_info in subscribers_info {
                let infos: Vec<String> = subscriber_info.split(':').map(|entry| entry.trim().to_string()).collect();
                subscribers.push(Ros2Subscriber { topic_name: infos[0].clone(), topic_type: infos[0].clone(), node_name: node_name.clone(), guid: "unknown".to_string(), host: Host::default() });
            }

            for publisher_info in publishers_info {
                let infos: Vec<String> = publisher_info.split(':').map(|entry| entry.trim().to_string()).collect();
                publishers.push(Ros2Publisher { topic_name: infos[0].clone(), topic_type: infos[0].clone(), node_name: node_name.clone(), guid: "unknown".to_string(), host: Host::default() });
            }

            for service_server_info in service_servers_info {
                let infos: Vec<String> = service_server_info.split(':').map(|entry| entry.trim().to_string()).collect();
                service_servers.push(Ros2ServiceServer { name: infos[0].clone(), topic_name: infos[0].clone(), node_name: node_name.clone() });
            }

            for service_client_info in service_clients_info {
                let infos: Vec<String> = service_client_info.split(':').map(|entry| entry.trim().to_string()).collect();
                service_clients.push(Ros2ServiceClient { name: infos[0].clone(), topic_name: infos[0].clone(), node_name: node_name.clone() });
            }

            for action_client_info in action_clients_info {
                let infos: Vec<String> = action_client_info.split(':').map(|entry| entry.trim().to_string()).collect();
                action_clients.push(Ros2ActionClient { name: infos[0].clone(), topic_name: infos[0].clone(), node_name: node_name.clone() });
            }

            for action_server_info in action_servers_info {
                let infos: Vec<String> = action_server_info.split(':').map(|entry| entry.trim().to_string()).collect();
                action_servers.push(Ros2ActionServer { name: infos[0].clone(), topic_name: infos[0].clone(), node_name: node_name.clone() });
            }

            let is_lifecycle: bool = self.is_node_lifecycle(node_name.clone());
            let lifecycle_state = match is_lifecycle {
                true => { self.lifecycle_state(node_name.clone()) }
                false => Ros2NodeState::NonLifecycle
            };

            //let host = self.find_node_host(node_name.clone(), self.params.domain_id);
            let host = Host::default();
            // TODO: to know package name of each node

            return Ok(Ros2Node {
                name: node_name.clone(),
                package_name: "package".to_string(),
                subscribers,
                publishers,
                action_clients,
                action_servers,
                service_clients,
                service_servers,
                host,
                is_lifecycle,
                state: lifecycle_state,
            });
        }

        pub fn explore_nodes(&self) -> Vec<Ros2Node> {
            let node_names = self.ros2_node_names();
            let mut nodes: Vec<Ros2Node> = Vec::new();
            for node_name in node_names {
                let node = self.explore_node(node_name).unwrap();
                nodes.push(node);
            }

            return nodes;
        }
    }
}

#[cfg(test)]
mod tests {
    use log::debug;
    use crate::ros2_server::ros2_server::{Ros2Discoverer, Ros2DiscovererParams};

    #[test]
    fn node_name_by_gid_test() {
        //let ros2_discoverer = Ros2Discoverer::new(Ros2DiscovererParams { domain_id: 0, interval: 1000 });
        //ros2_discoverer.node_name_by_gid("/turtle1/cmd_vel".to_string(), "0x7f0000000001".to_string());
    }
}