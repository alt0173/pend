use std::path::PathBuf;

use egui::{vec2, RichText, Sense, TextEdit, TextStyle};
use epub::doc::EpubDoc;

use crate::backend::{load_library, PathGroup};

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
      TextEdit::singleline(&mut state.shelf_search)
        .hint_text("Search Library...")
        .show(ui);
    }
  });
  ui.separator();

  // Avoids borrowing issues
  let mut path_group_remove_queue: Vec<String> = Vec::new();
  let mut book_remove_queue: Vec<(String, PathBuf)> = Vec::new();
  let path_group_names = state
    .shelf
    .iter()
    .map(|g| g.name.clone())
    .collect::<Vec<String>>();

  // Loops over books and handles display / etc.
  for path_group in state.shelf.iter_mut() {
    let response = ui.collapsing(&path_group.name, |ui| {
      egui::Grid::new(&path_group.name).show(ui, |ui| {
        for (index, path) in path_group.paths.iter().enumerate() {
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
                  .sense(Sense::click()),
                );

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

            // Controls number of columns
            if (index + 1) % 5 == 0 {
              ui.end_row();
            }
          }
        }
      });
    });

    // Header interaction
    response.header_response.context_menu(|ui| {
      if ui.button("Rename").clicked() {
        path_group.renaming = true;
      }
      if ui.button("Remove").clicked() {
        path_group_remove_queue.push(path_group.name.clone());
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

  // Executes scheduled removal of Books (PathBufs)
  if !book_remove_queue.is_empty() {
    for (shelf_name, path) in book_remove_queue.iter() {
      for shelf in state.shelf.iter_mut().find(|s| &s.name == shelf_name) {
        shelf.remove_path(path.into());
      }
    }
    book_remove_queue.clear();
  }

  // Executes scheduled removal of PathGroups
  if !path_group_remove_queue.is_empty() && state.shelf.len() > 1 {
    for name in path_group_remove_queue.iter() {
      for (index, path_group) in state.shelf.clone().iter().enumerate() {
        if &path_group.name == name {
          state.shelf.remove(index);
        }
      }
    }
    path_group_remove_queue.clear();
  }
}
