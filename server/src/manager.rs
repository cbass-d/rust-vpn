use anyhow::Result;
use cli_log::error;
use common::errors::OneShotRecvClosed;
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::registry::{ClientConnection, ClientRegistry};

#[derive(Debug)]
pub enum ManagerMessages {
    AddClient(SocketAddr, oneshot::Sender<ClientConnection>),
    UpdateLastSeen(SocketAddr),
    ResolveRoute(Ipv4Addr, oneshot::Sender<Option<SocketAddr>>),
    GetAllClients(oneshot::Sender<Vec<ClientConnection>>),
    GetClient(SocketAddr, oneshot::Sender<Option<ClientConnection>>),
    RemoveClient(SocketAddr),
}

pub async fn run(
    mut rx: mpsc::UnboundedReceiver<ManagerMessages>,
    cancel_token: CancellationToken,
) -> Result<()> {
    // This task will be owner of all client connections data
    // * socket_addr
    // * client ip
    // * route
    // * etc.
    let mut registry = ClientRegistry::new();
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                println!("[-] Shutting down manager...");
                break;
            },

            message = rx.recv() => {
                match message {
                    Some(ManagerMessages::AddClient(peer, tx)) => {
                        let _ = registry.add_client(peer);
                        if let Some(client) = registry.get_client(&peer) {
                            if tx.send(client).is_err() {
                                error!("add client response failed: receiver dropped");
                                return Err(OneShotRecvClosed.into());
                            }
                        }
                    },
                    Some(ManagerMessages::UpdateLastSeen(peer)) => {
                        registry.update_last_seen(peer, Instant::now())?;
                    },
                    Some(ManagerMessages::ResolveRoute(client_ip, tx)) => {
                        let route = registry.get_route(&client_ip);
                        if tx.send(route).is_err() {
                            error!("get route respnonse failed: receiver dropped");
                            return Err(OneShotRecvClosed.into());
                        }
                    },
                    Some(ManagerMessages::GetAllClients(tx)) => {
                        let all_clients = registry.get_all_clients();
                        if tx.send(all_clients).is_err() {
                            error!("get route respnonse failed: receiver dropped");
                            return Err(OneShotRecvClosed.into());
                        }
                    },
                    Some(ManagerMessages::GetClient(peer, tx)) => {
                        let client = registry.get_client(&peer);
                        if tx.send(client).is_err() {
                            error!("get route respnonse failed: receiver dropped");
                            return Err(OneShotRecvClosed.into());
                        }
                    },
                    Some(ManagerMessages::RemoveClient(peer)) => {
                        let _ = registry.remove_client(peer);
                    },

                    _ => {},
                }
            },
        }
    }

    Ok(())
}
