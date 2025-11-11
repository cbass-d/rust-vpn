use crate::{errors::ServerError, peer::Peer};
use anyhow::Result;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};
use x25519_dalek::PublicKey;

pub struct Registry {
    pub peers: HashMap<SocketAddr, Peer>,
    pub ip_to_sock: HashMap<Ipv4Addr, SocketAddr>,
    pub dead_connections: HashSet<Peer>,
    pub address_pool: VecDeque<Ipv4Addr>,
}

impl Registry {
    pub fn new() -> Self {
        let addresses =
            ipnet::Ipv4AddrRange::new("10.0.0.2".parse().unwrap(), "10.0.0.10".parse().unwrap());
        Self {
            peers: HashMap::new(),
            ip_to_sock: HashMap::new(),
            dead_connections: HashSet::new(),
            address_pool: VecDeque::from_iter(addresses),
        }
    }

    pub fn add_peer(
        &mut self,
        endpoint: SocketAddr,
        pub_key: PublicKey,
    ) -> Result<(), ServerError> {
        if let Some(address) = self.address_pool.pop_front() {
            println!("[+] Adding new peer {endpoint} to hashset with IP {address}");
            self.peers.insert(
                endpoint,
                Peer {
                    endpoint,
                    assigned_ip: address,
                    last_seen: Instant::now(),
                    pub_key,
                },
            );
            self.ip_to_sock.insert(address, endpoint);
        } else {
            return Err(ServerError::NoAddressLeft);
        }

        Ok(())
    }

    pub fn get_peer(&self, peer: &SocketAddr) -> Option<Peer> {
        self.peers.get(peer).cloned()
    }

    pub fn get_all_peers(&self) -> Vec<Peer> {
        self.peers.values().cloned().collect()
    }

    pub fn get_route(&self, ip: &Ipv4Addr) -> Option<SocketAddr> {
        self.ip_to_sock.get(ip).cloned()
    }

    pub fn remove_peer(&mut self, endpoint: SocketAddr) -> Result<(), ServerError> {
        if let Some(peer) = self.peers.get(&endpoint) {
            println!("[-] Removing {endpoint} from connections");
            let ip = peer.assigned_ip;
            self.address_pool.push_front(ip);
            self.ip_to_sock.remove_entry(&ip);
        } else {
            return Err(ServerError::NoSuchClient);
        }

        self.peers.remove_entry(&endpoint);
        Ok(())
    }

    pub fn update_last_seen(
        &mut self,
        peer: SocketAddr,
        instant: Instant,
    ) -> Result<(), ServerError> {
        if let Some(peer) = self.peers.get_mut(&peer) {
            peer.update_last_seen(instant);
        } else {
            return Err(ServerError::NoSuchClient);
        }
        Ok(())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
