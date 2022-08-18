#[macro_use]
extern crate log;

use env_logger::Builder;
use log::LevelFilter;
use tracker::http::server;
use tracker::Tracker;

fn main() {
    Builder::new()
        .filter_level(LevelFilter::max())
        .format_module_path(false)
        .init();
    info!("Starting server...");
    let s = server::HttpServer::new("127.0.0.1:7878").unwrap();
    let h = Box::new(Tracker::new());
    let api = tracker::build_endpoints();
    s.listen(h, api);
}
