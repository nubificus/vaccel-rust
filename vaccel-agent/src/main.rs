extern crate signal_hook;

use std::thread;
use std::sync::mpsc::{self};

mod rpc;

fn main() {
    let mut server = rpc::start("unix:///tmp/1");

    let _ = server.start().unwrap();

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
