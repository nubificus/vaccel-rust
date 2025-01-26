// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
#[cfg(not(feature = "async"))]
use std::sync::mpsc::{self};
#[cfg(not(feature = "async"))]
use std::thread;
#[cfg(feature = "async")]
use tokio::signal::unix::{signal, SignalKind};
use vaccel_rpc_agent::{Agent as VaccelRpcAgent, Cli};
//#[cfg(feature = "async")]
//use log::levelfilter;
//#[cfg(feature = "async")]
//use tracing::instrument;

use env_logger::Env;
#[allow(unused_imports)]
use log::{debug, info};

#[cfg(not(feature = "async"))]
fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    let mut agent = VaccelRpcAgent::new(&cli.server_address);
    if let Some(vaccel_config) = cli.vaccel_config {
        agent
            .set_vaccel_config(vaccel_config.try_into().unwrap())
            .unwrap();
    }

    agent.start().unwrap();

    info!("vAccel RPC agent started");

    // Hold the main thread until receiving SIGINT, SIGTERM or SIGHUP
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        ctrlc::set_handler(move || {
            tx.send(()).unwrap();
        })
        .expect("Error setting Ctrl-C handler");
        info!(
            "Listening on '{}', press Ctrl+C to exit",
            &cli.server_address
        );
    });

    rx.recv().unwrap();
    agent.shutdown().unwrap();
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

    let mut agent = VaccelRpcAgent::new(&cli.server_address);
    if let Some(vaccel_config) = cli.vaccel_config {
        agent
            .set_vaccel_config(vaccel_config.try_into().unwrap())
            .unwrap();
    }

    agent.start().await.unwrap();

    info!(
        "vAccel async ttrpc server started. Address: {}",
        &cli.server_address
    );

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sighup = signal(SignalKind::hangup()).unwrap();

    // Hold the main thread until receiving SIGINT, SIGTERM or SIGHUP
    info!("vAccel RPC agent is running, press Ctrl+C to exit");
    tokio::select! {
        _ = sigint.recv() => {}
        _ = sigterm.recv() => {}
        _ = sighup.recv() => {}
    };

    agent.shutdown().await.unwrap();
}
