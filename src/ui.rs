use std::{cmp::Ordering, sync::Arc};

use eframe::{
  egui::{
    self, ComboBox, Context, DragValue, RichText, ScrollArea, TextEdit,
    TextStyle,
  },
  epaint::{vec2, Color32, FontId},
};
use egui::{FontFamily, Label, Sense};
use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};

use crate::{
  backend::{load_library, parse_calibre, FormattingInfo},
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
  Library,
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
  chapter: u16,
  line: u16,
  content: String,
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
							PanelState::Library,
							"Library",
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
							ui.collapsing("Program", |ui| {
								// Path to directory containing books
								ui.horizontal(|ui| {
									ui.label("Library Path:");
									TextEdit::singleline(&mut state.library_path)
										.hint_text(r"e.g. C:\Users\Public\Documents\Lisci")
										.show(ui);
								});

								ui.checkbox(&mut state.ui_state.reader_focus_mode, "Focus Mode");
							});

							ui.collapsing("Document", |ui| {
								ComboBox::from_label("Font")
								.selected_text(match &state.book_style.font_family {
									f if f == &FontFamily::Proportional => "Work Sans",
									f if f == &FontFamily::Name(Arc::from("Merriweather"))  => "Merriweather",
									_ => "Unrecognized Font"
								})
								.show_ui(ui, |ui| {
									ui.selectable_value(&mut state.book_style.font_family, FontFamily::Proportional, "Work Sans");
									ui.selectable_value(&mut state.book_style.font_family, FontFamily::Name("Merriweather".into()), "Merriweather");
								});

								ui.add(
									egui::Slider::new(&mut state.book_style.font_size, 12.0..=120.0)
										.step_by(0.25)
										.prefix("Text Size: ")
								);

								ui.add(
									egui::Slider::new(&mut state.book_style.line_spacing_multiplier, 0.0..=6.0)
										.step_by(0.25)
										.prefix("Line Spacing: ")
								);

								ui.collapsing("Colors", |ui| {
									ui.horizontal(|ui| {
										ui.color_edit_button_srgba(&mut state.theme.highlight_color);
										ui.label(": Highlight Color");
									});
									ui.horizontal(|ui| {
										ui.color_edit_button_srgba(&mut state.theme.text_color);
										ui.label(": Text Color");
									});
									ui.horizontal(|ui| {
										ui.color_edit_button_srgba(&mut state.theme.page_color);
										ui.label(": Page Color");
									});

									ui.separator();

									if ui.button("Reset Colors").clicked() {
										state.theme = DocumentColors::default()
									}
								});

								ui.separator();

								ui.horizontal(|ui| {
									if ui.button("Reset Style").clicked() {
										state.book_style = BookTextStyle::default();
									}
									if ui.button("Clear Highlights").clicked() {
										if let Some(path) = &state.selected_book_path {
											state.book_userdata.get_mut(path).unwrap().highlights.clear();
										}
									}
								});
							});

							ui.collapsing("Other", |ui| {
								if ui.button("Acknowledgements").clicked() {
									state.ui_state.display_ofl_popup = true;
								}
								// ui.checkbox(&mut state.ui_state.display_raw_text, "[DEBUG] Display Raw Text");
							});
						}
						PanelState::Library => {
							ui.horizontal(|ui| {
								if ui.button("Load Library").clicked() {
									load_library(state);
								}
								if ui.button("Clear Library").clicked() {
									state.library.clear();
									state.book_covers.clear();
									state.selected_book = None;
									state.selected_book_path = None;
								}

								ui.separator();

								TextEdit::singleline(&mut state.library_search).hint_text("Search Library...").show(ui);
							});
							ui.separator();

							egui::Grid::new("Library Shelf Thing")
								.striped(true)
								.show(ui, |ui| {
									let y = ui.available_height() / 6.0;

									for book in state.library.iter() {
										if let Ok(doc) = EpubDoc::new(book) {
											let title = doc.mdata("title").unwrap();
											let author = doc.mdata("creator").unwrap();

											// Search application
											if title.contains(&state.library_search) || author.contains(&state.library_search) {
												// Display the cover & info
												ui.vertical_centered(|ui| {
													if ui
														.add(egui::ImageButton::new(
															state.book_covers.get(&title).unwrap().texture_id(ctx),
															vec2(y * 50.0, y * 80.0),
														))
														.clicked()
														&& state.selected_book_path != Some(book.to_path_buf())
													{
														state.selected_book = Some(EpubDoc::new(book).unwrap());
														state.selected_book_path = Some(book.clone());
														state.chapter_number = 1;
													}
													ui.label(RichText::new(title).text_style(TextStyle::Body));
													if let Some(author) = doc.mdata("creator") {
														ui.label(RichText::new(author).text_style(TextStyle::Body));
													}
												});
											}
										}
									}
								});
						}
						PanelState::Notes => {
							if let Some(path) = &state.selected_book_path {
								if let Some(book_info) = state.book_userdata.get_mut(path) {
									let notes = &mut book_info.notes;

									ui.horizontal(|ui| {
										if ui.button("Sort Notes").clicked() {
											notes.sort();
										}
										if ui.button("Clear Notes").clicked() {
											notes.clear();
										}
									});
									ui.separator();

									// Can't have mutable borrow && a mutable iter so a helper is needed
									let mut to_delete = None;
									for (index, note) in notes.iter_mut().enumerate() {
										let (chapter, line, content) = (note.chapter, note.line, &mut note.content);

										ui.horizontal(|ui| {
											let response = ui.collapsing(format!("Ch. {}, line: {}", chapter, line), |ui| {
												TextEdit::multiline(content)
													.show(ui);
											});

											if response.body_response.is_none() {
												if ui.button("Go to").clicked() {
													state.goto_target = Some(Note::new(chapter, line));
												}
												if ui.button("Remove Note").clicked() {
													to_delete = Some(index);
												}
											}
										});
									}

									if let Some(index) = to_delete {
										notes.remove(index);
									}
								} else {
									ui.label("No notes detected.");
								}
							} else {
								ui.label("No Book Selected");
							}
						}
						PanelState::Info => {
							ui.label("Info");
						}
						PanelState::Reader => {
							panic!("This shouldn't happen");
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

fn right_panel_reader_ui(state: &mut MyApp, ui: &mut egui::Ui) {
  // Displays page(s) of the book
  if let Some(book) = &mut state.selected_book {
    if let Some(target) = &state.goto_target {
      state.chapter_number = target.chapter as usize;
    }

    // Arrow key navigation
    if ui.ctx().input().key_pressed(egui::Key::ArrowLeft)
      && book.get_current_page() > 1
    {
      state.chapter_number -= 1;
    }
    if ui.ctx().input().key_pressed(egui::Key::ArrowRight)
      && book.get_current_page() < book.get_num_pages() - 1
    {
      state.chapter_number += 1;
    }

    // Button navigation
    ui.horizontal(|ui| {
      if !state.ui_state.reader_focus_mode {
        // Back page (CHAPTER) button
        if ui.button("\u{2190}").clicked() && book.get_current_page() > 1 {
          state.chapter_number -= 1;
        }
        // Page (CHAPTER) navigation thing
        ui.add(
          DragValue::new(&mut state.chapter_number)
            .max_decimals(0)
            .clamp_range(1..=book.get_num_pages() - 1),
        );
        // Forward page (CHAPTER) button
        if ui.button("\u{2192}").clicked()
          && book.get_current_page() < book.get_num_pages() - 1
        {
          state.chapter_number += 1;
        }
      } else {
        ui.horizontal(|ui| {
          ui.with_layout(egui::Layout::right_to_left(), |ui| {
            if ui.button("Exit Focus").clicked() {
              state.ui_state.reader_focus_mode = false;
            }
          });
        });
      }
    });

    // Apply page / chapter change of needed
    if book.get_current_page() != state.chapter_number {
      book.set_current_page(state.chapter_number).unwrap();
      state.goto_target = Some(Note::new(state.chapter_number as u16, 0));
    }

    ui.separator();

    // Display of page (CHAPTER) contents
    ScrollArea::new([false, true])
      .always_show_scroll(false)
      .auto_shrink([false, true])
      .show(ui, |ui| {
        if state.ui_state.display_raw_text {
          ui.label(&book.get_current_str().unwrap());
        } else {
          let style = &state.book_style;
          let theme = &state.theme;
          let contents = parse_calibre(
            &book.get_current_str().unwrap(),
            book.get_current_page(),
            &mut state
              .book_userdata
              .get_mut(state.selected_book_path.as_ref().unwrap())
              .unwrap(),
          );
          let contents: Vec<&str> = contents.lines().collect();

          // Background
          ui.painter()
            .rect_filled(ui.clip_rect(), 0.0, theme.page_color);

          // Actual "stuff"
          let font_id = FontId::new(style.font_size, style.font_family.clone());
          let line_spacing =
            ui.fonts().row_height(&font_id) * style.line_spacing_multiplier;

          ui.style_mut().spacing.item_spacing.y = line_spacing;

          let mut goto_target_response = None;

          for (line_number, line) in contents.into_iter().enumerate() {
            let response = ui.add(
              Label::new({
                // Creates text with normal / default appearence
                // This is how normal body text looks
                let mut text = RichText::new(line)
                  .color(theme.text_color)
                  .background_color(
                    if let Some(color) = state
                      .book_userdata
                      .get(&state.selected_book_path.as_ref().unwrap().clone())
                      .unwrap()
                      .highlights
                      .get(&(state.chapter_number, line_number))
                    {
                      color.clone()
                    } else {
                      Color32::TRANSPARENT
                    },
                  )
                  .font(font_id.clone());

                // Applies special formatting (heading, bold, etc.)
                if let Some(info) = state
                  .book_userdata
                  .get_mut(state.selected_book_path.as_ref().unwrap())
                  .unwrap()
                  .formatting_info
                  .get(&(state.chapter_number, line_number))
                {
                  match info {
                    FormattingInfo::Title => {
                      text = text.size(font_id.size * 1.75);
                    }
                    FormattingInfo::Heading => {
                      text = text.size(font_id.size * 1.5);
                    }
                    FormattingInfo::Heading2 => {
                      text = text.size(font_id.size * 1.25);
                    }
                    FormattingInfo::Bold => text = text.strong(),
                    FormattingInfo::Italic => text = text.italics(),
                  }
                }

                text
              })
              .sense(Sense::click()),
            );

            if let Some(target) = &state.goto_target {
              if line_number == target.line as usize {
                goto_target_response = Some(response.clone());
              }
            }

            // Context menu
            response.context_menu(|ui| {
              if ui.button("Highlight").clicked() {
                let highlights = &mut state
                  .book_userdata
                  .get_mut(state.selected_book_path.as_ref().unwrap())
                  .unwrap()
                  .highlights;
                let coord = (state.chapter_number, line_number);

                if let Some(color) = highlights.get_mut(&coord) {
                  if *color != state.theme.highlight_color {
                    *color = state.theme.highlight_color;
                  } else {
                    highlights.remove(&coord);
                  };
                } else {
                  highlights.insert(coord, state.theme.highlight_color);
                }

                ui.close_menu();
              }

              if ui.button("Add Note").clicked() {
                let notes = &mut state
                  .book_userdata
                  .get_mut(state.selected_book_path.as_ref().unwrap())
                  .unwrap()
                  .notes;
                let note =
                  Note::new(book.get_current_page() as u16, line_number as u16);

                // Adds the note if one is not already in place for the specified chapter / line combo
                if !notes.contains(&note) {
                  notes.push(note);
                  state.ui_state.left_panel_state = PanelState::Notes;

                  ui.close_menu();
                }
              }
            });
          }

          if let Some(response) = goto_target_response {
            response.scroll_to_me(Some(egui::Align::TOP));
            state.goto_target = None;
          }
        }
      });
  } else {
    ui.label("No book loaded");
  }
}
