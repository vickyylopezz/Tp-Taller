use std::fmt::Write;
use std::sync::mpsc::{self, SendError, TryRecvError};
use std::thread;
use std::time::Instant;

use crate::peer::peer_handler::Peer;
use crate::utils;

//Todo estos tipos de dato que terminan en info van a ser un vector de tuplas.
//El primer valor de la tupla va a ser el id del elemento y el segundo valor
//va a ser un string con la data correspondiente a ese elemento.

//La UI en el receive solo va a tener que recorrer el vector, agarrar cada ID,
//agarrar el elemento (creo que se puede usando parent.get_element_by_id o algo así)
//y va a insertar en ese elemento el texto correspondiente.

//Se puede almancenar esa metadata en un struct o en algun lado dentro de la view
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MessagesToUI {
    MainViewMsg(MainViewInfo),
    TorrentViewMsg(TorrentViewInfo),
    LiveViewMsg(LiveViewInfo),
    Terminate,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Card {
    pub title: String,
    pub info: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MainViewInfo {
    pub cards: Vec<Card>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TorrentViewInfo {
    pub cards: Vec<Card>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LiveViewInfo {
    pub card: Card,
}

//Los tipos de dato terminados en Raw Data quieren decir data cruda. Es decir, nos va a llegar
//la data como esta en el excel. Nosotros lo que tenemos que hacer es formatearla para poder
//enviarla en formato Info.
//Para eso esta la función data_to_info (hay que implementarla)
pub enum MessagesFromMain {
    MainViewMsg(MainViewRawData),
    TorrentViewMsg(TorrentViewRawData),
    LiveViewMsg(LiveViewRawData),
    Terminate,
}

#[derive(Debug, Clone)]
pub enum RawData {
    Main {
        name: String,
        authentication_hash: Vec<u8>,
        total_size: u32,
        number_of_pieces: u32,
        number_of_peers: u32,
        remaining_pieces: u32,
    },
    Torrent {
        name: String,
        authentication_hash: Vec<u8>,
        total_size: u32,
        number_of_pieces: u32,
        number_of_peers: u32,
        remaining_pieces: u32,
        active_connections: usize,
    },
    Live {
        name: String,
        active_peers: Vec<Peer>,
        upload_speed: u32,
        downloaded_files: u32,
        piece_size: u32,
    },
}

#[derive(Debug, Clone)]
pub enum ClientStatus {
    Choked,
    Unchoked,
    Interested,
    NotInterested,
}
pub struct MainViewRawData {
    pub raw_data: Vec<RawData>,
}

#[derive(Debug)]
pub struct TorrentViewRawData {
    pub raw_data: Vec<RawData>,
}
#[derive(Debug)]
pub struct LiveViewRawData {
    pub raw_data: RawData,
}

//Estos mensajes son para solicitar la view en la que se necesita info. Los vamos a recibir de
//la UI y se puede enviar al Main. Podrían enviarse derecho al main
#[derive(Debug, PartialEq, Eq)]
pub struct TorrentId(pub String);

#[derive(Debug, PartialEq, Eq)]
pub enum RequestMessage {
    MainView,
    TorrentView,
    LiveView(TorrentId),
    Terminate,
}

#[derive(Debug)]
pub struct Render {
    //Thread encargado de formatear los datos recibidos de Main para luego
    //mandarlos a la UI
    pub data_formatter: Option<thread::JoinHandle<()>>,

    // Este es el canal que va desde el main al render,
    // usa los mensajes MessagesFromMain
    pub main_render_chnl: mpsc::Sender<MessagesFromMain>, // Main -> Render

    //Este va del Render al Main para soliciarte los datos correspondientes
    // al pain
    pub render_main_chnl: mpsc::Receiver<RequestMessage>, // Render -> Main

    //Este va del render a la ui para comunicarle los cambios que tiene que hacer?
    //O va del render a la ui quizá con el string de dato a imprimir ya armado?
    pub render_ui_chnl: mpsc::Receiver<MessagesToUI>, // Render -> UI

    // Este va de la ui al Render para avisar el cambio que se hizo en la UI
    // Quiza este mensaje canal se puede sacar y que haya uno directo de UI a Main
    pub ui_render_chnl: mpsc::Sender<RequestMessage>, // UI -> Render
    pub start_time: Instant,
}

impl Default for Render {
    fn default() -> Self {
        Self::new()
    }
}

impl Render {
    pub fn new() -> Self {
        let now = Instant::now();
        let (main_render_chnl, rx_main) = mpsc::channel();
        let (tx_main, render_main_chnl) = mpsc::channel();
        let (ui_render_chnl, rx_ui) = mpsc::channel();
        let (tx_ui, render_ui_chnl) = mpsc::channel();
        let tx_main_aux = tx_main;
        let tx_ui_aux = tx_ui;

        let mut stop_condition = (false, false);
        let data_formatter = Some(thread::spawn(move || loop {
            let info_from_main = rx_main.try_recv();

            if let Ok(info_from_main) = info_from_main {
                match info_from_main {
                    MessagesFromMain::MainViewMsg(it) => match data_to_maininfo(it) {
                        Ok(data) => match tx_ui_aux.send(MessagesToUI::MainViewMsg(data)) {
                            Ok(_) => (),
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    },
                    MessagesFromMain::TorrentViewMsg(it) => match data_to_torrentinfo(it) {
                        Ok(data) => match tx_ui_aux.send(MessagesToUI::TorrentViewMsg(data)) {
                            Ok(_) => (),
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    },
                    MessagesFromMain::LiveViewMsg(it) => match data_to_liveinfo(it, now) {
                        Ok(data) => match tx_ui_aux.send(MessagesToUI::LiveViewMsg(data)) {
                            Ok(_) => (),
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    },
                    MessagesFromMain::Terminate => stop_condition.0 = true,
                }
            }

            let msg_from_ui = rx_ui.try_recv();
            if let Ok(msg_from_ui) = msg_from_ui {
                match msg_from_ui {
                    RequestMessage::MainView => match tx_main_aux.send(RequestMessage::MainView) {
                        Ok(_) => (),
                        Err(_) => continue,
                    },
                    RequestMessage::TorrentView => {
                        match tx_main_aux.send(RequestMessage::TorrentView) {
                            Ok(_) => (),
                            Err(_) => continue,
                        }
                    }
                    RequestMessage::LiveView(it) => {
                        match tx_main_aux.send(RequestMessage::LiveView(it)) {
                            Ok(_) => (),
                            Err(_) => continue,
                        }
                    }
                    RequestMessage::Terminate => stop_condition.1 = true,
                }
            }

            if stop_condition == (true, true) {
                break;
            }
        }));

        Render {
            data_formatter,
            main_render_chnl,
            render_main_chnl,
            render_ui_chnl,
            ui_render_chnl,
            start_time: now,
        }
    }

    pub fn receive_ui(&self) -> Result<MessagesToUI, TryRecvError> {
        self.render_ui_chnl.try_recv()
    }

    pub fn receive_main(&self) -> Result<RequestMessage, TryRecvError> {
        self.render_main_chnl.try_recv()
    }

    pub fn send_main(&self, msg: MessagesFromMain) -> Result<(), SendError<MessagesFromMain>> {
        self.main_render_chnl.send(msg)
    }
    pub fn send_ui(&self, msg: RequestMessage) -> Result<(), SendError<RequestMessage>> {
        self.ui_render_chnl.send(msg)
    }
}

//parsea data a info
fn data_to_maininfo(data: MainViewRawData) -> Result<MainViewInfo, Box<dyn std::error::Error>> {
    let mut cards = Vec::new();
    for r in data.raw_data {
        let mut title = String::new();
        let mut info = String::new();
        if let RawData::Main {
            name: nombre,
            authentication_hash: hash_de_verificación,
            total_size: tamaño_total,
            number_of_pieces: cantidad_de_piezas,
            number_of_peers: cantidad_de_peers,
            remaining_pieces: piezas_faltantes,
        } = r
        {
            title.push_str(&nombre);
            let mut hash = String::new();
            for h in hash_de_verificación {
                hash.push_str(&format!("{:x}", h));
            }
            write!(info, "Hash de verificacion: {}\n\n", hash)?;
            write!(info, "Tamaño total: {} MB\n\n", tamaño_total / 1000000)?;
            write!(info, "Cantidad de Piezas: {}\n\n", cantidad_de_piezas)?;
            write!(info, "Cantidad de Peers: {}\n\n", cantidad_de_peers)?;
            if piezas_faltantes == 0 {
                write!(info, "Estado: Almacenado\n\n")?;
            } else if piezas_faltantes == cantidad_de_piezas {
                write!(info, "Estado: Por descargar\n\n")?;
            } else {
                write!(info, "Estado: Descargando\n\n")?;
            }
            let card = Card { title, info };
            cards.push(card);
        };
    }

    Ok(MainViewInfo { cards })
}

fn data_to_torrentinfo(
    data: TorrentViewRawData,
) -> Result<TorrentViewInfo, Box<dyn std::error::Error>> {
    let mut cards = Vec::new();
    for r in data.raw_data {
        let mut title = String::new();
        let mut info = String::new();
        if let RawData::Torrent {
            name: nombre,
            authentication_hash: hash_de_verificación,
            total_size: tamaño_total,
            number_of_pieces: cantidad_de_piezas,
            number_of_peers: cantidad_de_peers,
            remaining_pieces: piezas_faltantes,
            active_connections: cantidad_conexiones_activas,
        } = r
        {
            title.push_str(&nombre);
            let mut hash = String::new();
            for h in hash_de_verificación {
                hash.push_str(&format!("{:x}", h));
            }
            write!(info, "Hash de verificacion: {}\n\n", hash)?;
            write!(info, "Tamaño total: {} MB\n\n", tamaño_total / 1000000)?;

            write!(info, "Cantidad de Piezas: {}\n\n", cantidad_de_piezas)?;
            write!(info, "Cantidad de Peers: {}\n\n", cantidad_de_peers)?;
            if piezas_faltantes == 0 {
                write!(info, "Estado: Almacenado\n\n")?;
            } else if piezas_faltantes == cantidad_de_piezas {
                write!(info, "Estado: Por descargar\n\n")?;
            } else {
                write!(info, "Estado: Descargando\n\n")?;
            }
            write!(info, "Estructura del Torrent: Single File\n\n")?;
            let piezas_descargadas = cantidad_de_piezas - piezas_faltantes;
            let porcentaje = (piezas_descargadas * 100) as f64 / cantidad_de_piezas as f64;
            write!(
                info,
                "Porcentaje de completitud: {}%\n\n",
                utils::round_float(porcentaje, 2)
            )?;
            write!(
                info,
                "Cantidad de piezas descargadas: {}\n\n",
                piezas_descargadas
            )?;
            write!(
                info,
                "Cantidad de conexiones activas: {}\n\n",
                cantidad_conexiones_activas
            )?;

            let card = Card { title, info };
            cards.push(card);
        };
    }

    Ok(TorrentViewInfo { cards })
}

fn data_to_liveinfo(
    data: LiveViewRawData,
    start: Instant,
) -> Result<LiveViewInfo, Box<dyn std::error::Error>> {
    let mut card = Card {
        title: String::new(),
        info: String::new(),
    };
    let mut title = String::new();
    let mut info = String::new();
    if let RawData::Live {
        name: nombre,
        active_peers: mut peers_activos,
        upload_speed: _velocidad_subida,
        downloaded_files: cantidad_de_descargadas,
        piece_size: tamanio_pieza,
    } = data.raw_data
    {
        let now = Instant::now();
        let duration = now.duration_since(start).as_secs();
        peers_activos.dedup_by(|a, b| a.ip == b.ip && a.port == b.port);
        title.push_str(&nombre);
        let velocidad_bajada =
            (cantidad_de_descargadas * tamanio_pieza / duration as u32) as f64 / 1000000_f64;
        write!(
            info,
            "Velocidad de bajada: {} MB/s\n\n",
            utils::round_float(velocidad_bajada, 2)
        )?;
        // write!(
        //     info,
        //     "Velocidad de subida: {} MB/s\n\n",
        //     utils::round_float(velocidad_bajada * 1.07, 2)
        // )?;
        write!(info, "Estado del cliente: Interested\n\n")?;

        for p in peers_activos {
            info.push_str("\n\n----- Peer Info -----\n\n");
            let mut id = String::new();
            if let Some(peer_id) = p.peer_id {
                for i in peer_id {
                    id.push(i as char);
                }
            }

            write!(info, "Peer id: {}\n\n", id)?;
            if let Some(ip) = p.ip {
                write!(info, "Ip: {}\n\n", ip)?;
            }
            write!(info, "Port: {}\n\n", p.port)?;
        }
        card = Card { title, info };
    };
    Ok(LiveViewInfo { card })
}

impl Drop for Render {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn msg() {
        let render = Render::new();
        render
            .send_main(MessagesFromMain::MainViewMsg(MainViewRawData {
                raw_data: vec![RawData::Main {
                    name: "hola".to_string(),
                    authentication_hash: vec![0, 0, 0],
                    total_size: 21,
                    number_of_pieces: 10,
                    number_of_peers: 3,
                    remaining_pieces: 3,
                }],
            }))
            .unwrap();

        render.receive_ui().unwrap();

        render.send_ui(RequestMessage::MainView).unwrap();
        render.receive_main().unwrap();
    }
}
