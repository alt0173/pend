use eframe::{
  egui::{
    self, ComboBox, Context, Direction, DragValue, Layout, RichText, ScrollArea,
    TextEdit, TextStyle,
  },
  epaint::{vec2, Color32, FontId},
};
use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use glob::glob;
use serde::{Serialize, Deserialize};

use crate::{backend::parse_calibre, MyApp};

#[derive(Serialize, Deserialize)]
pub struct UIState {
  pub left_panel_state: PanelState,
  pub right_panel_state: PanelState,
  pub display_ofl_popup: bool,
  pub display_raw_text: bool,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum PanelState {
  Reader,
  Config,
  Library,
  Info,
  Notes,
}

#[derive(Serialize, Deserialize)]
pub struct BookTextStyle {
  pub font_id: FontId,
  pub font_color: Color32,
  pub bg_color: Color32,
  pub line_spacing: f32,
  pub text_layout: PageLayout,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PageLayout {
	RightToLeft,
	LeftToRight,
	Centered
}

pub fn main_ui(ctx: &Context, state: &mut MyApp) {
  egui::Area::new("Container").movable(false).show(ctx, |ui| {
    let area_width = ui.available_width();
    let mut left_panel_width = 0.0;

    egui::SidePanel::left("Left Panel")
      .resizable(true)
      .width_range(area_width / 3.0..=area_width / 1.5)
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.selectable_value(
            &mut state.ui_state.left_panel_state,
            PanelState::Library,
            "Library",
          );
          ui.selectable_value(
            &mut state.ui_state.left_panel_state,
            PanelState::Info,
            "Info",
          );
          ui.selectable_value(
            &mut state.ui_state.left_panel_state,
            PanelState::Notes,
            "Notes",
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
            ui.collapsing("Program", |ui| {
              // Path to directory containing books
              ui.horizontal(|ui| {
                ui.label("Library Path:");
                TextEdit::singleline(&mut state.library_path)
                  .hint_text(r"e.g. C:\Users\Public\Documents\Lisci")
                  .show(ui);
              });

              ui.checkbox(&mut state.remember_layout, "Restore Layout on Startup");
            });

            ui.collapsing("Book Content", |ui| {
              ComboBox::from_label("Text Alignment")
								.selected_text(format!("{:?}", state.book_style.text_layout))
                .show_ui(ui, |ui| {
                  ui.selectable_value(
                    &mut state.book_style.text_layout,
                    PageLayout::LeftToRight,
                    "Left \u{2192} Right",
                  );
                  ui.selectable_value(
                    &mut state.book_style.text_layout,
                    PageLayout::RightToLeft,
                    "Left \u{2190} Right",
                  );
                  ui.selectable_value(
                    &mut state.book_style.text_layout,
                    PageLayout::Centered,
                    "Centered",
                  );
                });
            });

            ui.collapsing("Other", |ui| {
              if ui.button("Acknowledgements").clicked() {
                state.ui_state.display_ofl_popup = true;
              }
              ui.checkbox(&mut state.ui_state.display_raw_text, "Display Raw Text");
            });
          }
          PanelState::Library => {
            ui.horizontal(|ui| {
              if ui.button("Load Library").clicked() {
                // Finds all epub files in the user's library directory
                for file_path in glob(&format!("{}/**/*.epub", state.library_path))
                  .unwrap()
                  .flatten()
                {
                  // Add file to library if not already added
                  if !state.library.contains(&file_path) {
                    state.library.push(file_path.clone());
										println!("{:?}", &file_path);
                  }
                  // Same thing for the book cover
                  let mut doc = EpubDoc::new(file_path).unwrap();
                  let title = doc.mdata("title").unwrap();

                  if doc.get_cover().is_ok() {
                    let cover = doc.get_cover().unwrap();
                    let cover =
                      RetainedImage::from_image_bytes(&title, &cover).unwrap();

                    state.book_covers.insert(title, cover);
                  }
                }
              }
              if ui.button("Clear Library").clicked() {
                state.library.clear();
                state.book_covers.clear();
                state.selected_book = None;
								state.selected_book_path = None;
              }
            });
            ui.separator();

            egui::Grid::new("Library Shelf Thing")
              .striped(true)
              .show(ui, |ui| {
                let h = ui.available_height() / 6.0;

                for book in state.library.iter() {
                  if let Some(title) = EpubDoc::new(book).unwrap().mdata("title") {
                    ui.vertical_centered(|ui| {
                      if ui
                        .add(egui::ImageButton::new(
                          state.book_covers.get(&title).unwrap().texture_id(ctx),
                          vec2(h * 50.0, h * 80.0),
                        ))
                        .clicked()
                      {
                        state.selected_book = Some(EpubDoc::new(book).unwrap());
												state.selected_book_path = Some(book.clone());
                      }
                      ui.label(RichText::new(title).text_style(TextStyle::Body));
                    });
                  }
                }
              });
          }
          PanelState::Info => {
            ui.label("Info");
          }
          PanelState::Notes => {
            ui.label("Notes");
          }
          PanelState::Reader => {
            panic!("This shouldn't happen");
          }
        }

        left_panel_width = ui.min_rect().width();
      });

    egui::CentralPanel::default().show(ctx, |ui| {
      if state.ui_state.right_panel_state == PanelState::Reader {
        // Displays page(s) of the book
        if let Some(book) = &mut state.selected_book {
          ui.horizontal(|ui| {
            // Back page (CHAPTER) button
            if ui.button("\u{2190}").clicked() && book.get_current_page() > 0 {
              state.chapter_number -= 1;
            }
            // Page (CHAPTER) navigation thing
            ui.add(
              DragValue::new(&mut state.chapter_number)
                .max_decimals(0)
                .clamp_range(0..=book.get_num_pages() - 1),
            );
						// Forward page (CHAPTER) button
						if ui.button("\u{2192}").clicked() && book.get_current_page() < book.get_num_pages() - 1 {
							state.chapter_number += 1;
						}
						// Apply page / chapter change of needed
						if book.get_current_page() != state.chapter_number {
							book.set_current_page(state.chapter_number).unwrap()
						}
					});

          ui.separator();

          // Display of page (CHAPTER) contents
          ScrollArea::new([false, true])
            .always_show_scroll(false)
            .auto_shrink([false, true])
            .show(ui, |ui| {
              if state.ui_state.display_raw_text {
                ui.label(&book.get_current_str().unwrap());
              } else {
                // Layout (rtl, ltr, etc.)
                // ui.with_layout(Layout::with_main_wrap(Layout::left_to_right(), true), |ui| {
                ui.horizontal_wrapped(|ui| {
                  let style = &state.book_style;

                  // Background
                  ui.painter()
                    .rect_filled(ui.clip_rect(), 0.0, style.bg_color);

                  // Contents
                  let text =
                    RichText::new(&parse_calibre(&book.get_current_str().unwrap()))
                      .color(style.font_color)
                      .font(style.font_id.clone());

                  ui.label(text);
                });
              }
            });
        } else {
          ui.label("No book loaded");
        }
      } else {
        todo!()
      }
    });
  });

  // Popups, etc.
  if state.ui_state.display_ofl_popup {
    egui::Window::new("Acknowledgement Popup")
			.title_bar(false)
			.show(ctx, |ui| {
				if ui.button("Close Menu").clicked() {
					state.ui_state.display_ofl_popup = false;
				}

				ui.vertical_centered(|ui| {
					ui.heading("FONTS");
				});
				ui.label("In an effort to make this program more portable and accessible, the font files used have been included in the binary.");
				ui.collapsing("Work Sans", |ui| {
					ui.label("Work Sans is a font licensed under version 1.1 of the OFL");
					ui.hyperlink("https://scripts.sil.org/OFL");
					ui.hyperlink("https://github.com/weiweihuanghuang/Work-Sans");
				});
			});
  }
}
