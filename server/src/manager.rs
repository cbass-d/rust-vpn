use anyhow::Result;
use cli_log::error;
use common::errors::Errors;
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use x25519_dalek::PublicKey;

use crate::{peer::Peer, registry::Registry};

#[derive(Debug)]
pub enum ManagerMessages {
    AddPeer(SocketAddr, PublicKey, oneshot::Sender<Peer>),
    UpdateLastSeen(SocketAddr),
    ResolveRoute(Ipv4Addr, oneshot::Sender<Option<SocketAddr>>),
    GetAllPeers(oneshot::Sender<Vec<Peer>>),
    GetPeer(SocketAddr, oneshot::Sender<Option<Peer>>),
    RemovePeer(SocketAddr),
}

pub async fn run(
    mut rx: mpsc::UnboundedReceiver<ManagerMessages>,
    cancel_token: CancellationToken,
) -> Result<()> {
    // This task will be owner of all peer connections data
    // * socket_addr
    // * peer ip
    // * route
    // * etc.
    let mut registry = Registry::new();
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                println!("[-] Shutting down manager...");
                break;
            },

            message = rx.recv() => {
                match message {
                    Some(ManagerMessages::AddPeer(peer, client_pub, tx)) => {
                        let _ = registry.add_peer(peer, client_pub);
                        if let Some(peer) = registry.get_peer(&peer) {
                            if tx.send(peer).is_err() {
                                error!("add peer response failed: receiver dropped");
                                return Err(Errors::OneShotClosed);
                            }
                        }
                    },
                    Some(ManagerMessages::UpdateLastSeen(peer)) => {
                        registry.update_last_seen(peer, Instant::now())?;
                    },
                    Some(ManagerMessages::ResolveRoute(peer_ip, tx)) => {
                        let route = registry.get_route(&peer_ip);
                        if tx.send(route).is_err() {
                            error!("get route respnonse failed: receiver dropped");
                            return Err(Errors::OneShotClosed);
                        }
                    },
                    Some(ManagerMessages::GetAllPeers(tx)) => {
                        let all_peers = registry.get_all_peers();
                        if tx.send(all_peers).is_err() {
                            error!("get route respnonse failed: receiver dropped");
                            return Err(Errors::OneShotClosed);
                        }
                    },
                    Some(ManagerMessages::GetPeer(peer, tx)) => {
                        let peer = registry.get_peer(&peer);
                        if tx.send(peer).is_err() {
                            error!("get route respnonse failed: receiver dropped");
                            return Err(Errors::OneShotClosed);
                        }
                    },
                    Some(ManagerMessages::RemovePeer(peer)) => {
                        let _ = registry.remove_peer(peer);
                    },

                    _ => {},
                }
            },
        }
    }

    Ok(())
}
