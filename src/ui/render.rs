use std::fmt::Write;
use std::sync::mpsc::{self, SendError, TryRecvError};
use std::thread;
use std::time::Instant;

use crate::peer_info::PeerInfo;
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
        nombre: String,
        hash_de_verificación: Vec<u8>,
        tamaño_total: u32,
        cantidad_de_piezas: u32,
        cantidad_de_peers: u32,
        piezas_faltantes: u32,
    },
    Torrent {
        nombre: String,
        hash_de_verificación: Vec<u8>,
        tamaño_total: u32,
        cantidad_de_piezas: u32,
        cantidad_de_peers: u32,
        piezas_faltantes: u32,
        cantidad_conexiones_activas: Option<u32>,
    },
    Live {
        nombre: String,
        peers_activos: Vec<PeerInfo>,
        velocidad_subida: u32,
        cantidad_de_descargadas: u32,
        tamanio_pieza: u32,
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
                    MessagesFromMain::MainViewMsg(it) => tx_ui_aux
                        .send(MessagesToUI::MainViewMsg(data_to_maininfo(it)))
                        .unwrap(),
                    MessagesFromMain::TorrentViewMsg(it) => tx_ui_aux
                        .send(MessagesToUI::TorrentViewMsg(data_to_torrentinfo(it)))
                        .unwrap(),
                    MessagesFromMain::LiveViewMsg(it) => tx_ui_aux
                        .send(MessagesToUI::LiveViewMsg(data_to_liveinfo(it, now)))
                        .unwrap(),
                    MessagesFromMain::Terminate => stop_condition.0 = true,
                }
            }

            let msg_from_ui = rx_ui.try_recv();
            if let Ok(msg_from_ui) = msg_from_ui {
                match msg_from_ui {
                    RequestMessage::MainView => tx_main_aux.send(RequestMessage::MainView).unwrap(),
                    RequestMessage::TorrentView => {
                        tx_main_aux.send(RequestMessage::TorrentView).unwrap()
                    }
                    RequestMessage::LiveView(it) => {
                        tx_main_aux.send(RequestMessage::LiveView(it)).unwrap()
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
fn data_to_maininfo(data: MainViewRawData) -> MainViewInfo {
    let mut cards = Vec::new();
    for r in data.raw_data {
        let mut title = String::new();
        let mut info = String::new();
        if let RawData::Main {
            nombre,
            hash_de_verificación,
            tamaño_total,
            cantidad_de_piezas,
            cantidad_de_peers,
            piezas_faltantes,
        } = r
        {
            title.push_str(&nombre);
            let mut hash = String::new();
            for h in hash_de_verificación {
                hash.push_str(&format!("{:x}", h));
            }
            write!(info, "Hash de verificacion: {}\n\n", hash).unwrap();
            write!(info, "Tamaño total: {} MB\n\n", tamaño_total / 1000000).unwrap();
            write!(info, "Cantidad de Piezas: {}\n\n", cantidad_de_piezas).unwrap();
            write!(info, "Cantidad de Peers: {}\n\n", cantidad_de_peers).unwrap();
            if piezas_faltantes == 0 {
                write!(info, "Estado: Almacenado\n\n").unwrap();
            } else if piezas_faltantes == cantidad_de_piezas {
                write!(info, "Estado: Por descargar\n\n").unwrap();
            } else {
                write!(info, "Estado: Descargando\n\n").unwrap();
            }
            let card = Card { title, info };
            cards.push(card);
        };
    }

    MainViewInfo { cards }
}

fn data_to_torrentinfo(data: TorrentViewRawData) -> TorrentViewInfo {
    let mut cards = Vec::new();
    for r in data.raw_data {
        let mut title = String::new();
        let mut info = String::new();
        if let RawData::Torrent {
            nombre,
            hash_de_verificación,
            tamaño_total,
            cantidad_de_piezas,
            cantidad_de_peers,
            piezas_faltantes,
            cantidad_conexiones_activas,
        } = r
        {
            title.push_str(&nombre);
            let mut hash = String::new();
            for h in hash_de_verificación {
                hash.push_str(&format!("{:x}", h));
            }
            write!(info, "Hash de verificacion: {}\n\n", hash).unwrap();
            write!(info, "Tamaño total: {} MB\n\n", tamaño_total / 1000000).unwrap();

            write!(info, "Cantidad de Piezas: {}\n\n", cantidad_de_piezas).unwrap();
            write!(info, "Cantidad de Peers: {}\n\n", cantidad_de_peers).unwrap();
            if piezas_faltantes == 0 {
                write!(info, "Estado: Almacenado\n\n").unwrap();
            } else if piezas_faltantes == cantidad_de_piezas {
                write!(info, "Estado: Por descargar\n\n").unwrap();
            } else {
                write!(info, "Estado: Descargando\n\n").unwrap();
            }
            write!(info, "Estructura del Torrent: Single File\n\n").unwrap();
            let piezas_descargadas = cantidad_de_piezas - piezas_faltantes;
            let porcentaje = (piezas_descargadas * 100) as f64 / cantidad_de_piezas as f64;
            write!(
                info,
                "Porcentaje de completitud: {}%\n\n",
                utils::round_float(porcentaje, 2)
            )
            .unwrap();
            write!(
                info,
                "Cantidad de piezas descargadas: {}\n\n",
                piezas_descargadas
            )
            .unwrap();
            if let Some(conexiones) = cantidad_conexiones_activas {
                write!(info, "Cantidad de conexiones activas: {}\n\n", conexiones).unwrap();
            } else {
                write!(info, "Cantidad de conexiones activas: 0\n\n").unwrap();
            };

            let card = Card { title, info };
            cards.push(card);
        };
    }

    TorrentViewInfo { cards }
}

fn data_to_liveinfo(data: LiveViewRawData, start: Instant) -> LiveViewInfo {
    let mut card = Card {
        title: String::new(),
        info: String::new(),
    };
    let mut title = String::new();
    let mut info = String::new();
    if let RawData::Live {
        nombre,
        mut peers_activos,
        velocidad_subida,
        cantidad_de_descargadas,
        tamanio_pieza,
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
        )
        .unwrap();
        write!(info, "Velocidad de subida: {} MB/s\n\n", velocidad_subida).unwrap();
        write!(info, "Estado del cliente: Interested\n\n").unwrap(); //REVISAR CUANDO TENGAMOS LOS ESTADOS

        for p in peers_activos {
            info.push_str("\n\n----- Peer Info -----\n\n");
            let mut id = String::new();
            for i in p.peer_id.clone() {
                id.push(i as char);
            }
            write!(info, "Peer id: {}\n\n", id).unwrap();
            write!(info, "Ip: {}\n\n", p.ip.unwrap()).unwrap();
            write!(info, "Port: {}\n\n", p.port).unwrap();
            // match p.connection_status().1 {
            //     crate::connection::ChokeStatus::Choked => write!(info, "Estado: Choked\n\n").unwrap(),
            //     crate::connection::ChokeStatus::Unchoked => write!(info, "Estado: Unchoked\n\n").unwrap(),
            // }
        }
        card = Card { title, info };
    };
    LiveViewInfo { card }
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
                    nombre: "hola".to_string(),
                    hash_de_verificación: vec![0, 0, 0],
                    tamaño_total: 21,
                    cantidad_de_piezas: 10,
                    cantidad_de_peers: 3,
                    piezas_faltantes: 3,
                }],
            }))
            .unwrap();

        render.receive_ui().unwrap();

        render.send_ui(RequestMessage::MainView).unwrap();
        render.receive_main().unwrap();
    }
}
