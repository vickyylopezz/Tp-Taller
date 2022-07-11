use std::process::exit;

use bittorrent::ui::app::run_app;

use bittorrent::client::client_error::ClientError;
#[macro_use]
extern crate log;
use env_logger::Builder;
use log::LevelFilter;

fn main() -> Result<(), ClientError> {
    Builder::new()
        .filter_level(LevelFilter::max())
        .format_module_path(false)
        .init();
    info!("Starting client");
    match run_app() {
        Some((client, ui)) => {
            match client.join() {
                Ok(_) => (),
                Err(_) => exit(-1),
            };
            match ui.join() {
                Ok(_) => (),
                Err(_) => exit(-1),
            };
        }
        None => exit(-1),
    };

    Ok(())
}
