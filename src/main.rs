/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate core;

use std::env::args;
use std::io::{self};

use std::{env, fs, str};
use std::error::Error;
use std::ops::Deref;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, exit};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;
use futures::TryFutureExt;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixStream, UnixListener};
use tokio::runtime::Runtime;
use log::{debug, error, info, trace};
use crate::ros2entites::ros2entities::{Ros2State, Settings};

use tokio::{time};
use tokio::sync::{Mutex};
use crate::api::api::Api;

use crate::discovery_server::discovery_server::{DiscoveryFlags, DiscoveryServer};


use crate::ros2_wrapper::ros2;


mod ros2entites;
mod ros2utils;
mod api;
mod protocol;
mod network;

mod discovery_server_impl;
mod discovery_server;
mod ros2_wrapper;
mod fastdds_server;
mod ros2_server;

/**
Handle client json request
 */
async fn handle_client<'a>(mut stream: UnixStream, current_state: Arc<Mutex<Ros2State>>, api: Arc<Api>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Read request from stream
    let mut buffer = [0u8; 1024];
    let _nbytes = stream.read(&mut buffer[..]).await?;
    let request = str::from_utf8(&buffer).unwrap().trim().trim_matches(char::from(0));

    // Create response
    let response: String = api.handle_request(request.to_string(), current_state).await;
    debug!("Json string for response: {}", response);

    // Write back response to client
    let msg_len: u64 = response.as_bytes().len() as u64;
    stream.write_u64(msg_len).await?; // Write message header indicates message length
    stream.write_all(&response.as_bytes()).await?; // Write the body
    Ok(())
}

fn main() -> io::Result<()> {
    let ld_library_path = env::var("LD_LIBRARY_PATH").unwrap();

    env::set_var("LD_LIBRARY_PATH", format!("./src/c/lib/nodegraph:{ld_library_path}"));
    env::set_var("ROS_DISCOVERY_SERVER", "0.0.0.0:11811");
    env::set_var("FASTRTPS_DEFAULT_PROFILES_FILE", "super_client_configuration_file.xml");


    Command::new("ros2").arg("daemon").arg("stop").output().unwrap();
    Command::new("ros2").arg("daemon").arg("start").output().unwrap();
    sleep(Duration::from_secs(1));

    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let settings: Settings = Settings {
        domain_id: 1,
        include_internals: false,
        dds_topic_type: false,
    };

    let ctrlc_pressed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let ctrlc_pressed_setter = ctrlc_pressed.clone();
    ctrlc::set_handler(move || {
        info!("received Ctrl+C!");
        ctrlc_pressed_setter.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    ros2::init(args());

    let mut discovery_flags = DiscoveryFlags::EnableFastdds | DiscoveryFlags::EnableROS2;
    if settings.include_internals {
        discovery_flags |= DiscoveryFlags::IncludeInternals;
    }

    let (mut rx_state, mut discovery_server) = DiscoveryServer::new(settings.domain_id, discovery_flags);

    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let ros2discoverer = discovery_server.ros2_discoverer.clone();

    let api: Arc<Api> = Arc::new(Api::new(ros2discoverer));

    let socket_name = "/tmp/ros2monitor.sock";
    if Path::new(socket_name).exists() {
        fs::remove_file(socket_name).expect("Unable to release socket");
    }

    let listener = UnixListener::bind(socket_name)?;
    // Set socket permissions for non root users. TODO: make it in more secure way
    fs::set_permissions(socket_name, fs::Permissions::from_mode(0o777))?;

    let current_state = Arc::new(Mutex::<Ros2State>::new(Ros2State::new(false)));

    let ctrl_pressed_check = ctrlc_pressed.clone();
    let signal_handler = rt.spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        let seconds = interval.period().as_secs();
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
            trace!("Every minute at 00'th and {seconds}'th second");
        }
    });

    rt.spawn(signal_handler);


    // Accept connections from clients
    rt.spawn(async move {
        loop {
            // Check state
            let current_state = current_state.clone();
            let state = match rx_state.latest() {
                Some(state) => Arc::new(Mutex::new(state.clone())),
                None => current_state.clone()
            };

            let (stream, _) = listener.accept().await.unwrap();

            let api_clone = api.clone();
            tokio::spawn(async move {
                let _res = handle_client(stream, state, api_clone).await;
            });
        }
    });

    // Blocking operation
    discovery_server.run();
    return Ok(());
}