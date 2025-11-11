use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use x25519_dalek::PublicKey;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum KeepAliveType {
    Request,
    Reply,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientHelloMessage {
    pub client_pub: PublicKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerHelloMessage {
    pub assigned_ip: Ipv4Addr,
    pub server_pub: PublicKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeepAliveMessage {
    pub msg_type: KeepAliveType,
}

pub enum TunMessages {
    WritePacket(Vec<u8>),
}

pub enum SocketMessages {
    WritePacket(SocketAddr, Vec<u8>),
    WritePacketToServer(Vec<u8>),
}
