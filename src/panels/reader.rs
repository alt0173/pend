use egui::{Color32, FontId, Label, RichText, ScrollArea, Sense};

use crate::{
  backend::{parse_calibre, FormattingInfo},
  ui::{Note, PanelState},
  MyApp,
};

pub fn right_panel_reader_ui(state: &mut MyApp, ui: &mut egui::Ui) {
  // Displays page(s) of the book
  if let Some(book) = &mut state.selected_book {
    // If a book is loaded there must be a path, only panics if
    // unexpected unloading occurs
    let selected_book_path = state.selected_book_path.as_ref().unwrap();

    let book_userdata =
      state.book_userdata.get_mut(selected_book_path).unwrap();

    if let Some(target) = &state.goto_target {
      book_userdata.chapter = target.chapter as usize;
    }

    // Key-based page navigation
    if ui.ctx().input().key_pressed(egui::Key::ArrowLeft)
      && book.get_current_page() > 1
    {
      book_userdata.chapter -= 1;
    }
    if ui.ctx().input().key_pressed(egui::Key::ArrowRight)
      && book.get_current_page() < book.get_num_pages() - 1
    {
      book_userdata.chapter += 1;
    }

    // Skip to avoid trippled length
    #[rustfmt::skip]
    ui.horizontal(|ui| {
      if state.ui_state.reader_focus_mode {
        // Collapse focus
        if ui.add(egui::Button::new(RichText::new("Unfocus"))
				).clicked() {
          state.ui_state.reader_focus_mode = false;
        }
      } else {
        // Expand focus
        if ui.add(egui::Button::new(RichText::new("Focus"))
				).clicked() {
          state.ui_state.reader_focus_mode = true;
        }

        // Display chapter number
        ui.with_layout(egui::Layout::right_to_left(), |ui| {
          ui.label(format!("Chapter: {}", &book_userdata.chapter));
        });
      }
    });

    // Apply page / chapter change of needed
    if book.get_current_page() != book_userdata.chapter {
      book.set_current_page(book_userdata.chapter).unwrap();
      state.goto_target = Some(Note::new(book_userdata.chapter as u16, 0));
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

          if let Ok(page_data) = book.get_current_str() {
            let contents =
              parse_calibre(&page_data, book.get_current_page(), book_userdata);
            let contents = contents.lines();

            // Background
            ui.painter()
              .rect_filled(ui.clip_rect(), 0.0, theme.page_color);

            // Actual "stuff"
            let font_id =
              FontId::new(style.font_size, style.font_family.clone());
            let line_spacing =
              ui.fonts().row_height(&font_id) * style.line_spacing_multiplier;

            ui.style_mut().spacing.item_spacing.y = line_spacing;

            let mut goto_target_response = None;

            for (line_number, line) in contents.into_iter().enumerate() {
              let line_response = ui.add(
                Label::new({
                  // Creates text with normal / default appearence
                  // This is how normal body text looks
                  let mut text = RichText::new(line)
                    .color(theme.text_color)
                    .background_color(
                      if let Some(color) = book_userdata
                        .highlights
                        .get(&(book_userdata.chapter, line_number))
                      {
                        *color
                      } else {
                        Color32::TRANSPARENT
                      },
                    )
                    .font(font_id.clone());

                  // Applies special formatting (heading, bold, etc.)
                  if let Some(info) = book_userdata
                    .formatting_info
                    .get(&(book_userdata.chapter, line_number))
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
                      FormattingInfo::Break => {
                        // TODO: Add some sort of newline / break here
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
                  goto_target_response = Some(line_response.clone());
                }
              }

              // Context menu
              line_response.context_menu(|ui| {
                ui.horizontal(|ui| {
                  for (index, color) in [
                    state.theme.highlight_color,
                    Color32::from_rgb(255, 150, 138),
                    Color32::from_rgb(255, 209, 138),
                    Color32::from_rgb(138, 255, 150),
                    Color32::from_rgb(150, 138, 255),
                  ]
                  .iter()
                  .enumerate()
                  {
                    // Seperator placed after the first option to indicate
                    // the user's custom selected highlight color
                    if index == 1 {
                      ui.separator();
                      ui.add_space(6.0);
                    }

                    // Button & logic
                    if ui
                      .button(
                        RichText::new("\u{25CF}").color(*color).size(32.0),
                      )
                      .clicked()
                    {
                      let coord = (book_userdata.chapter.clone(), line_number);

                      if let Some(existing_color) =
                        book_userdata.highlights.get_mut(&coord)
                      {
                        if *existing_color == *color {
                          book_userdata.highlights.remove(&coord);
                        } else {
                          *existing_color = *color;
                        };
                      } else {
                        book_userdata.highlights.insert(coord, *color);
                      }

                      ui.close_menu();
                    }
                  }
                });

                if ui.button("Copy").clicked() {
                  ui.output().copied_text = line.to_string();
                  ui.close_menu();
                }

                if ui.button("Add Note").clicked() {
                  let note = Note::new(
                    book.get_current_page() as u16,
                    line_number as u16,
                  );

                  // Adds the note if one is not already in place for the specified chapter / line combo
                  if !book_userdata.notes.contains(&note) {
                    book_userdata.notes.push(note);
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
          } else {
            ui.label("Unable to load page data");
          }
        }
      });
  } else {
    ui.label("No book loaded");
  }
}
