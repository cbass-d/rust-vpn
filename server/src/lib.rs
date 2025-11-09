pub mod errors;
pub mod handshake;
pub mod keep_alive;
pub mod manager;
pub mod registry;
pub mod socket;
pub mod tun;

pub use cli_log::*;
pub use common::{errors::*, messages::*};

use anyhow::Result;
use std::net::Ipv4Addr;
use tokio::{net::UdpSocket, sync::mpsc};
use tokio_util::sync::CancellationToken;

use crate::{manager::ManagerMessages, socket::SocketMessage, tun::TunMessage};

pub async fn start(
    token: CancellationToken,
    tun_name: Option<String>,
    address: Ipv4Addr,
    port: u16,
) -> Result<()> {
    let bind_adress = format!("0.0.0.0:{port}");
    let socket = UdpSocket::bind(bind_adress).await?;
    println!(
        "[*] Binded to UDP Socket at {}",
        socket.local_addr().unwrap()
    );

    let tun_device = tun::create_tun(tun_name, address)?;
    let framed_device = tun_device.into_framed();

    // Channels for message/packet passing:
    // * messages for manager (adding and removing clients, and other stuff
    // * passing packets between the tun interface and the UDP socket facing the clients
    let (manager_tx, manager_rx) = mpsc::unbounded_channel::<ManagerMessages>();
    let (tun_tx, tun_rx) = mpsc::unbounded_channel::<TunMessage>();
    let (socket_tx, socket_rx) = mpsc::unbounded_channel::<SocketMessage>();

    // Spawn each of the task in a task set, for easier management of joining,
    // aborting, logging, etc.
    let mut task_set = tokio::task::JoinSet::new();

    let _manager_task = task_set.spawn(manager::run(manager_rx, token.clone()));

    let _tun_task = task_set.spawn(tun::run(
        framed_device,
        manager_tx.clone(),
        socket_tx.clone(),
        tun_rx,
        token.clone(),
    ));

    let _udp_socket_task = task_set.spawn(socket::run(
        socket,
        manager_tx.clone(),
        tun_tx.clone(),
        socket_rx,
        token.clone(),
    ));

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("[-] Shutting down server");
                break;
            },
            Some(res) = task_set.join_next_with_id() => {
                match res {
                    Ok((id, t)) if t.is_err() => {
                        error!("task with id {id} failed: {t:?}");
                        token.cancel();
                    },
                    Ok((id, t)) if t.is_ok() => {
                        info!("task with id {id} finished");
                        token.cancel();
                    },
                    Err(e) => {
                        error!("task join falied: {e}");
                        break;
                    }
                    _ => {},
                }
            },
        }
    }

    // Wait for all to finish
    task_set.join_all().await;
    Ok(())
}
