use gtk::Orientation;
use gtk::{prelude::*, Box, Button};

pub fn add_hspace(container: &Box) {
    let space = Box::new(Orientation::Horizontal, 1);
    space.set_hexpand(true);
    container.add(&space);
}

pub fn add_vspace(container: &Box) {
    let space = Box::new(Orientation::Vertical, 1);
    space.set_vexpand(true);
    container.add(&space);
}
pub fn create_hbox(
    id: Option<&str>,
    spacing: i32,
    expand: bool,
    width: Option<i32>,
    height: Option<i32>,
    border_w: Option<u32>,
) -> Box {
    let bx = Box::new(Orientation::Horizontal, spacing);
    bx.set_hexpand(expand);
    if let Some(id) = id {
        bx.set_widget_name(id);
    }
    if let Some(width) = width {
        bx.set_width_request(width);
    }
    if let Some(height) = height {
        bx.set_height_request(height);
    }
    if let Some(border_w) = border_w {
        bx.set_border_width(border_w);
    }
    bx
}

pub fn create_vbox(
    id: Option<&str>,
    spacing: i32,
    expand: bool,
    width: Option<i32>,
    height: Option<i32>,
    border_w: Option<u32>,
) -> Box {
    let bx = Box::new(Orientation::Vertical, spacing);
    bx.set_vexpand(expand);
    if let Some(id) = id {
        bx.set_widget_name(id);
    }
    if let Some(width) = width {
        bx.set_width_request(width);
    }
    if let Some(height) = height {
        bx.set_height_request(height);
    }
    if let Some(border_w) = border_w {
        bx.set_border_width(border_w);
    }

    bx
}

pub fn create_button(
    pad_left: i32,
    pad_right: i32,
    pad_top: i32,
    pad_bottom: i32,
    label: &str,
    tool_tip: Option<&str>,
) -> Button {
    let bt = Button::builder()
        .label(label)
        .margin_top(pad_top)
        .margin_bottom(pad_bottom)
        .margin_start(pad_right)
        .margin_end(pad_left)
        .build();
    if let Some(tool_tip) = tool_tip {
        bt.set_tooltip_text(Some(tool_tip));
    }
    bt
}
