use anyhow::Result;
use cli_log::error;
use common::{
    errors::Errors,
    messages::{ClientHelloMessage, ServerHelloMessage},
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};
use x25519_dalek::PublicKey;

use tokio::{net::UdpSocket, time::timeout};

pub async fn run(
    socket: &UdpSocket,
    server: SocketAddr,
    client_pub: PublicKey,
) -> Result<(Ipv4Addr, PublicKey)> {
    println!("[*] Connecting to server...");
    socket.connect(server).await?;

    // Send over client hello to server
    let cli_hello = {
        let hello = ClientHelloMessage { client_pub };
        serde_json::to_vec(&hello).unwrap()
    };
    let len = socket.send(&cli_hello[..]).await?;
    println!("[*] Sent client_hello of {len} bytes");

    // Wait for response from server for 5 seconds
    let mut sock_buf = [0; 1024];
    println!("[*] Waiting for server_hello...");
    if let Ok((len, peer)) =
        timeout(Duration::from_secs(5), socket.recv_from(&mut sock_buf[..])).await?
    {
        // Check for valid ServerHello and verify source
        if peer != server {
            error!("[-] Unexpected source address of message");
            return Err(Errors::UnexpectedSourece.into());
        }

        let server_hello = serde_json::from_slice::<ServerHelloMessage>(&sock_buf[..len])?;
        let server_pub = server_hello.server_pub;
        let assigned_addr = server_hello.assigned_ip;
        println!("[*] Recived server_hello with assigned ip: {assigned_addr}");

        Ok((assigned_addr, server_pub))
    } else {
        return Err(Errors::Timeout.into());
    }
}
