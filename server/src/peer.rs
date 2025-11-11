use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};

use x25519_dalek::PublicKey;

#[derive(Debug, Clone)]
pub struct Peer {
    pub endpoint: SocketAddr,
    pub assigned_ip: Ipv4Addr,
    pub last_seen: Instant,
    pub pub_key: PublicKey,
}

impl Peer {
    pub fn update_last_seen(&mut self, instant: Instant) {
        self.last_seen = instant;
    }
}
