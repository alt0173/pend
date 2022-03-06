use std::sync::Arc;

use egui::{ComboBox, FontFamily, TextEdit};

use crate::ui::{BookTextStyle, DocumentColors};

pub fn config_ui(state: &mut crate::MyApp, ui: &mut egui::Ui) {
  ui.collapsing("Program", |ui| {
    // Path to directory containing books
    ui.horizontal(|ui| {
      ui.label("Library Path:");
      TextEdit::singleline(&mut state.shelf_path)
        .hint_text(r"e.g. C:\Users\Public\Documents\Lisci")
        .show(ui);
    });

    ui.checkbox(&mut state.ui_state.reader_focus_mode, "Focus Mode");
  });

  ui.collapsing("Document", |ui| {
    ComboBox::from_label("Font")
      .selected_text(match &state.book_style.font_family {
        f if f == &FontFamily::Proportional => "Work Sans",
        f if f == &FontFamily::Name(Arc::from("Merriweather")) => {
          "Merriweather"
        }
        _ => "Unrecognized Font",
      })
      .show_ui(ui, |ui| {
        ui.selectable_value(
          &mut state.book_style.font_family,
          FontFamily::Proportional,
          "Work Sans",
        );
        ui.selectable_value(
          &mut state.book_style.font_family,
          FontFamily::Name("Merriweather".into()),
          "Merriweather",
        );
      });

    ui.add(
      egui::Slider::new(&mut state.book_style.font_size, 12.0..=120.0)
        .step_by(0.25)
        .prefix("Text Size: "),
    );

    ui.add(
      egui::Slider::new(
        &mut state.book_style.line_spacing_multiplier,
        0.0..=6.0,
      )
      .step_by(0.25)
      .prefix("Line Spacing: "),
    );

    ui.collapsing("Colors", |ui| {
      ui.horizontal(|ui| {
        ui.color_edit_button_srgba(&mut state.theme.highlight_color);
        ui.label(": Highlight Color");
      });
      ui.horizontal(|ui| {
        ui.color_edit_button_srgba(&mut state.theme.text_color);
        ui.label(": Text Color");
      });
      ui.horizontal(|ui| {
        ui.color_edit_button_srgba(&mut state.theme.page_color);
        ui.label(": Page Color");
      });

      ui.separator();

      if ui.button("Reset Colors").clicked() {
        state.theme = DocumentColors::default()
      }
    });

    ui.separator();

    ui.horizontal(|ui| {
      if ui.button("Reset Style").clicked() {
        state.book_style = BookTextStyle::default();
      }
      if ui.button("Clear Highlights").clicked() {
        if let Some(path) = &state.selected_book_path {
          state
            .book_userdata
            .get_mut(path)
            .unwrap()
            .highlights
            .clear();
        }
      }
    });
  });

  ui.collapsing("Other", |ui| {
    if ui.button("Acknowledgements").clicked() {
      state.ui_state.display_ofl_popup = true;
    }
    // ui.checkbox(&mut state.ui_state.display_raw_text, "[DEBUG] Display Raw Text");
  });
}
