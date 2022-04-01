pub use crate::panels::reader;
use crate::ui::{
  BookTextStyle, DocumentColors, Note, PanelState, UIState, BLUISH,
  DARKISH_BLUISH, DARK_BLUISH, LIGHTISH_BLUISH, LIGHT_BLUISH,
};
use crate::{
  backend::{LocalBookInfo, PathGroup},
  ui,
};
use eframe::{
  egui::{self, style::WidgetVisuals, FontDefinitions},
  epaint::{FontFamily, Rounding},
  epi,
};
use egui::{Color32, Stroke};
use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::PathBuf, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct Pend {
  pub ui_state: UIState,
  pub library_path: String,
  pub shelves: Vec<PathGroup>,
  pub shelf_search: String,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  pub book_covers: HashMap<String, RetainedImage>,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  pub selected_book: Option<EpubDoc<File>>,
  pub selected_book_path: Option<PathBuf>,
  pub book_style: BookTextStyle,
  pub book_userdata: HashMap<PathBuf, LocalBookInfo>,
  pub goto_target: Option<Note>,
  pub theme: DocumentColors,
  pub book_cover_width_multiplier: f32,
  /// Path, original shelf name, title
  pub dragged_book: Option<(PathBuf, String, String)>,
  pub reorganizing_shelf: bool,
}

impl Default for Pend {
  fn default() -> Self {
    Self {
      ui_state: UIState {
        left_panel_state: PanelState::Shelf,
        right_panel_state: PanelState::Reader,
        reader_focus_mode: false,
        display_ofl_popup: false,
        display_raw_text: false,
      },
      library_path: "./library".into(),
      shelves: Vec::new(),
      shelf_search: String::new(),
      book_covers: HashMap::new(),
      selected_book: None,
      selected_book_path: None,
      book_style: BookTextStyle::default(),
      book_userdata: HashMap::new(),
      goto_target: None,
      theme: DocumentColors::default(),
      book_cover_width_multiplier: 1.0,
      dragged_book: None,
      reorganizing_shelf: false,
    }
  }
}

impl epi::App for Pend {
  fn setup(
    &mut self,
    ctx: &egui::Context,
    _frame: &epi::Frame,
    _storage: Option<&dyn epi::Storage>,
  ) {
    // Load memory (only in release mode)
    #[cfg(not(debug_assertions))]
    if let Some(storage) = _storage {
      *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
    }

    // Configure egui
    ctx.set_style(egui::Style {
      override_text_style: Some(egui::TextStyle::Heading),
      override_font_id: Some(eframe::epaint::FontId::new(
        20.0,
        FontFamily::Proportional,
      )),
      visuals: egui::Visuals {
        dark_mode: true,
        widgets: egui::style::Widgets {
          noninteractive: WidgetVisuals {
            bg_fill: DARK_BLUISH,
            bg_stroke: Stroke::new(2.5, DARKISH_BLUISH),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(180)),
            rounding: Rounding::same(1.5),
            expansion: 0.0,
          },
          inactive: WidgetVisuals {
            bg_fill: BLUISH,
            bg_stroke: Stroke::new(2.0, LIGHTISH_BLUISH),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(200)),
            rounding: Rounding::same(1.5),
            expansion: 0.0,
          },
          hovered: WidgetVisuals {
            bg_fill: LIGHTISH_BLUISH,
            bg_stroke: Stroke::new(2.0, LIGHT_BLUISH),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(220)),
            rounding: Rounding::same(1.5),
            expansion: 1.0,
          },
          active: WidgetVisuals {
            bg_fill: LIGHTISH_BLUISH,
            bg_stroke: Stroke::new(2.0, LIGHT_BLUISH),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(220)),
            rounding: Rounding::same(1.5),
            expansion: 1.0,
          },
          open: WidgetVisuals {
            bg_fill: DARK_BLUISH,
            bg_stroke: Stroke::new(2.5, DARKISH_BLUISH),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(200)),
            rounding: Rounding::same(1.5),
            expansion: 0.0,
          },
        },
        selection: egui::style::Selection {
          bg_fill: Color32::from_rgb(72, 85, 137),
          stroke: Stroke::new(1.0, Color32::from_gray(220)),
        },
        window_rounding: Rounding::from(5.0),
        resize_corner_size: 8.0,
        ..egui::Visuals::default()
      },
      // For debugging
      // debug: egui::style::DebugOptions {
      //   debug_on_hover: true,
      //   show_expand_width: true,
      //   show_expand_height: true,
      //   show_resize: true,
      // },
      ..egui::Style::default()
    });

    // Font setup
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
      "work_sans_medium".to_string(),
      egui::FontData::from_static(include_bytes!(
        "../compiletime_resources/WorkSans-Medium.ttf"
      )),
    );

    fonts.font_data.insert(
      "merriweather_regular".to_string(),
      egui::FontData::from_static(include_bytes!(
        "../compiletime_resources/Merriweather-Regular.ttf"
      )),
    );

    fonts.font_data.insert(
      "noto_mono_regular".to_string(),
      egui::FontData::from_static(include_bytes!(
        "../compiletime_resources/NotoSansMono-Regular.ttf"
      )),
    );

    fonts
      .families
      .entry(eframe::epaint::FontFamily::Proportional)
      .or_default()
      .insert(0, "work_sans_medium".into());

    fonts
      .families
      .entry(FontFamily::Name(Arc::from("Merriweather")))
      .or_default()
      .insert(0, "merriweather_regular".into());

    fonts
      .families
      .entry(FontFamily::Monospace)
      .or_default()
      .insert(0, "noto_mono_regular".into());

    ctx.set_fonts(fonts);

    // Some fields of the state do not support (de)serialization, so they must be rebuilt manually
    // Loads selected book
    if let Some(path) = &self.selected_book_path {
      if let Ok(doc) = EpubDoc::new(path) {
        self.selected_book = Some(doc);
      };
    }

    // Loads book covers
    for path in self.shelves.iter().flat_map(|g| &g.paths) {
      if let Ok(mut doc) = EpubDoc::new(path) {
        let title = doc.mdata("title").unwrap();

        if doc.get_cover().is_ok() {
          let cover = doc.get_cover().unwrap();
          let cover = RetainedImage::from_image_bytes(&title, &cover).unwrap();

          self.book_covers.insert(title, cover);
        }
      }
    }
  }

  fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
    ui::main(ctx, self);
  }

  fn save(&mut self, storage: &mut dyn epi::Storage) {
    epi::set_value(storage, epi::APP_KEY, self);
  }

  fn name(&self) -> &str {
    "Pend"
  }

  // Prevents single frame of un-layedout text
  fn warm_up_enabled(&self) -> bool {
    true
  }
}
