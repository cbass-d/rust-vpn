use anyhow::Result;
use clap::Parser;
use clap_derive::Parser;
use cli_log::*;
use common::ClientHelloMessage;
use futures::{SinkExt, StreamExt};
use pnet_packet::Packet;
use pnet_packet::icmp::{self};
use pnet_packet::ip::{self};
use pnet_packet::ipv4::{self};
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;
use tokio_util::sync::CancellationToken;

use tun::{self, AbstractDevice, BoxError, Configuration};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    address: Ipv4Addr,

    #[arg(short, long)]
    tun_name: Option<String>,

    #[arg(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    init_cli_log!();
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let ctrlc = ctrlc2::AsyncCtrlC::new(move || {
        token_clone.cancel();
        true
    })?;

    start(token).await?;
    ctrlc.await?;
    Ok(())
}

async fn start(token: CancellationToken) -> Result<(), BoxError> {
    let args = Args::parse();

    let address = args.address;
    let tun_name = args.tun_name;
    let port = args.port;

    let bind_address = format!("localhost:{port}");
    let socket = UdpSocket::bind(bind_address).await?;
    println!("{:?}", socket);

    println!(
        "[*] Binded to UDP Socket at {}",
        socket.local_addr().unwrap()
    );

    let remote_peer = "localhost:9001";
    socket.connect(remote_peer).await?;

    println!("[*] Connected to server at: {remote_peer}");

    let cli_hello = ClientHelloMessage {};
    let cli_hello = serde_json::to_vec(&cli_hello).unwrap();

    let len = socket.send(&cli_hello[..]).await?;
    println!("sent {len}");

    //  let mut config = Configuration::default();
    //  config
    //      .tun_name(tun_name.unwrap_or("".to_string()))
    //      .address(address)
    //      .netmask((255, 255, 255, 252))
    //      .up();

    //  let dev = tun::create_as_async(&config).unwrap();

    //  println!(
    //      "TUN: {}, address: {}",
    //      dev.tun_name().unwrap(),
    //      dev.address().unwrap()
    //  );
    //  let mut framed_dev = dev.into_framed();

    //  let mut interval = time::interval(Duration::from_secs(3));

    //  let mut tun_buf = [0; 1024];

    //  let max_retries = 3;
    //  let mut tries = 0;
    //  let mut packet_id = 1;
    //  let mut icmp_seq = 0;
    //  let mut icmp_id = 1;
    //  loop {
    //      tokio::select! {
    //          _ = token.cancelled() => {
    //              println!("Quitting");
    //              break;
    //          },
    //          _ = interval.tick() => {

    //              // Send through the created TUN device
    //              // Need to build actual IP packet (TUN is Layer 3 - Network Layer)

    //              let mut icmp_buf = [0; 8];
    //              let mut echo_req = icmp::echo_request::MutableEchoRequestPacket::new(&mut icmp_buf).unwrap();
    //              echo_req.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    //              echo_req.set_icmp_code(icmp::IcmpCode(0));
    //              echo_req.set_sequence_number(icmp_seq);
    //              echo_req.set_identifier(icmp_id);
    //              let cks = {
    //                  let bytes = echo_req.packet();
    //                  let icmp = icmp::IcmpPacket::new(bytes).unwrap();
    //                  icmp::checksum(&icmp)
    //              };
    //              echo_req.set_checksum(cks);

    //              let mut buf = [0; 128];
    //              let mut ipv4_packet = ipv4::MutableIpv4Packet::new(&mut buf).unwrap();

    //              ipv4_packet.set_version(4);
    //              ipv4_packet.set_ttl(64);
    //              ipv4_packet.set_identification(packet_id);
    //              ipv4_packet.set_total_length(128);
    //              ipv4_packet.set_header_length(24);
    //              ipv4_packet.set_source(address);
    //              ipv4_packet.set_destination(Ipv4Addr::new(10, 0, 0, 3));
    //              ipv4_packet.set_next_level_protocol(ip::IpNextHeaderProtocols::Icmp);
    //              ipv4_packet.set_payload(echo_req.packet());

    //              match framed_dev.send(ipv4_packet.packet().to_vec()).await {
    //                  Ok(_) => println!("[*] Wrote to TUN"),
    //                  Err(e) => panic!("{e}"),
    //              }

    //              // Send through regular UDP Socket
    //              match socket.send(ipv4_packet.packet()).await {
    //                  Ok(len) => println!("[*] Sent {len} bytes"),
    //                  Err(e) => {
    //                      println!("[-] Error: {e}");
    //                      tries += 1;
    //                      if tries == max_retries {
    //                          panic!("{e}");
    //                      }
    //                  },
    //              }

    //              packet_id += 1;
    //              icmp_seq += 1;
    //              icmp_id += 1;
    //          },
    //          Some(packet) = framed_dev.next() => {

    //              println!("[+] Packet received");
    //          },
    //      }
    //  }

    Ok(())
}
