/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod network {
    use std::cell::RefCell;
    use std::fmt::Write;
    use std::num::ParseIntError;
    use dns_lookup::lookup_addr;
    use log::{error};
    use serde::{Deserialize, Serialize};
    use crate::discovery_server_impl::{FastDDSEndpoint, ParticipantData, rmw_transport_SHM_TRANSPORT, rmw_transport_TCPV4_TRANSPORT, rmw_transport_TCPV6_TRANSPORT, rmw_transport_UPDV4_TRANSPORT, rmw_transport_UPDV6_TRANSPORT};
    use crate::ros2entites::ros2entities::Host;

    pub struct Port {
        discovery_mcast_port: u32,
        user_mcast_port: u32,
        discovery_unicast_port: u32,
        user_unicast_port: u32,
    }

    pub fn parse_endpoint(endpoint: FastDDSEndpoint) -> String {
        if endpoint.transport == rmw_transport_SHM_TRANSPORT {
            return "SHM".to_string();
        } else if endpoint.transport == rmw_transport_UPDV4_TRANSPORT {
            let endpoint_vec: Vec<String> = unsafe {
                endpoint.__bindgen_anon_1.endpoint_v4.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<Vec<String>>()
            };
            return format!("{}", endpoint_vec.join("."));
        } else if endpoint.transport == rmw_transport_UPDV6_TRANSPORT {
            let endpoint_vec: Vec<String> = unsafe {
                endpoint.__bindgen_anon_1.endpoint_v6.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<Vec<String>>()
            };
            return format!("{}", endpoint_vec.join("."));
        } else if endpoint.transport == rmw_transport_TCPV4_TRANSPORT {
            let endpoint_vec: Vec<String> = unsafe {
                endpoint.__bindgen_anon_1.endpoint_v4.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<Vec<String>>()
            };
            return format!("{}", endpoint_vec.join("."));
        } else if endpoint.transport == rmw_transport_TCPV6_TRANSPORT {
            let endpoint_vec: Vec<String> = unsafe {
                endpoint.__bindgen_anon_1.endpoint_v6.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<Vec<String>>()
            };
            return format!("{}", endpoint_vec.join("."));
        }

        return "".to_string();
    }

    pub fn domain_ports(domain_id: u32) -> Port {
        // calculate function
        let d0 = 0;
        let d2 = 1;
        let d1 = 10;
        let d3 = 11;
        let pb = 7400;
        let dg = 250;
        let pg = 2;

        // TODO: temporary value. For the current moment, we assume that there is only one participant
        let participant_id = 0;

        /*
        Theses formulas are taken from ros2 documentation website. They can be found in the javascript code.
         */
        let port: Port = Port {
            discovery_mcast_port: pb + (dg * domain_id) + d0,
            user_mcast_port: pb + (dg * domain_id) + d2,
            discovery_unicast_port: pb + (dg * domain_id) + d1 + (pg * participant_id),
            user_unicast_port: pb + (dg * domain_id) + d3 + (pg * participant_id),
        };

        return port;
    }

    pub fn find_node_host(node_name: String, domain_id: u32) -> Host {
        /*let up_devices: Vec<Device> = running_devices();
        // Print devices
        for device in up_devices {
            println!("**************************************");
            println!("Found device: {}", device.name);
            // Flags
            // Is up
            println!("Is up: {}", device.flags.if_flags.contains(IfFlags::UP));
            // Is running
            println!("Is running: {}", device.flags.if_flags.contains(IfFlags::RUNNING));
            // Is loopback
            println!("Is loopback: {}", device.flags.if_flags.contains(IfFlags::LOOPBACK));
            println!("**************************************");
        }

        let device_name = "any";
        let capture_device = CaptureDevice::new(device_name.to_string()).unwrap();

        let port = domain_ports(domain_id);

        let packages_limit: i32 = 100;
        let packets = capture_device.capture(packages_limit.clone());
        debug!("############ Capturing data from device {} ####################", device_name);
        for packet in packets {
            parse_rtps(packet, &port);
        }
        debug!("###############################################################");*/

        return Host {
            name: "localhost".to_string(),
            ip: "127.0.0.1".to_string(),
        };
    }

    pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect()
    }

    pub fn encode_hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            write!(&mut s, "{:02x}", b).unwrap();
        }
        s
    }

    /// This function is used for discarding fastdds prefixes for topic names
    ///
    /// # Arguments
    ///
    /// * `topic_name`:
    ///
    /// returns: String
    ///
    /// # Examples
    ///
    /// let topic_fastdds: String = "rt/rosout".to_string();
    /// let ros2_name = fastdds_to_ros2(topic_fastdds);
    /// assert!(ros2_name.eq("rosout"));
    pub fn fastdds_to_ros2(topic_name: String) -> String {
        if topic_name.starts_with("rr/") || topic_name.starts_with("rt/") {
            return topic_name[3..].to_string();
        }
        return topic_name;
    }


    pub fn string_from_c<const N: usize>(str: [::std::os::raw::c_char; N]) -> String {
        return String::from_iter(str.iter().take_while(|c| **c != 0).map(|c| *c as u8 as char));
    }

    fn sub_strings(string: &str, sub_len: usize) -> Vec<&str> {
        let mut subs = Vec::with_capacity(string.len() / sub_len);
        let mut iter = string.chars();
        let mut pos = 0;

        while pos < string.len() {
            let mut len = 0;
            for ch in iter.by_ref().take(sub_len) {
                len += ch.len_utf8();
            }
            subs.push(&string[pos..pos + len]);
            pos += len;
        }
        subs
    }

    pub fn hex_str_from_uc<const N: usize>(str: [::std::os::raw::c_uchar; N]) -> String {
        let hex_array = hex::encode(str);
        let hex_guid = sub_strings(hex_array.as_str(), 2).join(".");
        return hex_guid;
    }

    pub fn hostname_ip(ip_str: String) -> String {
        let ip: std::net::IpAddr = ip_str.parse().unwrap();
        let hostname = match lookup_addr(&ip) {
            Ok(name) => name,
            Err(err) => "unknown".to_string()
        };
        return hostname;
    }
}