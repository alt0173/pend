use egui::{vec2, Button, RichText, TextEdit};
use epub::doc::EpubDoc;

use crate::backend::{load_library, PathGroup};

pub fn ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
  // Top menu bar
  ui.horizontal(|ui| {
    if state.shelves.is_empty() {
      if ui.button("Load Library").clicked() {
        load_library(state);
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
          .button(if state.shelf_reorganize_mode {
            "\u{1F513}"
          } else {
            "\u{1F512}"
          })
          .clicked()
        {
          state.shelf_reorganize_mode ^= true;
        }
      });
    }
  });
  ui.separator();

  // Loop over all shelves
  for (shelf_index, path_group) in state.shelves.clone().iter().enumerate() {
    let collapsing_response = ui.collapsing(path_group.name.clone(), |ui| {
      egui::Grid::new(&path_group.name).show(ui, |ui| {
        // Loop over all paths within a shelf and show the books
        for (path_index, path) in path_group.paths.iter().enumerate() {
          // Ensure the path leads to a valid epub document
          if let Ok(doc) = EpubDoc::new(path) {
            let title = doc
              .mdata("title")
              .unwrap_or_else(|| "<Missing Title>".to_string());
            let author = doc
              .mdata("creator")
              .unwrap_or_else(|| "<Missing Title>".to_string());

            // If searching: only show items beind searched for
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

                // Cover image button thing
                let cover_response = ui.add(
                  egui::ImageButton::new(
                    state
                      .book_covers
                      .get(&title)
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

                if state.shelf_reorganize_mode {
                  // Set dragged book when dragged
                  if cover_response.drag_started() {
                    state.dragged_book =
                      Some((path.clone(), title, path_group.name.clone()));
                  }

                  // Drag & Drop
                  if let (
                    Some(mouse_position),
                    Some((dragged_path, _dragged_title, old_shelf_name)),
                    true,
                  ) = (
                    ui.ctx().pointer_hover_pos(),
                    state.dragged_book.as_ref(),
                    ui.ctx().input().pointer.any_released(),
                  ) {
                    if cover_response.rect.contains(mouse_position) {
                      // Find the shelf the dragged book's path is in and remove the path from it
                      state
                        .shelves
                        .iter_mut()
                        .find(|s| s.name == *old_shelf_name)
                        .unwrap()
                        .paths
                        .retain(|p| p != dragged_path);

                      // Add path to shelf after this book
                      state.shelves[shelf_index]
                        .paths
                        .insert(path_index + 1, dragged_path.clone());

                      state.dragged_book = None;
                    }
                  }
                } else {
                  // Select book on click
                  if cover_response.clicked() {
                    state.selected_book = Some(EpubDoc::new(path).unwrap());
                    state.selected_book_path = Some(path.clone());
                    state.chapter_number = 1;
                  }
                }

                // Context menu
                cover_response.context_menu(|ui| {
                  if ui.button("Remove").clicked() {
                    state.shelves[shelf_index].paths.retain(|p| p != path);
                    ui.close_menu();
                  }
                });
              });
            }
          }
        }
      });
    });

    // Shelf context menu
    collapsing_response.header_response.context_menu(|ui| {
      if ui
        .add_enabled(state.shelves.len() > 1, Button::new("Remove"))
        .clicked()
      {
        for path in &state.shelves[shelf_index].paths.clone() {
          state.shelves[shelf_index - 1].paths.push(path.clone());
        }

        state.shelves.remove(shelf_index);
      };
    });
  }

  // Shelf addition
  if let Some(mouse_position) = ui.ctx().pointer_hover_pos() {
    if let Some((path, _, old_shelf_name)) = &state.dragged_book {
      ui.centered_and_justified(|ui| {
        if ui.button("New Shelf").rect.contains(mouse_position)
          && ui.ctx().input().pointer.any_released()
        {
          // Find the shelf the dragged book's path is in and remove the path from it
          state
            .shelves
            .iter_mut()
            .find(|s| s.name == *old_shelf_name)
            .unwrap()
            .paths
            .retain(|p| p != path);
          // Create new shelf
          // Ensure that the name of the new PathGroup will be unique
          let mut shelf_number: u16 = 1;
          let mut shelf_name = String::from("Shelf 1");

          // PathGroup comparrision works soley on name, so it's easy to
          // search for potential name collisions
          while state.shelves.contains(&PathGroup::new(&shelf_name)) {
            shelf_name = format!("Shelf {}", shelf_number);
            shelf_number += 1;
          }

          // Create shelf with name, add book to it, and push it
          let shelf =
            PathGroup::new_with_contents(shelf_name, Vec::from([path.clone()]));
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
