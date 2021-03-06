#[cfg(all(not(debug_assertions), not(target_arch = "wasm32")))]
use crate::backend::load_directory;
use crate::ui::{
  BookTextStyle, DocumentColors, Note, PanelState, UIState, BLUISH,
  DARKISH_BLUISH, DARK_BLUISH, LIGHTISH_BLUISH, LIGHT_BLUISH,
};
use crate::{
  backend::{LocalBookInfo, Shelf},
  ui,
};
use eframe::{
  egui::{self, style::WidgetVisuals, FontDefinitions},
  epaint::{FontFamily, Rounding},
  epi,
};
use egui::{vec2, Color32, Stroke, Vec2};
use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::Cursor;
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct Pend {
  pub ui_state: UIState,
  pub library_path: String,
  pub shelves: Vec<Shelf>,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  pub epub_cache: HashMap<String, EpubDoc<Cursor<Vec<u8>>>>,
  pub shelf_search: String,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  pub book_covers: HashMap<String, RetainedImage>,
  pub selected_book_uuid: Option<String>,
  pub book_style: BookTextStyle,
  pub book_userdata: HashMap<String, LocalBookInfo>,
  pub goto_target: Option<Note>,
  pub theme: DocumentColors,
  pub book_cover_width_multiplier: f32,
  /// UUID original shelf name, title
  pub dragged_book: Option<(String, String, String)>,
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
      epub_cache: HashMap::new(),
      shelf_search: String::new(),
      book_covers: HashMap::new(),
      selected_book_uuid: None,
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

    // Load local book directory (only in native && release mode)
    #[cfg(all(not(debug_assertions), not(target_arch = "wasm32")))]
    load_directory(self, self.library_path.clone());

    #[cfg(target_arch = "wasm32")]
    {
      self.shelves.clear();
      self.book_userdata.clear();
      self.book_covers.clear();
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

  // Controls the maximum size of the web canvas
  // This may cause serious performance issues on Linux / MacOS
  // Set it to something smaller (ie. 1280x1280 to mitigate)
  fn max_size_points(&self) -> Vec2 {
    vec2(f32::MAX, f32::MAX)
  }
}

impl Pend {
  pub fn remove_book<U: Into<String> + Display>(&mut self, uuid: U) {
    let uuid = uuid.to_string();
    // Remove actual epub
    self.epub_cache.retain(|u, _| *u != uuid);
    // Remove uuid from shelves
    for shelf in self.shelves.iter_mut() {
      shelf.uuids.retain(|u| *u != uuid);
    }
    // Remove cover
    self.book_covers.remove(&uuid);
  }
}
