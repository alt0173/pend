use egui::{vec2, RichText, Sense, TextEdit, TextStyle};
use epub::doc::EpubDoc;

use crate::backend::{load_library, DraggedBook, PathGroup};

pub fn shelf_ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
  ui.horizontal(|ui| {
    if state.shelf.is_empty() {
      if ui.button("Load Library").clicked() {
        load_library(state);
      }

      TextEdit::singleline(&mut state.library_path)
        .hint_text("Path to books...")
        .show(ui);
    } else {
      if ui.button("New Shelf").clicked() {
        let mut shelf_number = state.shelf.len();
        let mut shelf_names = state.shelf.iter().map(|g| g.name.clone());

        // Prevents duplicate names
        while shelf_names.any(|x| x == format!("Shelf {}", shelf_number)) {
          shelf_number += 1;
        }
        state
          .shelf
          .push(PathGroup::new(&format!("Shelf {}", shelf_number)));
      }

      ui.separator();

      TextEdit::singleline(&mut state.shelf_search)
        .hint_text("Search Library...")
        .show(ui);
    }
  });
  ui.separator();

  // Avoids borrowing issues
  let mut remove_queue: Vec<String> = Vec::new();
  let path_group_names = state
    .shelf
    .iter()
    .map(|g| g.name.clone())
    .collect::<Vec<String>>();

  let mut dropped_book: Option<DraggedBook> = None;
  // Loops over books and handles display / etc.
  for path_group in state.shelf.iter_mut() {
    let response = ui.collapsing(&path_group.name, |ui| {
      egui::Grid::new(&path_group.name).show(ui, |ui| {
        for path in &path_group.paths {
          if let Ok(doc) = EpubDoc::new(path) {
            let title = doc.mdata("title").unwrap();
            let author = doc.mdata("creator").unwrap();

            // Only shows items searched for (shows all if search is empty)
            if title.contains(&state.shelf_search)
              || author.contains(&state.shelf_search)
            {
              // Display the cover & info
              ui.vertical_centered(|ui| {
                // Cover
                let response = ui.add(
                  egui::ImageButton::new(
                    state
                      .book_covers
                      .get(&title)
                      .unwrap_or(state.book_covers.get("fallback").unwrap())
                      .texture_id(ui.ctx()),
                    vec2(state.book_cover_size, state.book_cover_size * 1.6),
                  )
                  .sense(Sense::drag()),
                );

                // Drag stuff
                if response.drag_started() {
                  state.dragged_book = Some(DraggedBook::new(
                    path.clone(),
                    title.clone(),
                    path_group.name.clone(),
                  ));
                }
                if response.drag_released() {
                  dropped_book = state.dragged_book.clone();
                  state.dragged_book = None;
                }

                // Selection
                if response.clicked()
                  && state.selected_book_path != Some(path.to_path_buf())
                {
                  state.selected_book = Some(EpubDoc::new(path).unwrap());
                  state.selected_book_path = Some(path.clone());
                  state.chapter_number = 1;
                }

                // Display info
                ui.label(RichText::new(title).text_style(TextStyle::Body));
                if let Some(author) = doc.mdata("creator") {
                  ui.label(RichText::new(author).text_style(TextStyle::Body));
                }
              });
            }
          }
        }
      });
    });

    ui.label("SNEEGUS");

    // Header interaction
    response.header_response.context_menu(|ui| {
      if ui.button("Rename").clicked() {
        path_group.renaming = true;
      }
      if ui.button("Remove").clicked() {
        remove_queue.push(path_group.name.clone());
      }
    });

    // Rename stuff
    let current_name = &mut path_group.name;
    if path_group.renaming {
      egui::Window::new("Shelf Renamer")
        .resizable(false)
        .collapsible(false)
        .show(ui.ctx(), |ui| {
          TextEdit::singleline(&mut path_group.desired_name)
            .hint_text(current_name.clone())
            .show(ui);

          if ui.ctx().input().key_pressed(egui::Key::Enter) {
            if path_group.desired_name != *current_name
              && !path_group_names.contains(&path_group.desired_name)
            {
              *current_name = path_group.desired_name.clone();
            }

            ui.close_menu();
            path_group.renaming = false;
          }
          if ui.ctx().input().key_pressed(egui::Key::Escape) {
            path_group.desired_name = "".into();
            ui.close_menu();
            path_group.renaming = false;
          }
        });
    }
  }

  if state.dragged_book.is_some() || dropped_book.is_some() {
    ui.centered_and_justified(|ui| {
      if ui
        .button("New Shelf")
        .rect
        .contains(ui.ctx().pointer_hover_pos().unwrap())
      {
        if let Some(book) = dropped_book {
          let mut shelf_number = state.shelf.len();
          let mut shelf_names = state.shelf.iter().map(|g| g.name.clone());

          // Prevents duplicate shelf names
          while shelf_names.any(|x| x == format!("Shelf {}", shelf_number)) {
            shelf_number += 1;
          }

          // Remove the book from it's previous shelf
          for path_group in state.shelf.iter_mut() {
            path_group.remove_path(book.path.clone());
          }

          // Create the new shelf with the dragged book
          state.shelf.push({
            let mut group = PathGroup::new(&format!("Shelf {}", shelf_number));
            group.paths.push(book.path.clone());

            group
          });
        }
      }
    });
  }

  if !remove_queue.is_empty() && state.shelf.len() > 1 {
    for name in remove_queue.iter() {
      for (index, path_group) in state.shelf.clone().iter().enumerate() {
        if &path_group.name == name {
          state.shelf.remove(index);
        }
      }
    }
    remove_queue.clear();
  }
}
