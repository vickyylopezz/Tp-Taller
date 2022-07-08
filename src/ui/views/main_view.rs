use std::cell::RefCell;
use std::rc::Rc;

use crate::client::client_handler::Client;
use crate::ui::render::{Card, MessagesToUI, RequestMessage};
use crate::ui::utils;
use gtk::{prelude::*, Box, Button};
use gtk::{Image, Label, PolicyType, ScrolledWindow};

pub struct MainView {
    client: Rc<RefCell<Client>>,
    pub cards_bt: Vec<Button>,
    general_pane: Box,
    cards: Vec<Card>,
}

impl MainView {
    pub fn init_right_pane(&mut self, cards: Vec<Card>) -> Box {
        self.cards = cards;
        {
            utils::add_hspace(&self.general_pane)
        }
        {
            let right_pane = utils::create_vbox(Some("right_pane"), 15, true, None, None, None);
            //vertical
            {
                utils::add_hspace(&right_pane)
            }
            {
                //title
                let title_box = utils::create_hbox(Some("title_box"), 2, false, None, None, None);

                //horizontal
                {
                    utils::add_hspace(&title_box);
                }
                {
                    let title = utils::create_vbox(None, 2, false, None, None, None);
                    let label = Label::new(Some("General Information"));
                    title.add(&label);
                    title_box.add(&title);
                }
                {
                    utils::add_hspace(&title_box);
                }
                right_pane.add(&title_box);
            }
            {
                utils::add_hspace(&right_pane)
            }
            {
                let list_box = utils::create_hbox(None, 1, false, None, None, None);
                self.cards.iter().for_each(|card| {
                    let aux = utils::create_vbox(None, 1, false, None, None, None);
                    let bt = Button::new();
                    bt.set_border_width(50);
                    bt.set_width_request(500);
                    bt.set_widget_name("torrent_card");
                    bt.add(&create_torrent_card(card));
                    aux.add(&bt);
                    self.cards_bt.push(bt);
                    list_box.add(&aux);
                });

                let scrolled_window = ScrolledWindow::builder()
                    .hscrollbar_policy(PolicyType::Always) // Disable horizontal scrolling
                    .vscrollbar_policy(PolicyType::Never)
                    .min_content_width(1000)
                    .min_content_height(500)
                    .child(&list_box)
                    .build();

                scrolled_window.set_widget_name("scroll_window");
                right_pane.add(&scrolled_window);
            }
            {
                utils::add_hspace(&right_pane)
            }
            self.general_pane.add(&right_pane);
        }
        {
            utils::add_hspace(&self.general_pane)
        }

        self.general_pane.clone()
    }

    pub fn new(client: Rc<RefCell<Client>>) -> Self {
        MainView {
            client,
            general_pane: utils::create_hbox(None, 2, true, Some(1000), Some(700), Some(30)),
            cards: Vec::new(),
            cards_bt: Vec::new(),
        }
    }

    pub fn change_view(&mut self, parent: Box) {
        self.update();
        parent.remove(parent.children().get(1).unwrap());
        parent.add(&self.general_pane);
        parent.show_all();
    }

    pub fn update(&mut self) {
        //Pido vista main
        self.client
            .borrow_mut()
            .render
            .lock()
            .unwrap()
            .send_ui(RequestMessage::MainView)
            .unwrap();

        let mut info_main = None;
        while info_main.is_none() {
            info_main = match self.client.borrow_mut().render.lock().unwrap().receive_ui() {
                Ok(it) => match it {
                    MessagesToUI::MainViewMsg(it) => Some(it),
                    _ => None,
                },
                Err(_) => None,
            };
        }

        for b in &self.cards_bt {
            b.remove(b.children().get(0).unwrap())
        }

        self.cards = info_main.unwrap().cards;
        self.cards.sort_by(|a, b| a.title.cmp(&b.title));

        for i in 0..(self.cards.len()) {
            self.cards_bt[i].add(&create_torrent_card(&self.cards[i]));
        }
    }
}

fn create_torrent_card(card: &Card) -> Box {
    let torrent_card = utils::create_vbox(Some("torrent_card"), 2, false, None, None, None);
    {
        utils::add_hspace(&torrent_card);
    }
    {
        //card title
        let card_title = utils::create_hbox(None, 1, false, None, None, None);
        //horizontal
        {
            let image_container = utils::create_hbox(None, 1, true, None, None, None);

            let img = Image::from_file("src/Captura de Pantalla 2022-06-18 a la(s) 16.45.09.png");
            image_container.add(&img);
            card_title.add(&image_container);
        }
        {
            utils::add_hspace(&card_title);
        }
        {
            let title = utils::create_hbox(Some("title-box"), 3, false, None, Some(100), None);
            {
                utils::add_vspace(&title);
            }
            {
                let aux_box =
                    utils::create_hbox(Some("card_title"), 1, false, Some(300), Some(100), None);
                let label = Label::new(Some(&card.title));
                aux_box.add(&label);
                title.add(&aux_box);
            }
            {
                utils::add_vspace(&title);
            }
            card_title.add(&title);
        }
        {
            utils::add_hspace(&card_title);
        }
        torrent_card.add(&card_title);
    }
    {
        utils::add_hspace(&torrent_card);
    }
    {
        let text_box = utils::create_hbox(Some("card_text_box"), 1, false, None, Some(400), None);
        text_box.add(&Label::new(Some(&card.info)));
        torrent_card.add(&text_box);
    }
    {
        utils::add_hspace(&torrent_card);
    }
    torrent_card
}
