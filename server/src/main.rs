use anyhow::Result;
use clap::Parser;
use clap_derive::Parser;
use cli_log::*;
use std::net::Ipv4Addr;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

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
async fn main() -> Result<()> {
    init_cli_log!();

    let token = CancellationToken::new();
    let token_clone = token.clone();

    let args = Args::parse();
    let tun_name = args.tun_name;
    let address = args.address;
    let port = args.port;

    let mut task_set = JoinSet::new();
    task_set.spawn(server::start(token, tun_name, address, port));

    println!("[*] Starting server...");
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                token_clone.cancel();
                break;
            },
            Some(res) = task_set.join_next() => {
                match res {
                    Ok(t) if t.is_err() => {
                        error!("server ended with error: {t:?}");
                    },
                    Ok(t) if t.is_ok() => {
                        info!("server finished");
                    },
                    Err(e) => {
                        error!("main server join falied: {e}");
                    }
                    _ => {},
                }
            },
        }
    }

    task_set.join_all().await;

    Ok(())
}
