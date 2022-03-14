use egui::{Color32, DragValue, FontId, Label, RichText, ScrollArea, Sense};

use crate::{
  backend::{parse_calibre, FormattingInfo},
  ui::{Note, PanelState},
  MyApp,
};

pub fn right_panel_reader_ui(state: &mut MyApp, ui: &mut egui::Ui) {
  // Displays page(s) of the book
  if let Some(book) = &mut state.selected_book {
    // If a book is loaded there must be a path, only panis if
    // unexpected unloading occurs
    let selected_book_path = state.selected_book_path.as_ref().unwrap();

    if let Some(target) = &state.goto_target {
      state.chapter_number = target.chapter as usize;
    }

    // Key-based page navigation
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

    // Button-based page navigation
    if state.ui_state.reader_focus_mode {
      ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(), |ui| {
          if ui.button("Exit Focus").clicked() {
            state.ui_state.reader_focus_mode = false;
          }
        });
      });
    } else {
      ui.horizontal(|ui| {
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
      });
    }

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

          if let Ok(page_data) = book.get_current_str() {
            let contents = parse_calibre(
              &page_data,
              book.get_current_page(),
              state.book_userdata.get_mut(selected_book_path).unwrap(),
            );
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
              let response = ui.add(
                Label::new({
                  // Creates text with normal / default appearence
                  // This is how normal body text looks
                  let mut text = RichText::new(line)
                    .color(theme.text_color)
                    .background_color(
                      if let Some(color) = state
                        .book_userdata
                        .get(selected_book_path)
                        .unwrap()
                        .highlights
                        .get(&(state.chapter_number, line_number))
                      {
                        *color
                      } else {
                        Color32::TRANSPARENT
                      },
                    )
                    .font(font_id.clone());

                  // Applies special formatting (heading, bold, etc.)
                  if let Some(info) = state
                    .book_userdata
                    .get_mut(selected_book_path)
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
                    .get_mut(selected_book_path)
                    .unwrap()
                    .highlights;
                  let coord = (state.chapter_number, line_number);

                  if let Some(color) = highlights.get_mut(&coord) {
                    if *color == state.theme.highlight_color {
                      highlights.remove(&coord);
                    } else {
                      *color = state.theme.highlight_color;
                    };
                  } else {
                    highlights.insert(coord, state.theme.highlight_color);
                  }

                  ui.close_menu();
                }

                if ui.button("Add Note").clicked() {
                  let notes = &mut state
                    .book_userdata
                    .get_mut(selected_book_path)
                    .unwrap()
                    .notes;
                  let note = Note::new(
                    book.get_current_page() as u16,
                    line_number as u16,
                  );

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
          } else {
            ui.label("Unable to load page data");
          }
        }
      });
  } else {
    ui.label("No book loaded");
  }
}
