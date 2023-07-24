pub mod network {
    use std::io::Read;
    use pcap::{Capture, Device, IfFlags};
    use rclrs::QoSHistoryPolicy::KeepAll;
    use crate::ros2entites::ros2entities::Settings;

    pub struct Port {
        discovery_mcast_port: u32,
        user_mcast_port: u32,
        discovery_unicast_port: u32,
        user_unicast_port: u32,
    }

    pub fn running_devices() -> Vec<Device> {
        let devices = Device::list().unwrap();
        let up_devices: Vec<Device> = devices.into_iter().filter(|device| device.flags.if_flags.contains(IfFlags::UP) && device.flags.if_flags.contains(IfFlags::RUNNING)).collect();
        return up_devices;
    }

    pub fn domain_ports(domain_id: u32) -> Port {
        // calculate function
        let d0 = 0;
        let d2 = 1;
        let d1 = 10;
        let d3 = 11;
        let PB = 7400;
        let DG = 250;
        let PG = 2;

        // TODO: temporary value. For the current moment, we assume that there is only one participant
        let participant_id = 0;

        let port: Port = Port {
            discovery_mcast_port: PB + (DG * domain_id) + d0,
            user_mcast_port: PB + (DG * domain_id) + d2,
            discovery_unicast_port: PB + (DG * domain_id) + d1 + (PG * participant_id),
            user_unicast_port: PB + (DG * domain_id) + d3 + (PG * participant_id),
        };

        return port;
    }


    pub fn find_node_ip(node_name: String, domain_id: u32) -> String {
        let up_devices: Vec<Device> = running_devices();
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

        let main_device = Device::lookup().unwrap().unwrap();
        println!("Main device: {}", main_device.name);
        let mut cap = Capture::from_device(main_device).unwrap()
            .promisc(true)
            .snaplen(5000)
            .timeout(1000)
            .open().unwrap();

        let packet = cap.next_packet().unwrap();
        println!("received packet! {:?}", packet.data);
        // Print as ASCII data
        println!("received packet! {:?}", packet.data.iter().map(|&c| c as char).collect::<String>());
        /*while let Ok(packet) = cap.next_packet() {
            println!("received packet! {:?}", packet.data);
        }*/

        return "".to_string();
    }




}