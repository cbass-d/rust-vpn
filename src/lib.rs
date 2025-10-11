use packet_parser::parse::internet::Internet;
use pnet::packet::Packet;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;

mod device;
mod tests;

pub fn parse_packet(data: &[u8]) {
    let internet = Internet::try_from(data).unwrap();
    println!("{:?}", internet);

    //match internet.protocol_name.as_str() {
    //    "IPv4" => {
    //        let packet = Ipv4Packet::new(internet.payload).unwrap();
    //        println!("{:?}", packet);
    //    }
    //    "IPv6" => {
    //        let packet = Ipv6Packet::new(internet.payload).unwrap();
    //        println!("{:?}", packet);
    //    }
    //    _ => {}
    //}
    //match data[0] >> 4 {
    //    4 => {
    //        let packet = Ipv4Packet::new(data).unwrap();
    //        println!("{:?}", packet);
    //    }
    //    6 => {
    //        let packet = Ipv6Packet::new(data).unwrap();
    //        println!("{:?}", packet);
    //    }
    //    _ => {}
    //}
}
