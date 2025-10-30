use futures::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::net::Ipv4Addr;

use anyhow::Result;
use clap::Parser;
use clap_derive::Parser;
use cli_log::*;
use pnet_packet::Packet;
use pnet_packet::ip::{self};
use pnet_packet::ipv4::{self};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;
use tun::{self, AbstractDevice, BoxError, Configuration};

use common::{ClientHelloMessage, ServerHelloMessage};

const BIND_ADDRESS: &str = "localhost:9001";

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    address: Ipv4Addr,

    #[arg(short, long)]
    tun_name: Option<String>,
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

    println!("[*] Starting server...");
    start(token).await?;
    ctrlc.await?;
    Ok(())
}

// Main function for starting the server
async fn start(token: CancellationToken) -> Result<(), BoxError> {
    let socket = UdpSocket::bind(BIND_ADDRESS).await?;

    let args = Args::parse();
    let tun_name = args.tun_name;
    let address = args.address;

    println!(
        "[*] Binded to UDP Socket at {}",
        socket.local_addr().unwrap()
    );

    let mut config = Configuration::default();
    config
        .name(tun_name.unwrap_or("".to_string()))
        .up()
        .address(address)
        .netmask((255, 255, 255, 0));

    let tun = tun::create_as_async(&config).unwrap();
    println!(
        "TUN: {}, address: {}",
        tun.tun_name().unwrap(),
        tun.address().unwrap()
    );

    let mut framed_tun = tun.into_framed();

    let mut sock_buf = [0; 2048];

    let mut known_conns = HashSet::new();

    // Main event loop
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("[*] Quitting");
                break;
            },
            result = socket.recv_from(&mut sock_buf) => {
                let (len, peer) = result?;
                println!("[*] Recv {len} from {peer}");

                if !known_conns.contains(&peer) {

                    let client_hello = serde_json::from_slice::<ClientHelloMessage>(&sock_buf[..len]);
                    println!("{:?}", client_hello);

                    if let Ok(client_hello) = client_hello {
                        println!("successful client hello");
                        println!("[*] Adding new peer {peer} to hashset");
                        known_conns.insert(peer);
                    }
                    else {
                        println!("[-] Invalid client hello received");
                    }

                }
                else {
                    let ipv4_packet = ipv4::Ipv4Packet::new(&sock_buf[..len]).unwrap();

                    framed_tun.send(ipv4_packet.packet().to_vec()).await?;

                    println!("[*] Wrote to TUN");
                }

            },
            Some(packet) = framed_tun.next() => {
                //if let Ok(packet) = packet {

                //    match packet[0] >> 4 {
                //        4 => {
                //            println!("ipv4");
                //            let ipv4_packet = ipv4::Ipv4Packet::new(&packet[..]).unwrap();

                //            println!("{:?}", ipv4_packet);
                //            match ipv4_packet.get_next_level_protocol() {
                //                ip::IpNextHeaderProtocol(1) => {
                //                    println!("icmp recvd");
                //                },
                //                _ => {},
                //            }

                //        },
                //        6 => {
                //        },
                //        _ => {},
                //    }

                //}


            },
        }
    }

    Ok(())
}
