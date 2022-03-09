use egui::{vec2, TextEdit};
use epub::doc::EpubDoc;

use crate::backend::load_library;

pub fn shelf_ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
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
            if cover_response.drag_released() {
              state.dragged_book = None;
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

  // Image for book drag
  // Note that this must be after everything else in this ui to ensure it be drawn on top
  if let Some((_, title)) = &state.dragged_book {
    if let Some(mouse_position) = ui.ctx().pointer_hover_pos() {
      let size = vec2(state.book_cover_width, state.book_cover_width * 1.6);
      let image = egui::Image::new(
        state
          .book_covers
          .get(title)
          .unwrap_or(state.book_covers.get("fallback").unwrap())
          .texture_id(ui.ctx()),
        size,
      );

      ui.put(
        egui::Rect::from_min_max(mouse_position, mouse_position + size),
        image,
      );
    }
  }
}
