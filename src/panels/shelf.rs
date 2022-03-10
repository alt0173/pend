use std::time::Instant;

use egui::{vec2, TextEdit};
use epub::doc::EpubDoc;

use crate::backend::load_library;

pub fn shelf_ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
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
    }
  });
  ui.separator();

  // Loop over all shelves
  for (shelf_index, path_group) in state.shelves.clone().iter().enumerate() {
    ui.collapsing(path_group.name.clone(), |ui| {
      ui.horizontal_wrapped(|ui| {
        // Loop over all paths within a shelf
        for (index, path) in path_group.paths.iter().enumerate() {
          // Ensure the path leads to a valid epub document
          if let Ok(doc) = EpubDoc::new(path) {
            let title =
              doc.mdata("title").unwrap_or("<Missing Title>".to_string());

            // Cover image button thing
            let cover_response = ui.add(
              egui::ImageButton::new(
                state
                  .book_covers
                  .get(&title)
                  .unwrap_or(state.book_covers.get("fallback").unwrap())
                  .texture_id(ui.ctx()),
                vec2(state.book_cover_width, state.book_cover_width * 1.6),
              )
              .sense(egui::Sense::click_and_drag()),
            );

            // Select book on click
            if cover_response.clicked() {
              state.selected_book = Some(EpubDoc::new(path).unwrap());
              state.selected_book_path = Some(path.to_path_buf());
              state.chapter_number = 1;
            }

            if cover_response.drag_started() {
              state.dragged_book = Some((path.to_path_buf(), title));
            }

            // Context menu
            cover_response.context_menu(|ui| {
              if ui.button("Remove").clicked() {
                state.shelves[shelf_index].remove_path(path.to_path_buf());
                ui.close_menu();
              }
            });
          }
        }
      });
    });
  }

  if state.dragged_book.is_some() {
    ui.centered_and_justified(|ui| {
      if ui.button("New Shelf").clicked() {};
    });
  }

  // If dragged book is Some (not already consumed) and mouse button released, set it to None
  if ui.ctx().input().pointer.any_released() && state.dragged_book.is_some() {
    state.dragged_book = None;
  }

  // Shows the cover of the book currently being dragged
  match (&state.dragged_book, ui.ctx().pointer_hover_pos()) {
    (Some((_, title)), Some(mouse_position)) => {
      egui::Area::new("Book Cover Drag Area")
        .fixed_pos(mouse_position)
        .order(egui::Order::Foreground)
        .show(ui.ctx(), |ui| {
          let image_size =
            vec2(state.book_cover_width, state.book_cover_width * 1.6);

          ui.image(
            state
              .book_covers
              .get(title)
              .unwrap_or(state.book_covers.get("fallback").unwrap())
              .texture_id(ui.ctx()),
            image_size,
          );
        });
    }
    _ => {}
  }
}
