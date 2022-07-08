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
    let (client, ui) = run_app().unwrap();
    client.join().unwrap();
    ui.join().unwrap();

    Ok(())
}
