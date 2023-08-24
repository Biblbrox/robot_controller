pub mod network {
    use std::cell::RefCell;
    use std::ffi::c_char;
    use std::fmt::Write;
    use std::num::ParseIntError;
    use log::{debug, error};
    use pcap::{Active, Capture, Device, IfFlags};
    use serde::{Deserialize, Serialize};
    use crate::discovery_server_impl::{ParticipantData, ReaderData, rmw_transport_SHM_TRANSPORT, rmw_transport_TCPV4_TRANSPORT, rmw_transport_TCPV6_TRANSPORT, rmw_transport_UPDV4_TRANSPORT, rmw_transport_UPDV6_TRANSPORT, WriterData};
    use crate::ros2entites::ros2entities::Host;
    use crate::snoopy::capture::PacketCapture;
    use crate::snoopy::parse::{PacketParse};

    pub struct Port {
        discovery_mcast_port: u32,
        user_mcast_port: u32,
        discovery_unicast_port: u32,
        user_unicast_port: u32,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct RTPSPacket {
        src_addr: String,
        src_port: String,
        dst_addr: String,
        dst_port: String,
        raw_data: Vec<u8>,
    }

    pub fn is_rtps(raw_data: Vec<u8>) -> bool {
        // Convert to ascii data
        let str_data: String = raw_data.iter().map(|byte| *byte as char).collect();
        return str_data.contains("RTPS");
    }


    pub struct CaptureDevice {
        pub device_name: String,
        pub capture: RefCell<Capture<Active>>,
    }

    impl CaptureDevice {
        pub fn new(device_name: String) -> Result<CaptureDevice, pcap::Error> {
            let capture = match Capture::from_device(Device::from(device_name.as_str())).unwrap()
                .promisc(true)
                .snaplen(5000)
                .timeout(1000)
                .open() {
                Ok(capture) => capture,
                Err(err) => {
                    error!("Error: {:?}", err);
                    return Err(err);
                }
            };

            let capture_device = CaptureDevice {
                device_name,
                capture: RefCell::new(capture),
            };

            return Ok(capture_device);
        }

        pub fn capture(&self, packets_num: i32) -> Vec<RTPSPacket> {
            let mut cnt: i32 = 0;
            let mut rtps_packets: Vec<RTPSPacket> = Vec::new();
            let mut capture = self.capture.borrow_mut();
            while cnt < packets_num {
                let packet = capture.next_packet().unwrap();
                let packet_parser = PacketParse::new();
                let parsed_packet = match packet_parser.parse_packet(packet.data.to_owned(), packet.header.len, packet.header.ts.tv_sec.to_string()) {
                    Ok(packet) => packet,
                    Err(_err_msg) => continue
                };
                if is_rtps(packet.data.to_owned()) {
                    // Extract src addr
                    let (src_addr, src_port, dst_addr, dst_port) = PacketCapture::get_packet_meta(&parsed_packet);
                    rtps_packets.push(RTPSPacket {
                        src_addr,
                        src_port,
                        dst_addr,
                        dst_port,
                        raw_data: packet.data.to_owned(),
                    });
                }
                cnt += 1;
            }

            return rtps_packets;
        }
    }

    pub fn running_devices() -> Vec<Device> {
        let devices = Device::list().unwrap();
        let up_devices: Vec<Device> = devices.into_iter().filter(|device| device.flags.if_flags.contains(IfFlags::UP) && device.flags.if_flags.contains(IfFlags::RUNNING)).collect();
        return up_devices;
    }

    pub fn parse_endpoint(participant_data: ParticipantData) -> String {
        let mut endpoint: String = "".to_string();
        if participant_data.transport == rmw_transport_SHM_TRANSPORT {
            endpoint = "SHM".to_string();
        } else if participant_data.transport == rmw_transport_UPDV4_TRANSPORT {
            let udpv4: String = unsafe {
                participant_data.__bindgen_anon_1.endpoint_v4.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<String>()
            };
            endpoint = format!("{}", udpv4);
        } else if participant_data.transport == rmw_transport_UPDV6_TRANSPORT {
            let udpv6: String = unsafe {
                participant_data.__bindgen_anon_1.endpoint_v6.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<String>()
            };
            endpoint = format!("{}", udpv6);
        } else if participant_data.transport == rmw_transport_TCPV4_TRANSPORT {
            let tcpv4: String = unsafe {
                participant_data.__bindgen_anon_1.endpoint_v4.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<String>()
            };
            endpoint = format!("{}", tcpv4);
        } else if participant_data.transport == rmw_transport_TCPV6_TRANSPORT {
            let tcpv6: String = unsafe {
                participant_data.__bindgen_anon_1.endpoint_v6.to_ascii_uppercase().iter().map(|&c| c.to_string()).collect::<String>()
            };
            endpoint = format!("{}", tcpv6);
        }
        return endpoint;
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

    pub fn parse_rtps(packet: RTPSPacket, port: &Port) {
        let raw_data = packet.raw_data.to_owned();

        if !(packet.dst_port == port.discovery_mcast_port.to_string()
            || packet.dst_port == port.user_mcast_port.to_string()
            || packet.src_port == port.discovery_mcast_port.to_string()
            || packet.src_port == port.user_mcast_port.to_string()) {
            return;
        }

        println!("Packet port: {:?}", packet.dst_port);
        // Print data as string
        let mut data_string = String::new();
        for byte in raw_data {
            data_string.push(byte as char);
        }

        println!("Packet data: {:?}", data_string);
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

    pub fn parse_guid(participant_data: ParticipantData) -> String {
        let mut guid: String = participant_data.guid.to_vec().iter().map(|&c| format!("{}::", encode_hex(c.to_string().as_bytes()))).collect();
        guid = guid[..guid.len() - 2].to_string();
        return guid;
    }


    pub fn string_from_c<const N: usize>(str: [::std::os::raw::c_char; N]) -> String {
        return String::from_iter(str.iter().take_while(|c| **c != 0).map(|c| *c as u8 as char));
    }
}