// extern crate signal_hook;

#[cfg(not(feature = "async"))]
use std::sync::mpsc::{self};
#[cfg(not(feature = "async"))]
use std::thread;
use structopt::StructOpt;
#[cfg(feature = "async")]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(feature = "async")]
use tokio::time::sleep;
//#[cfg(feature = "async")]
//use log::levelfilter;
//#[cfg(feature = "async")]
//use tracing::instrument;

mod cli;
#[cfg(not(feature = "async"))]
mod rpc_sync;
#[cfg(not(feature = "async"))]
use rpc_sync as rpc;
#[cfg(feature = "async")]
mod rpc_async;
#[cfg(feature = "async")]
use rpc_async as rpc;

#[cfg(not(feature = "async"))]
fn main() {
    let cli = cli::VaccelAgentCli::from_args();

    let mut server = rpc::new(&cli.uri).unwrap();

    server.start().unwrap();

    println!("vaccel ttRPC server started. address: {}", &cli.uri);

    // Hold the main thread until receiving signal SIGTERM
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        ctrlc::set_handler(move || {
            tx.send(()).unwrap();
        })
        .expect("Error setting Ctrl-C handler");
        println!("Server is running, press Ctrl + C to exit");
    });

    let _ = rx.recv().unwrap();
}

#[cfg(feature = "async")]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    /*
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        //.with_timer(now)
        .init();
    */

    let cli = cli::VaccelAgentCli::from_args();

    let mut server = rpc::new(&cli.uri).unwrap();

    let mut hangup = signal(SignalKind::hangup()).unwrap();
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    server.start().await.unwrap();

    println!("vaccel ttRPC server started. address: {}", &cli.uri);

	tokio::select! {
		_ = hangup.recv() => {
			// test stop_listen -> start
			println!("stop listen");
			server.stop_listen().await;
			println!("start listen");
			server.start().await.unwrap();

			// hold some time for the new test connection.
			sleep(std::time::Duration::from_secs(100)).await;
		}
		_ = interrupt.recv() => {
			// test graceful shutdown
			println!("graceful shutdown");
			server.shutdown().await.unwrap();
		}
	};
}
