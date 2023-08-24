extern crate core;

use std::env::args;
use std::io::{self};

use std::{fs, str};

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixStream, UnixListener};
use tokio::runtime::Runtime;
use log::{debug, error, warn};
use crate::ros2entites::ros2entities::{Ros2State, Settings};

use tls_parser::nom::{AsChar, HexDisplay, ToUsize};
use tokio::time;

use crate::api::api::handle_request;
use crate::discovery_server::discovery_server::{is_discovery_running, run_discovery_server, stop_discovery_server};
use crate::discovery_server_impl::{ParticipantData, ReaderData, WriterData};

use crate::network::network::{find_node_host, parse_endpoint, parse_guid, string_from_c};
use crate::ros2_wrapper::ros2;
use crate::ros2utils::ros2utils::{ros2_state};

mod snoopy;
mod ros2entites;
mod ros2utils;
mod api;
mod protocol;
mod network;
mod rpc;
mod discovery_server_impl;
mod discovery_server;
mod ros2_wrapper;

/**
Handle client json request
 */
async fn handle_client(mut stream: UnixStream, current_state: Arc<Mutex<Ros2State>>) -> io::Result<()> {
    // Read request from stream
    let mut buffer = [0u8; 1024];
    let _nbytes = stream.read(&mut buffer[..]).await?;
    let request = str::from_utf8(&buffer).unwrap().trim().trim_matches(char::from(0));

    // Create response
    let response: String = handle_request(request.to_string(), current_state);
    debug!("Json string for response: {}", response);

    // Write back response to client
    let msg_len: u64 = response.as_bytes().len() as u64;
    stream.write_u64(msg_len).await?; // Write message header indicates message length
    stream.write_all(&response.as_bytes()).await?; // Write the body
    Ok(())
}

fn on_participant_discovery(participant_data: ParticipantData) {
    debug!("---------------------On participant discovery---------------------");
    let endpoint = parse_endpoint(participant_data);
    let guid = parse_guid(participant_data);
    debug!("Port: {}", participant_data.port);
    debug!("Guid: {:?}", guid);
    debug!("Raw guid: {:?}", participant_data.guid);
    debug!("Endpoint: {}", endpoint);
    debug!("------------------------------------------------------------------");
}

fn on_reader_discovery(reader_data: ReaderData) {
    debug!("------------------------On reader discovery-----------------------");
    let topic_name = string_from_c(reader_data.topic_name);
    let topic_type = string_from_c(reader_data.type_name);
    debug!("Topic name: {}", topic_name);
    debug!("Topic type: {}", topic_type);
    debug!("------------------------------------------------------------------");
}

fn on_writer_discovery(writer_data: WriterData) {
    debug!("------------------------On writer discovery-----------------------");
    let topic_name = string_from_c(writer_data.topic_name);
    let topic_type = string_from_c(writer_data.type_name);
    debug!("Topic name: {}", topic_name);
    debug!("Topic type: {}", topic_type);
    debug!("------------------------------------------------------------------");
}

fn init_discovery_server(settings: Settings) {
    if is_discovery_running(settings.domain_id) {
        warn!("Discovery server is already running. Trying to stop it");
        stop_discovery_server(settings.domain_id);
    }
    run_discovery_server(settings.domain_id, on_participant_discovery, on_reader_discovery, on_writer_discovery);
}

fn main() -> io::Result<()> {
    //#[link(name = "libnode_graph", kind = "static")]
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    let settings: Settings = Settings {
        domain_id: 1
    };

    let ctrlc_pressed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let ctrlc_pressed_setter = ctrlc_pressed.clone();
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        ctrlc_pressed_setter.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    ros2::init(args());

    init_discovery_server(settings.clone());

    find_node_host("".to_string(), settings.domain_id.into());

    let socket_name = "/tmp/ros2monitor.sock";
    if Path::new(socket_name).exists() {
        std::fs::remove_file(socket_name).expect("Unable to release socket");
    }
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();
    let listener = UnixListener::bind(socket_name)?;
    // Set socket permissions for non root users. TODO: make it in more secure way
    fs::set_permissions(socket_name, fs::Permissions::from_mode(0o777))?;

    let current_state = Arc::new(Mutex::<Ros2State>::new(Ros2State::new()));

    let state_clone = current_state.clone();

    let ctrl_pressed_check = ctrlc_pressed.clone();
    let update_ros2_state = rt.spawn(async move {
        let mut interval = time::interval(Duration::from_millis(10000));
        loop {
            if ctrl_pressed_check.load(Ordering::SeqCst) {
                debug!("Shutting down ros2");
                let res = ros2::shutdown();
                if !res {
                    error!("Unable to shutdown ros2");
                }
                exit(0);
            }

            interval.tick().await;
            debug!("Every minute at 00'th and 30'th second");
            *state_clone.lock().unwrap() = ros2_state(state_clone.clone(), settings.clone());
        }
    });

    rt.spawn(update_ros2_state);

    // Accept connections from clients

    rt.block_on(async {
        loop {
            let (stream, _) = listener.accept().await?;
            let current_state = current_state.clone();
            tokio::spawn(async move {
                let _res = handle_client(stream, current_state).await;
            });
        }
    })
}