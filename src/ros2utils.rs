pub mod ros2utils {
    use std::env::Args;
    use std::ffi::OsStr;
    use std::process::{Command};
    use std::string::String;
    use regex;

    use std::sync::{Arc, Mutex};
    use rclrs::{Context, Node};

    use crate::ros2entites::ros2entities::{Host, Ros2NodeState, Ros2ActionClient, Ros2ActionServer, Ros2Executable, Ros2Node, Ros2Package, Ros2Publisher, Ros2ServiceClient, Ros2ServiceServer, Ros2State, Ros2Subscriber, Ros2Topic};

    pub fn ros2_init(args: Args) -> Context {
        let context = rclrs::Context::new(args).unwrap();

        let test_node = Node::new(&context, "test_node").unwrap();
        let node_names = test_node.get_node_names().unwrap();

        println!("Domain ID: {}", test_node.domain_id());
        for node_name in node_names {
            println!("Node name: {}", node_name.name);
        }

        return context;
    }

    pub fn ros2_cold_state() -> Ros2State {
        // Unsupported for now!!!
        let nodes: Vec<Ros2Node> = explore_nodes();
        //let packages: Vec<Ros2Package> = explore_packages();
        let packages: Vec<Ros2Package> = Vec::new();

        let state = Ros2State {
            nodes,
            packages,
            topics: Vec::new(),
        };

        return state;
    }

    pub fn ros2_hot_state() -> Ros2State {
        // Unsupported for now!!!
        let nodes: Vec<Ros2Node> = explore_nodes();
        let topics: Vec<Ros2Topic> = explore_topics();
        //let packages: Vec<Ros2Package> = packages();

        let state = Ros2State {
            nodes,
            topics,
            packages: Vec::new(),
        };

        return state;
    }

    pub fn ros2_state(current_state: Arc<Mutex<Ros2State>>) -> Ros2State {
        let nodes: Vec<Ros2Node> = explore_nodes();
        let mut packages: Vec<Ros2Package> = Vec::new();
        if current_state.lock().unwrap().packages.is_empty() {
            packages = explore_packages();
        } else {
            packages = current_state.lock().unwrap().packages.clone();
        }

        let topics = explore_topics();

        let state = Ros2State {
            nodes,
            packages,
            topics,
        };

        return state;
    }

    pub fn explore_nodes() -> Vec<Ros2Node> {
        let node_names = ros2_node_names();
        let mut nodes: Vec<Ros2Node> = Vec::new();
        for node_name in node_names {
            let node_info: String = node_info(node_name.clone());
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
                subscribers.push(Ros2Subscriber { name: infos[0].clone(), topic_name: infos[0].clone(), node_name: node_name.clone() });
            }

            for publisher_info in publishers_info {
                let infos: Vec<String> = publisher_info.split(':').map(|entry| entry.trim().to_string()).collect();
                publishers.push(Ros2Publisher { name: infos[0].clone(), topic_name: infos[0].clone(), node_name: node_name.clone() });
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

            let is_lifecycle: bool = is_node_lifecycle(node_name.clone());
            let lifecycle_state = match is_lifecycle {
                true => { lifecycle_state(node_name.clone()) }
                false => Ros2NodeState::NonLifecycle
            };

            // TODO: to know package name of each node
            nodes.push(Ros2Node {
                name: node_name.clone(),
                package_name: "package".to_string(),
                subscribers,
                publishers,
                action_clients,
                action_servers,
                service_clients,
                service_servers,
                host: Host::new(),
                is_lifecycle,
                state: lifecycle_state,
            })
        }

        return nodes;
    }

    pub fn node_info(node_name: String) -> String {
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

    pub fn lifecycle_state(node_name: String) -> Ros2NodeState {
        let data_bytes = Command::new("ros2")
            .arg("lifecycle")
            .arg("get")
            .arg(node_name)
            .output()
            .expect("failed to execute process");
        let state_str: String = match String::from_utf8(data_bytes.stdout) {
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

        return state;
    }

    pub fn lifecycled_node_names() -> Vec<String> {
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

    pub fn is_node_lifecycle(node_name: String) -> bool {
        return lifecycled_node_names().contains(&node_name);
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
    pub fn shutdown_node(node_name: String) -> String {
        return if is_node_lifecycle(node_name.clone()) { // Perform lifecycle scenario
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

    pub fn run_sample_node() -> String {
        let node_name = "turtle_teleop_key";
        let _output = Command::new("ros2")
            .arg("run")
            .arg("turtlesim")
            .arg(node_name)
            .spawn();

        return node_name.to_string();
    }

    pub fn is_node_running(node_name: String) -> bool {
        return ros2_node_names().contains(&node_name);
    }

    pub fn run_node<I, S>(node_name: String, args: I) -> String
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

    pub fn explore_packages() -> Vec<Ros2Package> {
        let package_names = ros2_package_names();
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

            let package = Ros2Package { name: package_name.to_string(), path: package_path(package_name.to_string()), executables };
            packages.push(package);
        };

        return packages;
    }

    pub fn explore_topics() -> Vec<Ros2Topic> {
        let topic_names = ros2_topic_names();
        let mut topics = Vec::new();
        for name in topic_names {
            let info = topic_info(name.clone());
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

            topics.push(Ros2Topic { name, node_name: node_name.to_string(), topic_type: topic_type.to_string() });
        }

        return topics;
    }

    pub fn topic_info(topic_name: String) -> String {
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

    pub fn package_path(package_name: String) -> String {
        let prefix = package_prefix(package_name);
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
    pub fn package_prefix(package_name: String) -> String {
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

    /*
    Find ros2 entities in local filesystem
     */
    /*pub fn discover_local_filesystem() -> Ros2State {
        // Check if ros2 workspace is activated
        let workspace_condition = "COLCON_PREFIX_PATH";
        let is_activated = env::var(workspace_condition).is_ok();

        if !is_activated {
            return Ros2State { state: HashMap::new() };
        }

        let install_dir = env::var(workspace_condition).unwrap();
        let src_dir = fs::canonicalize(format!("{}/../src", install_dir)).unwrap();

        // Find package names from src_dir
        let package_names = fs::read_dir(src_dir).unwrap();
        let mut packages: Vec<Ros2Entity> = Vec::new();
        for pkg_name in package_names {
            let package_name = pkg_name.unwrap().path().display().to_string();
            println!("{}", package_name);
            packages.push(Ros2Entity { name: package_name.clone(), parent_name: package_name.clone(), path: package_name.clone() });
        }

        // Find executables from src_dir


        return Ros2State { state: HashMap::new() };
    }*/

    pub fn ros2_node_names() -> Vec<String> {
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

    pub fn ros2_topic_names() -> Vec<String> {
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

    pub fn ros2_package_names() -> Vec<String> {
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

    pub fn ros2_subscriber_names() -> Vec<String> {
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

    pub fn ros2_executable_names(package_name: String) -> Vec<String> {
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
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use crate::ros2utils::ros2utils::{is_node_running, shutdown_node, run_sample_node};

    #[test]
    fn test_kill_running_node() {
        // Run sample node
        let node_name = run_sample_node();
        assert_eq!(is_node_running(node_name.clone()), true);
    }

    #[test]
    fn test_kill_node_failure() {
        let node_name = "node_name".to_string();
        let expected_response = format!("{{\"result\": \"failure\", \"msg\": \"Unable to kill node {}\"}}", node_name);
        let output = Command::new("killall").arg(node_name.clone()).output().unwrap();
        //output.status.set_success(false);
        let response = shutdown_node(node_name);
        assert_eq!(response, expected_response);
    }
}
