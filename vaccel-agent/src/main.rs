use std::sync::mpsc::{self};
use std::thread;
use structopt::StructOpt;

mod cli;
mod rpc;

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
