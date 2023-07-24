use std::collections::HashMap;
use log::debug;
use serde_json::{Map, Value};

pub struct JsonProtocol {
    // List of all possible commands
    pub allowed_commands: Vec<String>,
    // Map of required arguments for each command
    pub allowed_arguments: HashMap<String, Vec<String>>,

    pub command: String,
    pub arguments: HashMap<String, String>,
}

impl JsonProtocol {
    pub fn new() -> JsonProtocol {
        let mut commands = Vec::new();
        let mut arguments: HashMap<String, Vec<String>> = HashMap::new();

        commands.push("state".to_string());
        arguments.insert("state".to_string(), Vec::new());

        commands.push("shutdown".to_string());
        let kill_node_args = ["node_name".to_string()].to_vec();
        arguments.insert("shutdown".to_string(), kill_node_args);

        commands.push("rename_topic".to_string());
        let rename_topic_args = ["node_name".to_string()].to_vec();
        arguments.insert("rename_topic".to_string(), rename_topic_args);

        commands.push("launch".to_string());
        let start_node_args = ["node_name".to_string()].to_vec();
        arguments.insert("launch".to_string(), start_node_args);

        commands.push("cleanup".to_string());
        let cleanup_node_args = ["node_name".to_string()].to_vec();
        arguments.insert("cleanup".to_string(), cleanup_node_args);

        commands.push("configure".to_string());
        let configure_node_args = ["node_name".to_string()].to_vec();
        arguments.insert("configure".to_string(), configure_node_args);

        return JsonProtocol {
            allowed_commands: commands,
            allowed_arguments: arguments,
            command: "".to_string(),
            arguments: HashMap::new(),
        };
    }

    /// Parse json formatted request string. Return nothing on success, error message - on error
    pub fn parse_request(&mut self, json_request: &str) -> Result<(), String> {
        debug!("Parsing json request: {}", json_request);
        let valid_example = r#"
            {
                "command": <command_name>,
                "arguments": [<argument list>]
            }
            "#;

        let trimmed = json_request.trim();
        let request_parse = serde_json::from_str(trimmed);
        if !request_parse.is_ok() {
            let msg = format!("Request must be valid json. Please, use the followed command structure: \n{}", valid_example).to_string();
            return Err(msg);
        }

        let request: Map<String, Value> = request_parse.unwrap();
        if !request.contains_key("command") {
            let msg = format!("Json request must contain command name. Please, use the followed command structure: \n{}", valid_example).to_string();
            return Err(msg);
        }
        let command = request.get("command").unwrap().as_str().unwrap().to_string();
        if !self.allowed_commands.contains(&command.clone()) {
            let msg = format!("You must use one of the following supported commands: {:?}. Command {} is not supported", self.allowed_commands, command.clone());
            return Err(msg);
        }
        self.command = command.clone();
        debug!("Command: {}", self.command);

        if !request.contains_key("arguments") {
            let msg = format!("Json request must contain arguments array. Please, use the followed command structure: \n{}", valid_example).to_string();
            return Err(msg);
        }

        for argument in request.get("arguments") {
            if argument.as_array().is_none() || argument.as_array().unwrap().len() == 0 {
                continue;
            }

            let arg_obj = argument.as_array().unwrap()[0].as_object().unwrap();
            if !arg_obj.contains_key("name") {
                let msg = "Each argument object in request must have a name field";
                return Err(msg.to_string());
            }

            if !arg_obj.contains_key("value") {
                let msg = "Each argument object in request must have a value field";
                return Err(msg.to_string());
            }

            let arg_name = arg_obj.get("name").unwrap().as_str().unwrap().to_string();
            let arg_value = arg_obj.get("value").unwrap().as_str().unwrap().to_string();

            if !self.allowed_arguments.get(&command).unwrap().contains(&arg_name) {
                let allowed_arguments = self.allowed_arguments.get(self.command.as_str()).unwrap();
                let msg = format!("Argument {} is not allowed for command {}. Allowed arguments for this command: {:?}", arg_name, self.command, allowed_arguments);
                return Err(msg);
            }

            self.arguments.insert(arg_name, arg_value);
        }

        Ok(())
    }
}