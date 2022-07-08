use std::cell::RefCell;
use std::rc::Rc;

use crate::client::client_handler::Client;
use crate::ui::render::{Card, MessagesToUI, RequestMessage, TorrentId};
use crate::ui::utils;
use chrono::Local;
use gtk::{prelude::*, Box, PolicyType, ScrolledWindow};
use gtk::{Button, Label};

pub struct LiveView {
    client: Rc<RefCell<Client>>,
    view_container: Box,
    title: Box,
    text_box: Box,
    pub refresh_bt: Button,
    pub index: usize, //ID DEL TORRENT, CAMBIAR
    refresh_date: Box,
    pub card: Card,
}

impl LiveView {
    pub fn init_right_pane(&mut self, card: Card) -> Box {
        self.card = card;
        {
            utils::add_hspace(&self.view_container)
        }
        {
            //title
            //horizontal
            let title_container =
                utils::create_hbox(Some("torrent-title-container"), 5, false, None, None, None);
            {
                utils::add_hspace(&title_container);
            }
            {
                //title box
                //horizontal
                let title_box =
                    utils::create_hbox(Some("live-title-box"), 0, false, Some(700), None, Some(10));
                {
                    //space
                    utils::add_hspace(&title_box);
                }
                {
                    //title
                    let label = Label::new(Some(&self.card.title));
                    self.title.add(&label);
                    title_box.add(&self.title);
                }
                {
                    utils::add_hspace(&title_box);
                }
                title_container.add(&title_box);
            }
            {
                utils::add_hspace(&title_container);
            }

            self.view_container.add(&title_container);
        }
        {
            utils::add_hspace(&self.view_container);
        }
        {
            // //Boton de refresh
            {
                //     let refresh_box = utils::create_hbox(None, 2, false, None, None, None);
                //     let refresh_bt = utils::create_button(
                //         12,
                //         12,
                //         12,
                //         12,
                //         "Refresh",
                //         Some("Refresh the page to see the last live data"),
                //     );
                //     refresh_box.add(&refresh_bt);
                //     self.view_container.add(&refresh_box);
                let bt_live_box = utils::create_hbox(None, 2, false, None, None, None);
                utils::add_hspace(&bt_live_box);
                self.refresh_bt.set_label("Refresh");
                bt_live_box.add(&self.refresh_bt);
                bt_live_box.set_margin_end(60);
                self.view_container.add(&bt_live_box);
            }

            {
                utils::add_hspace(&self.refresh_date);
                let time = Label::new(Some(
                    &("Last refresh:\n".to_owned()
                        + &Local::now().format("%d-%m-%Y %H:%M:%S").to_string()),
                ));
                self.refresh_date.add(&time);
                time.set_margin_end(40);
                self.view_container.add(&self.refresh_date);
            }
        }
        {
            let content_box =
                utils::create_hbox(Some("horizontal-content-box"), 3, false, None, None, None);
            {
                utils::add_hspace(&content_box);
            }
            {
                //content
                self.text_box.add(&Label::new(Some(&self.card.info)));
                let scrolled_window = ScrolledWindow::builder()
                    .hscrollbar_policy(PolicyType::Never) // Disable horizontal scrolling
                    .vscrollbar_policy(PolicyType::Always)
                    .min_content_width(900)
                    .min_content_height(500)
                    .child(&self.text_box)
                    .build();
                content_box.add(&scrolled_window);
            }
            {
                utils::add_hspace(&content_box);
            }
            self.view_container.add(&content_box);
        }
        {
            utils::add_hspace(&self.view_container);
        }
        {
            //utils::add_connection_refresh_bt(parent, lp, Panes::LivePane, vec![card]);
        }
        self.view_container.clone()
    }

    pub fn new(client: Rc<RefCell<Client>>) -> Self {
        LiveView {
            client,
            view_container: utils::create_vbox(
                Some("torrent-container"),
                10,
                true,
                Some(1000),
                Some(700),
                Some(20),
            ),
            title: utils::create_vbox(Some("live-title-box"), 2, false, None, None, None),
            text_box: utils::create_hbox(Some("card_text_box"), 1, false, Some(900), None, None),
            refresh_bt: Button::new(),
            index: 0,
            refresh_date: utils::create_hbox(None, 2, false, None, None, None),
            card: Card {
                title: String::new(),
                info: String::new(),
            },
        }
    }

    //El index seria el id con el que solicitamos el torrent
    pub fn change_view(&mut self, parent: Box, torrent_name: String) {
        //lp.live_bt.set_sensitive(true);
        self.update(torrent_name);
        parent.remove(parent.children().get(1).unwrap());
        parent.add(&self.view_container);

        self.refresh_date
            .remove(self.refresh_date.children().get(1).unwrap());
        let time = Label::new(Some(
            &("Last refresh:\n".to_owned() + &Local::now().format("%d-%m-%Y %H:%M:%S").to_string()),
        ));
        time.set_margin_start(10);
        time.set_margin_end(10);
        self.refresh_date.add(&time);
        self.refresh_date.show_all();

        parent.show_all();
    }

    pub fn update(&mut self, index: String) {
        //Pido vista main
        self.client
            .borrow_mut()
            .render
            .lock()
            .unwrap()
            .send_ui(RequestMessage::LiveView(TorrentId(index)))
            .unwrap();

        let mut info_live = None;
        while info_live.is_none() {
            info_live = match self.client.borrow_mut().render.lock().unwrap().receive_ui() {
                Ok(it) => match it {
                    MessagesToUI::LiveViewMsg(it) => Some(it),
                    _ => None,
                },
                Err(_) => None,
            };
        }

        for l in self.title.children() {
            self.title.remove(&l);
        }

        let label = Label::new(Some(&info_live.clone().unwrap().card.title));
        self.title.add(&label);
        self.title.show_all();

        for t in self.text_box.children() {
            self.text_box.remove(&t);
        }
        self.text_box
            .add(&Label::new(Some(&info_live.clone().unwrap().card.info)));
        self.text_box.show_all();

        self.card = info_live.unwrap().card;
    }
}
