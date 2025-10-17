use eframe::epaint::Color32;
use serde::{Deserialize, Serialize};

pub struct Theme {
    pub main_background: Color32,
    pub main_foreground: Color32,
    pub top_bar_background: Color32,
    pub top_bar_text: Color32,
    pub default_node_color: Color32,
    pub node_text_color: Color32,
    pub default_edge_color: Color32,
    pub right_click_menu_background: Color32,
    pub right_click_menu_item: Color32,
    pub right_click_menu_item_text: Color32,
    pub annotation_title: Color32,
    pub annotation_body: Color32,
    pub annotation_date: Color32,
    pub popup_form_background: Color32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializeableTheme {
    main_background: [u8;4],
    main_foreground: [u8;4],
    top_bar_background: [u8;4],
    top_bar_text: [u8;4],
    default_node_color: [u8;4],
    node_text_color: [u8;4],
    default_edge_color: [u8;4],
    right_click_menu_background: [u8;4],
    right_click_menu_item: [u8;4],
    right_click_menu_item_text: [u8;4],
    annotation_title: [u8;4],
    annotation_body: [u8;4],
    annotation_date: [u8;4],
    popup_form_background: [u8;4],
}

impl Theme{
    pub fn default() -> Theme {
        Theme {
            main_background: Color32::from_gray(27),
            main_foreground: Color32::from_gray(140),
            top_bar_background: Color32::from_gray(27),
            top_bar_text: Color32::from_gray(140),
            default_node_color: Color32::LIGHT_BLUE,
            node_text_color: Color32::BLACK,
            default_edge_color: Color32::GRAY,
            right_click_menu_background: Color32::from_gray(27),
            right_click_menu_item: Color32::from_gray(60),
            right_click_menu_item_text: Color32::from_gray(140),
            annotation_title: Color32::WHITE,
            annotation_body: Color32::LIGHT_GRAY,
            annotation_date: Color32::DARK_GRAY,
            popup_form_background: Color32::from_gray(10),
        }
    }
    pub(crate) fn from_serializeable(input: SerializeableTheme) -> Theme {
        Theme {
            main_background: to_egui_color(input.main_background),
            main_foreground: to_egui_color(input.main_foreground),
            top_bar_background: to_egui_color(input.top_bar_background),
            top_bar_text: to_egui_color(input.top_bar_text),
            default_node_color: to_egui_color(input.default_node_color),
            node_text_color: to_egui_color(input.node_text_color),
            default_edge_color: to_egui_color(input.default_edge_color),
            right_click_menu_background: to_egui_color(input.right_click_menu_background),
            right_click_menu_item: to_egui_color(input.right_click_menu_item),
            right_click_menu_item_text: to_egui_color(input.right_click_menu_item_text),
            annotation_title: to_egui_color(input.annotation_title),
            annotation_body: to_egui_color(input.annotation_body),
            annotation_date: to_egui_color(input.annotation_date),
            popup_form_background: to_egui_color(input.popup_form_background),
        }
    }
}

fn to_egui_color(input: [u8;4]) -> Color32 {
    Color32::from_rgba_unmultiplied(input[0], input[1], input[2], input[3])
}