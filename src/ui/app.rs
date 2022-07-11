use crate::client::client_handler::Client;
use crate::config;
use crate::log::logger::Logger;

use super::render::{Card, Render};
use super::utils;
use super::views::live_view::LiveView;
use super::views::main_view::MainView;
use super::views::torrent_view::TorrentView;
use gtk::{prelude::*, Box};
use gtk::{Application, ApplicationWindow, Button};
use gtk::{CssProvider, Orientation};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Clone)]
pub struct LeftPaneElements {
    pub main_bt: Button,
    pub torrent_bt: Button,
}

pub enum Panes {
    MainPane,
    TorrentPane,
    LivePane,
    None,
}

pub fn run_app() -> Option<(JoinHandle<()>, JoinHandle<()>)> {
    gtk::init().unwrap();
    let threads = init_window();
    gtk::main();
    threads
}

fn init_css(win: &ApplicationWindow) {
    let css = CssProvider::new();
    css.load_from_path("src/ui/main.css").unwrap();
    let screen = win.screen().unwrap();
    gtk::StyleContext::add_provider_for_screen(
        &screen,
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn init_window() -> Option<(JoinHandle<()>, JoinHandle<()>)> {
    let application = Application::builder().application_id("main-app").build();
    let file = fs::File::open("client.config").ok();

    let config = match file {
        Some(f) => config::Config::new(f).ok()?,
        None => config::Config::default(),
    };

    let render = Arc::new(Mutex::new(Render::new()));

    let client = Rc::new(RefCell::new(
        Client::new(Arc::clone(&render), &config).unwrap(),
    ));

    let log_file = fs::File::create(Path::new(&format!("{}run.log", config.logs()))).ok()?;
    let logger = Logger::new(log_file);
    let threads = match Client::run(
        client.borrow_mut().torrents.clone(),
        render,
        config,
        &logger,
    ) {
        Ok(it) => Some(it),
        Err(_) => return None,
    };

    let main_view = Rc::new(RefCell::new(MainView::new(client.clone())));
    let torrent_view = Rc::new(RefCell::new(TorrentView::new(client.clone())));
    let live_view = Rc::new(RefCell::new(LiveView::new(client.clone())));

    application.connect_activate(move |app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .title("Bittorent APP")
            .default_width(350)
            .default_height(70)
            .build();

        win.connect_destroy(|_| exit(0));

        init_css(&win);

        let mut info_main = None;

        while info_main.is_none() {
            info_main = match client.borrow_mut().render.lock().unwrap().receive_ui() {
                Ok(it) => match it {
                    super::render::MessagesToUI::MainViewMsg(it) => Some(it),
                    _ => None,
                },
                Err(_) => None,
            };
        }

        {
            let parent = gtk::Box::new(Orientation::Horizontal, 2);
            parent.set_widget_name("app-parent");
            let left_pane_elements = init_left_pane(&parent);

            //Logica boton main
            let main_view_copy = main_view.clone();
            let parent_copy = parent.clone();
            left_pane_elements.main_bt.connect_clicked(move |_| {
                main_view_copy.borrow_mut().change_view(parent_copy.clone());
            });
            //Logica boton torrent
            let parent_copy = parent.clone();
            let torrent_view_copy = torrent_view.clone();
            left_pane_elements.torrent_bt.connect_clicked(move |_| {
                torrent_view_copy
                    .borrow_mut()
                    .change_view(parent_copy.clone());
            });

            //Inicializo vistas
            let mut cards = info_main.unwrap().cards;
            cards.sort_by(|a, b| a.title.cmp(&b.title));
            let main = main_view.borrow_mut().init_right_pane(cards);
            torrent_view.borrow_mut().init_right_pane(Card {
                title: String::new(),
                info: String::new(),
            });
            live_view.borrow_mut().init_right_pane(Card {
                title: String::new(),
                info: String::new(),
            });

            //Agrego logica flechas torrent view
            let torrent_view_copy = torrent_view.clone();
            torrent_view
                .borrow_mut()
                .rigth_arrow
                .connect_clicked(move |_| {
                    torrent_view_copy.borrow_mut().next_torrent("rigth");
                });
            let torrent_view_copy = torrent_view.clone();
            torrent_view
                .borrow_mut()
                .left_arrow
                .connect_clicked(move |_| {
                    torrent_view_copy.borrow_mut().next_torrent("left");
                });

            //Agrego logica en cartas main
            let cards = main_view.borrow_mut().cards_bt.clone();
            for i in 0..(cards.len()) {
                let torrent_view_copy = torrent_view.clone();
                let parent_copy = parent.clone();
                let main_view_copy = main_view.clone();
                main_view_copy.borrow_mut().cards_bt[i].connect_clicked(move |_| {
                    torrent_view_copy
                        .borrow_mut()
                        .change_view_to_torrent_pos(parent_copy.clone(), i);
                });
            }

            //Agrego logica en boton live view de cada torrent
            let live_view_copy = live_view.clone();
            let torrent_view_copy = torrent_view.clone();
            let parent_copy = parent.clone();
            torrent_view
                .borrow_mut()
                .live_view
                .connect_clicked(move |_| {
                    //torrent_view_copy.borrow_mut().next_torrent("left");
                    let index = torrent_view_copy.borrow_mut().get_index();
                    live_view_copy.borrow_mut().change_view(
                        parent_copy.clone(),
                        torrent_view_copy.clone().borrow_mut().torrents[index]
                            .title
                            .clone(),
                    );
                });

            //Agrego logica boton refresh en live view
            let torrent_view_copy = torrent_view.clone();
            let live_view_copy = live_view.clone();
            let parent_copy = parent.clone();
            live_view.borrow_mut().refresh_bt.connect_clicked(move |_| {
                let index = torrent_view_copy.borrow_mut().get_index();
                live_view_copy.borrow_mut().change_view(
                    parent_copy.clone(),
                    torrent_view_copy.clone().borrow_mut().torrents[index]
                        .title
                        .clone(),
                );
            });
            //Cargo primer Main
            parent.add(&main);

            win.add(&parent);
        }
        win.show_all();
    });
    let vec: Vec<String> = Vec::new();
    application.run_with_args(&vec);
    threads
}

fn init_left_pane(parent: &Box) -> LeftPaneElements {
    let left_pane = gtk::Box::new(Orientation::Vertical, 4);
    left_pane.set_widget_name("left_pane");

    {
        utils::add_vspace(&left_pane);
    }
    let left_pane_elements = LeftPaneElements {
        main_bt: {
            let bt = utils::create_button(12, 12, 0, 4, "Main View", None);
            left_pane.add(&bt);
            bt
        },
        torrent_bt: {
            let bt = utils::create_button(12, 12, 4, 4, "Torrent View", None);
            left_pane.add(&bt);
            bt
        },
    };
    {
        utils::add_vspace(&left_pane);
    }
    parent.add(&left_pane);
    left_pane_elements
}
