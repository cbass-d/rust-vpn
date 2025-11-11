use anyhow::Result;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use x25519_dalek::PublicKey;

use crate::ServerHelloMessage;

pub async fn run(
    socket: &UdpSocket,
    assigned_ip: Ipv4Addr,
    peer: &SocketAddr,
    server_pub: PublicKey,
) -> Result<()> {
    let server_hello = {
        let hello = ServerHelloMessage {
            assigned_ip,
            server_pub,
        };
        serde_json::to_vec(&hello).unwrap()
    };

    let len = socket.send_to(&server_hello[..], peer).await?;
    println!("[+] Wrote {len} bytes of sever_hello to {peer}");

    Ok(())
}
