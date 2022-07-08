use std::cell::RefCell;
use std::rc::Rc;

use crate::client::client_handler::Client;
use crate::ui::render::{Card, MessagesToUI, RequestMessage};
use crate::ui::utils;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{prelude::*, Box};
use gtk::{Button, Image, Label};

pub struct TorrentView {
    client: Rc<RefCell<Client>>,
    torrent_container: Box,
    title: Box,
    text_box: Box,
    pub left_arrow: Button,
    pub rigth_arrow: Button,
    pub torrents: Vec<Card>,
    pub index: usize,
    pub live_view: Button,
}

impl TorrentView {
    pub fn init_right_pane(&self, card: Card) -> Box {
        {
            utils::add_hspace(&self.torrent_container)
        }
        {
            //title
            //horizontal
            let title_container = utils::create_hbox(
                Some("torrent-title-container"),
                6,
                true,
                None,
                Some(100),
                None,
            );
            {
                //arrow button
                let aux_b = utils::create_hbox(None, 3, false, None, None, None);
                let pixbuf =
                    Pixbuf::from_file_at_scale("src/ui/views/flecha_izq.png", 50, 50, true)
                        .unwrap();

                self.left_arrow
                    .set_image(Some(&Image::from_pixbuf(Some(&pixbuf))));
                self.left_arrow.set_margin_bottom(70);
                self.left_arrow.set_margin_top(70);
                self.left_arrow
                    .set_image(Some(&Image::from_pixbuf(Some(&pixbuf))));

                aux_b.set_margin_start(40);
                aux_b.add(&self.left_arrow);
                title_container.add(&aux_b);
            }
            {
                utils::add_hspace(&title_container);
            }
            {
                //title box
                //horizontal
                let title_box = utils::create_hbox(
                    Some("torrent-title-box"),
                    6,
                    false,
                    Some(700),
                    Some(200),
                    Some(50),
                );
                {
                    //space
                    utils::add_hspace(&title_box);
                }
                {
                    //title
                    let label = Label::new(Some(&card.title));
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
            {
                //arrow button
                let aux_b = utils::create_hbox(None, 3, false, None, None, None);
                let pixbuf =
                    Pixbuf::from_file_at_scale("src/ui/views/flecha_der.png", 50, 50, true)
                        .unwrap();

                self.rigth_arrow
                    .set_image(Some(&Image::from_pixbuf(Some(&pixbuf))));
                self.rigth_arrow.set_margin_bottom(70);
                self.rigth_arrow.set_margin_top(70);

                aux_b.set_margin_end(40);
                aux_b.add(&self.rigth_arrow);
                title_container.add(&aux_b);
            }
            self.torrent_container.add(&title_container);
        }
        {
            //Boton de live view
            let bt_live_box = utils::create_hbox(None, 2, false, None, None, Some(20));
            utils::add_hspace(&bt_live_box);
            self.live_view.set_label("Go to live view");
            bt_live_box.add(&self.live_view);
            bt_live_box.set_margin_end(40);
            self.torrent_container.add(&bt_live_box);
        }
        {
            utils::add_hspace(&self.torrent_container);
        }
        {
            let content_box =
                utils::create_hbox(Some("horizontal-content-box"), 3, false, None, None, None);
            {
                utils::add_hspace(&content_box);
            }
            {
                //content
                self.text_box.add(&Label::new(Some(&card.info)));
                content_box.add(&self.text_box);
            }
            {
                utils::add_hspace(&content_box);
            }
            self.torrent_container.add(&content_box);
        }
        {
            utils::add_hspace(&self.torrent_container);
        }
        {
            self.left_arrow.set_sensitive(false)
        }
        self.torrent_container.clone()
    }

    pub fn new(client: Rc<RefCell<Client>>) -> Self {
        TorrentView {
            client,
            torrent_container: utils::create_vbox(
                Some("torrent-container"),
                3,
                true,
                Some(1000),
                Some(700),
                Some(10),
            ),
            title: utils::create_vbox(Some("torrent-title-box"), 2, false, None, None, None),
            text_box: utils::create_hbox(
                Some("card_text_box"),
                1,
                false,
                Some(850),
                Some(400),
                None,
            ),
            rigth_arrow: Button::new(),
            left_arrow: Button::new(),
            torrents: Vec::new(),
            index: 0,
            live_view: Button::new(),
        }
    }

    pub fn change_view(&mut self, parent: Box) {
        self.index = 0;
        self.rigth_arrow.set_sensitive(true);
        self.left_arrow.set_sensitive(false);

        //lp.live_bt.set_sensitive(true);
        self.update(0);
        parent.remove(parent.children().get(1).unwrap());
        parent.add(&self.torrent_container);
        parent.show_all();
    }

    pub fn update(&mut self, index: usize) {
        //Pido vista main
        self.client
            .borrow_mut()
            .render
            .lock()
            .unwrap()
            .send_ui(RequestMessage::TorrentView)
            .unwrap();

        let mut info_torrent = None;
        while info_torrent.is_none() {
            info_torrent = match self.client.borrow_mut().render.lock().unwrap().receive_ui() {
                Ok(it) => match it {
                    MessagesToUI::TorrentViewMsg(it) => Some(it),
                    _ => None,
                },
                Err(_) => None,
            };
        }

        let mut cards = info_torrent.unwrap().cards;
        cards.sort_by(|a, b| a.title.cmp(&b.title));

        self.torrents = Vec::new();

        for c in cards {
            self.torrents.push(c);
        }

        for l in self.title.children() {
            self.title.remove(&l);
        }

        self.title
            .add(&Label::new(Some(&self.torrents[index].title)));
        self.title.show_all();

        for t in self.text_box.children() {
            self.text_box.remove(&t);
        }
        self.text_box
            .add(&Label::new(Some(&self.torrents[index].info)));
        self.text_box.show_all();
    }

    pub fn next_torrent(&mut self, direction: &str) {
        if direction == "rigth" {
            self.index += 1;
            if self.index >= (self.torrents.len() - 1) {
                self.rigth_arrow.set_sensitive(false);
                self.left_arrow.set_sensitive(true);
            }
            self.update_torrent_en_pos(self.index)
        } else if direction == "left" {
            self.index -= 1;
            if self.index == 0 {
                self.left_arrow.set_sensitive(false);
                self.rigth_arrow.set_sensitive(true);
            } else {
                self.left_arrow.set_sensitive(true);
            }
            self.update_torrent_en_pos(self.index)
        }
    }

    fn update_torrent_en_pos(&self, index: usize) {
        for l in self.title.children() {
            self.title.remove(&l);
        }
        self.title
            .add(&Label::new(Some(&self.torrents[index].title)));
        self.title.show_all();

        for t in self.text_box.children() {
            self.text_box.remove(&t);
        }
        self.text_box
            .add(&Label::new(Some(&self.torrents[index].info)));
        self.text_box.show_all();
    }

    pub fn change_view_to_torrent_pos(&mut self, parent: Box, index: usize) {
        self.index = index;

        self.update(self.index);

        if self.index >= (self.torrents.len() - 1) {
            self.rigth_arrow.set_sensitive(false);
            self.left_arrow.set_sensitive(true);
        } else if self.index == 0 {
            self.rigth_arrow.set_sensitive(true);
            self.left_arrow.set_sensitive(false);
        }
        parent.remove(parent.children().get(1).unwrap());
        parent.add(&self.torrent_container);
        parent.show_all();
    }

    pub fn get_index(&self) -> usize {
        self.index
    }
}
