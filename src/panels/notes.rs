use egui::TextEdit;

use crate::ui::Note;

pub fn ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
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

      // Can't have mutable borrow while iterating w/ mutability so a helper is needed
      let mut to_delete = None;

      for (index, note) in notes.iter_mut().enumerate() {
        let (chapter, line, content) =
          (note.chapter, note.line, &mut note.content);

        ui.horizontal(|ui| {
          let response =
            ui.collapsing(format!("Ch. {}, line: {}", chapter, line), |ui| {
              TextEdit::multiline(content).show(ui);
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
