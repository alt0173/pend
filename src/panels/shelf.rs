use egui::{vec2, RichText, TextEdit, TextStyle};
use epub::doc::EpubDoc;

use crate::backend::{load_library, PathGroup};

pub fn shelf_ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
  ui.horizontal(|ui| {
    if ui.button("Load Library").clicked() {
      load_library(state);
    }
    if ui.button("Clear Library").clicked() {
      state.shelf.clear();
      state.book_covers.clear();
      state.selected_book = None;
      state.selected_book_path = None;
    }
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
  });
  ui.separator();

  // Controls the scale of book covers
  let y = ui.available_size_before_wrap().y / 3.8;
  // Avoids borrowing issues
  let mut remove_queue: Vec<String> = Vec::new();
  let path_group_names = state
    .shelf
    .iter()
    .map(|g| g.name.clone())
    .collect::<Vec<String>>();

  for path_group in state.shelf.iter_mut() {
    let response = ui.collapsing(&path_group.name, |ui| {
      egui::Grid::new(&path_group.name)
        .striped(true)
        .show(ui, |ui| {
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
                  if ui
                    .add(egui::ImageButton::new(
                      state
                        .book_covers
                        .get(&title)
                        .unwrap()
                        .texture_id(ui.ctx()),
                      vec2(y / 1.6, y),
                    ))
                    .clicked()
                    && state.selected_book_path != Some(path.to_path_buf())
                  {
                    state.selected_book = Some(EpubDoc::new(path).unwrap());
                    state.selected_book_path = Some(path.clone());
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
        })
    });

    response.header_response.context_menu(|ui| {
      if ui.button("Rename").clicked() {
        path_group.renaming = true;
      }
      if ui.button("Remove").clicked() {
        remove_queue.push(path_group.name.clone());
      }
    });

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
