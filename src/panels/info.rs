pub fn ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
  if let Some(book) = &state.selected_book {
    ui.label(format!("{:#?}", book.metadata));
  }
}
