use bittorrent::client::Client;

fn main() {
    let cliente = Client::new();
    cliente.run();
}
