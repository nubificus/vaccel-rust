// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
#[cfg(not(feature = "async"))]
use std::sync::mpsc::{self};
#[cfg(not(feature = "async"))]
use std::thread;
#[cfg(feature = "async")]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(feature = "async")]
use tokio::time::sleep;
//#[cfg(feature = "async")]
//use log::levelfilter;
//#[cfg(feature = "async")]
//use tracing::instrument;

use env_logger::Env;
#[allow(unused_imports)]
use log::{debug, info};

#[derive(Debug, Default, Parser)]
#[command(name = "vAccel RPC Agent")]
#[command(about = "A vAccel RPC agent that can respond to acceleration requests")]
pub struct Cli {
    #[arg(short = 'a')]
    #[arg(long = "server-address")]
    #[arg(help = "The server address in the format <socket-type>://<host>:<port>")]
    #[arg(default_value = "tcp://127.0.0.1:65500")]
    pub server_address: String,
}

#[cfg(not(feature = "async"))]
fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let mut server = vaccel_rpc_agent::server_init(&cli.server_address).unwrap();

    server.start().unwrap();

    info!(
        "vaccel sync ttRPC server started. address: {}",
        &cli.server_address
    );

    // Hold the main thread until receiving signal SIGTERM
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        ctrlc::set_handler(move || {
            tx.send(()).unwrap();
        })
        .expect("Error setting Ctrl-C handler");
        info!("Server is running, press Ctrl + C to exit");
    });

    rx.recv().unwrap();
}

#[cfg(feature = "async")]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    /*
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        //.with_timer(now)
        .init();
    */

    let cli = Cli::parse();
    let mut server = vaccel_rpc_agent::server_init(&cli.server_address).unwrap();

    let mut hangup = signal(SignalKind::hangup()).unwrap();
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    server.start().await.unwrap();

    info!(
        "vaccel async ttRPC server started. address: {}",
        &cli.server_address
    );

    tokio::select! {
        _ = hangup.recv() => {
            // test stop_listen -> start
            debug!("stop listen");
            server.stop_listen().await;
            debug!("start listen");
            server.start().await.unwrap();

            // hold some time for the new test connection.
            sleep(std::time::Duration::from_secs(100)).await;
        }
        _ = interrupt.recv() => {
            // test graceful shutdown
            debug!("graceful shutdown");
            server.shutdown().await.unwrap();
        }
    };
}
