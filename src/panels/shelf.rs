use std::io::Cursor;

use crate::backend::{load_directory, register_epub, RenameState, Shelf};
use egui::{vec2, Align2, RichText, TextEdit};
use epub::doc::EpubDoc;

pub fn ui(state: &mut crate::Pend, ui: &mut egui::Ui) {
  for file in &ui.ctx().input().raw.dropped_files {
    // Loading epubs for da web
    if let Some(bytes) = &file.bytes {
      let bytes = bytes.to_vec();
      let bytes_cursor = Cursor::new(bytes);

      if let Ok(epub) = EpubDoc::from_reader(bytes_cursor) {
        register_epub(state, epub);
      };
    }
  }

  // Top menu bar
  ui.horizontal(|ui| {
    if state.shelves.is_empty() {
      if ui.button("Load Library").clicked() {
        load_directory(state, state.library_path.clone());
      }

      TextEdit::singleline(&mut state.library_path)
        .hint_text("Path to books...")
        .show(ui);
    } else {
      TextEdit::singleline(&mut state.shelf_search)
        .hint_text("Search Library...")
        .show(ui);

      ui.with_layout(egui::Layout::right_to_left(), |ui| {
        if ui
          .button(
            RichText::new(if state.reorganizing_shelf {
              // Open lock
              "\u{1F513}"
            } else {
              // Closed lock
              "\u{1F512}"
            })
            .monospace(),
          )
          .clicked()
        {
          state.reorganizing_shelf ^= true;
        }
      });
    }
  });
  ui.separator();

  // Loop over all shelves
  for (shelf_index, path_group) in state.shelves.clone().iter().enumerate() {
    // Renaming window
    if path_group.renaming != RenameState::Inactive {
      egui::Window::new("Rename Shelf")
        .auto_sized()
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ui.ctx(), |ui| {
          let shelf_names = state.shelves.clone();
          let mut shelf_names = shelf_names.iter();
          let shelf = &mut state.shelves[shelf_index];

          // The textedit
          TextEdit::singleline(&mut shelf.desired_name)
            .hint_text("Type Name Here...")
            .show(ui);

          if shelf.name == shelf.desired_name {
            if ui.ctx().input().key_pressed(egui::Key::Enter) {
              shelf.renaming = RenameState::Inactive;
            }
          } else if shelf.desired_name.chars().count() > 32
            || shelf_names.any(|x| x.name == shelf.desired_name.to_lowercase())
          {
            ui.label("Invalid name");
          } else if ui.ctx().input().key_pressed(egui::Key::Enter) {
            shelf.name = shelf.desired_name.clone();
            shelf.renaming = RenameState::Inactive;
          }

          if ui.ctx().input().key_pressed(egui::Key::Escape) {
            shelf.renaming = RenameState::Inactive;
          }
        });
    }

    // The collapsing header / section
    let collapsing_response = ui.collapsing(path_group.name.clone(), |ui| {
      egui::Grid::new(&path_group.name).show(ui, |ui| {
        // Loop over all paths within a shelf and show the books
        for (uuid_index, uuid) in path_group.uuids.iter().enumerate() {
          // Ensure the path leads to a valid epub document
          if let Some(epub) = state.epub_cache.get(uuid) {
            let title = epub
              .mdata("title")
              .unwrap_or_else(|| "<Missing Title>".to_string());
            let author = epub
              .mdata("creator")
              .unwrap_or_else(|| "<Missing Title>".to_string());

            // Only show items being searched for
            if title
              .to_lowercase()
              .contains(&state.shelf_search.to_lowercase())
              || author
                .to_lowercase()
                .contains(&state.shelf_search.to_lowercase())
            {
              ui.vertical_centered(|ui| {
                // This is very important to ensure everything disaplys sanely
                ui.set_max_width(140.0 * state.book_cover_width_multiplier);

                // The button / image of the book's cover
                let cover_response = ui.add(
                  egui::ImageButton::new(
                    state
                      .book_covers
                      .get(uuid)
                      .unwrap_or_else(|| {
                        state.book_covers.get("fallback").unwrap()
                      })
                      .texture_id(ui.ctx()),
                    vec2(
                      140.0 * state.book_cover_width_multiplier,
                      140.0 * state.book_cover_width_multiplier * 1.6,
                    ),
                  )
                  .sense(egui::Sense::click_and_drag()),
                );

                // Book data / information
                ui.label(
                  RichText::new(&title)
                    .size(140.0 * state.book_cover_width_multiplier / 10.0),
                );
                ui.label(
                  RichText::new(&author)
                    .size(140.0 * state.book_cover_width_multiplier / 10.0),
                );

                if state.reorganizing_shelf {
                  // Set dragged book when dragged
                  if cover_response.drag_started() {
                    state.dragged_book =
                      Some((uuid.clone(), title, path_group.name.clone()));
                  }

                  // Drag & Drop
                  if let (
                    Some(mouse_position),
                    Some((dragged_uuid, _dragged_title, old_shelf_name)),
                    true,
                  ) = (
                    ui.ctx().pointer_hover_pos(),
                    state.dragged_book.as_ref(),
                    ui.ctx().input().pointer.any_released(),
                  ) {
                    if cover_response.rect.contains(mouse_position)
                      && uuid != dragged_uuid
                    {
                      // Find the shelf the dragged book's path is in and remove the path from it
                      state
                        .shelves
                        .iter_mut()
                        .find(|s| s.name == *old_shelf_name)
                        .unwrap()
                        .uuids
                        .retain(|p| p != dragged_uuid);

                      // Add path to shelf after this book
                      if uuid_index >= state.shelves[shelf_index].uuids.len() {
                        state.shelves[shelf_index]
                          .uuids
                          .push(dragged_uuid.clone());
                      } else {
                        state.shelves[shelf_index]
                          .uuids
                          .insert(uuid_index, dragged_uuid.clone());
                      }

                      state.dragged_book = None;
                    }
                  }
                } else {
                  // Select book on click
                  if cover_response.clicked() {
                    state.selected_book_uuid =
                      Some(epub.unique_identifier.as_ref().unwrap().clone());
                  }
                }

                // Context menu
                if !state.reorganizing_shelf {
                  cover_response.context_menu(|ui| {
                    if ui.button("Remove").clicked() {
                      state.shelves[shelf_index].uuids.retain(|p| p != uuid);
                      ui.close_menu();
                    }
                  });
                }
              });
            }

            if (uuid_index + 1)
              % (5.0 / state.book_cover_width_multiplier).round() as usize
              == 0
            {
              ui.end_row();
            }
          }
        }
      });
    });

    // Shelf context menu
    if path_group.renaming == RenameState::Inactive {
      collapsing_response.header_response.context_menu(|ui| {
        if ui.button("Rename").clicked() {
          state.shelves[shelf_index].renaming = RenameState::Active;
          state.shelves[shelf_index].desired_name = path_group.name.clone();
          ui.close_menu();
        }

        // Only allows the shelf to be deleted if there is >1 other shelves
        ui.set_enabled(state.shelves.len() > 1);
        ui.menu_button("Remove Shelf", |ui| {
          if ui.button("Confirm").clicked() {
            for path in &state.shelves[shelf_index].uuids.clone() {
              if shelf_index == 0 {
                state.shelves[1].uuids.push(path.clone());
              } else {
                state.shelves[shelf_index - 1].uuids.push(path.clone());
              }
            }

            state.shelves.remove(shelf_index);
          };

          if ui.button("Cancel").clicked() {
            ui.close_menu();
          };
        });
      });
    }
  }

  // Shelf addition
  if let Some(mouse_position) = ui.ctx().pointer_hover_pos() {
    if let Some((uuid, _, old_shelf_name)) = &state.dragged_book {
      ui.centered_and_justified(|ui| {
        if ui.button("New Shelf").rect.contains(mouse_position)
          && ui.ctx().input().pointer.any_released()
        {
          // Find the shelf the dragged book's uuid is in and remove the uuid
          state
            .shelves
            .iter_mut()
            .find(|s| s.name == *old_shelf_name)
            .unwrap()
            .uuids
            .retain(|u| u != uuid);
          // Create new shelf
          // Ensure that the name of the new PathGroup will be unique
          let mut shelf_number: u16 = 1;
          let mut shelf_name = String::from("Shelf 1");

          // PathGroup comparrision works soley on name, so it's easy to
          // search for potential name collisions
          while state.shelves.contains(&Shelf::new(&shelf_name)) {
            shelf_name = format!("Shelf {}", shelf_number);
            shelf_number += 1;
          }

          // Create shelf with name, add book to it, and push it
          let shelf =
            Shelf::new_with_contents(shelf_name, Vec::from([uuid.clone()]));
          state.shelves.push(shelf);
        };
      });
    }
  }

  // If dragged book is Some (not already set to None) and mouse button
  // released: set it to None
  if ui.ctx().input().pointer.any_released() && state.dragged_book.is_some() {
    state.dragged_book = None;
  }

  // Shows the cover of the book currently being dragged, if any
  if let (Some((_, title, _)), Some(mouse_position)) =
    (&state.dragged_book, ui.ctx().pointer_hover_pos())
  {
    egui::Area::new("Book Cover Drag Area")
      .fixed_pos(mouse_position)
      .order(egui::Order::Foreground)
      .show(ui.ctx(), |ui| {
        let image_size = vec2(
          140.0 * state.book_cover_width_multiplier,
          140.0 * state.book_cover_width_multiplier * 1.6,
        );

        ui.image(
          state
            .book_covers
            .get(title)
            .unwrap_or_else(|| state.book_covers.get("fallback").unwrap())
            .texture_id(ui.ctx()),
          image_size,
        );
      });
  }
}
