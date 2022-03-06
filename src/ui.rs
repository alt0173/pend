use std::cmp::Ordering;

use eframe::{
  egui::{self, Context, RichText, ScrollArea},
  epaint::{vec2, Color32},
};
use egui::FontFamily;
use serde::{Deserialize, Serialize};

use crate::{
  panels::{config::config_ui, notes::notes_ui, shelf::shelf_ui},
  reader::right_panel_reader_ui,
  MyApp,
};

#[derive(Serialize, Deserialize)]
pub struct UIState {
  pub left_panel_state: PanelState,
  pub right_panel_state: PanelState,
  pub reader_focus_mode: bool,
  pub display_ofl_popup: bool,
  pub display_raw_text: bool,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum PanelState {
  Reader,
  Config,
  Shelf,
  Info,
  Notes,
}

#[derive(Serialize, Deserialize)]
pub struct BookTextStyle {
  pub font_size: f32,
  pub font_family: FontFamily,
  pub line_spacing_multiplier: f32,
}

impl Default for BookTextStyle {
  fn default() -> Self {
    Self {
      font_size: 22.0,
      font_family: FontFamily::Name("Merriweather".into()),
      line_spacing_multiplier: 1.0,
    }
  }
}

// The PartiqlOrd derive may lead to issues?
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialOrd)]
pub struct Note {
  pub chapter: u16,
  pub line: u16,
  pub content: String,
}

// Slightly modified partialeq that disregards content
impl PartialEq for Note {
  fn eq(&self, other: &Self) -> bool {
    self.chapter == other.chapter && self.line == other.line
  }
}

impl Ord for Note {
  fn cmp(&self, other: &Self) -> Ordering {
    match self.chapter.cmp(&other.chapter) {
      Ordering::Greater => Ordering::Greater,
      Ordering::Less => Ordering::Less,
      Ordering::Equal => match self.line.cmp(&other.line) {
        Ordering::Greater => Ordering::Greater,
        Ordering::Less => Ordering::Less,
        Ordering::Equal => Ordering::Equal,
      },
    }
  }
}

impl Note {
  pub fn new(chapter: u16, line: u16) -> Self {
    Self {
      chapter,
      line,
      content: String::new(),
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct DocumentColors {
  pub highlight_color: Color32,
  pub text_color: Color32,
  pub page_color: Color32,
}

impl Default for DocumentColors {
  fn default() -> Self {
    Self {
      highlight_color: Color32::YELLOW,
      text_color: Color32::BLACK,
      page_color: Color32::from_rgb(239, 229, 213),
    }
  }
}

pub fn main_ui(ctx: &Context, state: &mut MyApp) {
  egui::Area::new("Container").movable(false).show(ctx, |ui| {
    let area_width = ui.available_width();
		let area_height = ui.available_height();
    let mut left_panel_width = 0.0;

		// Popups
  	if state.ui_state.display_ofl_popup {
    egui::Window::new("Acknowledgements")
			.title_bar(false)
			.resizable(false)
			.fixed_size(vec2(area_height / 2.0, area_height / 2.0))
			.show(ctx, |ui| {

				if ui.button("Close Menu").clicked() {
					state.ui_state.display_ofl_popup = false;
				}

				ui.vertical_centered(|ui| {
					ui.heading("FONTS");
				});
				ui.label("In an effort to make this program more portable and accessible, the font files used have been included in the binary.");
				ui.label("A copy of the Open Font License (OFL) is available at the bottom of this menu.");
				ui.collapsing("Work Sans", |ui| {
					ui.label("Work Sans is a font licensed under version 1.1 of the OFL");
					ui.hyperlink("https://github.com/weiweihuanghuang/Work-Sans");
				});
				ui.collapsing("Merriweather", |ui| {
					ui.label("Merriweather is a font licensed under version 1.1 of the OFL");
					ui.hyperlink("https://github.com/SorkinType/Merriweather");
				});

				ui.separator();

				ui.collapsing("Open Font License Version 1.1", |ui| {
					ScrollArea::new([true; 2])
						.max_height(area_height / 2.0)
						.max_width(area_height / 2.0)
						.show(ui, |ui| {
							ui.horizontal_wrapped(|ui| {
								ui.label(
									RichText::new(String::from_utf8(include_bytes!("../compiletime_resources/OFL_1.1.txt")	.to_vec()).expect("Failed to locate OFL v1.1")).monospace()
								);
							});
						});
				})
			});
  }

		// Panels
		if !state.ui_state.reader_focus_mode || state.selected_book.is_none() {
			egui::SidePanel::left("Left Panel")
				.resizable(true)
				.width_range(area_width / 3.0..=area_width / 1.5)
				.show(ctx, |ui| {
					ui.horizontal(|ui| {
						ui.selectable_value(
							&mut state.ui_state.left_panel_state,
							PanelState::Shelf,
							"Shelf",
						);
						ui.selectable_value(
							&mut state.ui_state.left_panel_state,
							PanelState::Notes,
							"Notes",
						);
						ui.selectable_value(
							&mut state.ui_state.left_panel_state,
							PanelState::Info,
							"Info",
						);

						ui.with_layout(egui::Layout::right_to_left(), |ui| {
							ui.selectable_value(
								&mut state.ui_state.left_panel_state,
								PanelState::Config,
								"Config",
							);
						});
					});
					ui.separator();

					match state.ui_state.left_panel_state {
						PanelState::Config => {
							config_ui(state, ui);
						}
						PanelState::Shelf => {
							shelf_ui(state, ui);
						}
						PanelState::Notes => {
							notes_ui(state, ui);
						}
						PanelState::Info => {
							ui.label("TODO :0");
						}
						_ => {
							panic!("Error: Invalid Panel Selected");
						}
					}

					left_panel_width = ui.min_rect().width();
				});
		}

    egui::CentralPanel::default().show(ctx, |ui| {
      if state.ui_state.right_panel_state == PanelState::Reader {
				right_panel_reader_ui(state, ui);
      } else {
        todo!()
      }
    });
  });
}

// Other stuff
