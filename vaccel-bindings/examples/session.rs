extern crate env_logger;

use env_logger::Env;
use log::{error, info};
use vaccel::session::Session;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    info!("Starting vAccel session handling example");

    info!("Creating new vAccel session");
    let mut sess = match Session::new(0) {
        Ok(sess) => sess,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    info!("New session {}", sess.id());

    info!("Closing session {}", sess.id());
    match sess.close() {
        Ok(()) => info!("Done"),
        Err(e) => info!("Error: {}", e),
    }
}
