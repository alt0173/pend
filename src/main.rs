#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

mod backend;
mod panels;
pub use crate::panels::reader;
mod ui;
use backend::{LocalBookInfo, PathGroup};
use eframe::{
  egui::{self, style::WidgetVisuals, FontDefinitions},
  epaint::{FontFamily, Rounding},
  epi, run_native, NativeOptions,
};
use egui::vec2;
use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::PathBuf, sync::Arc};
use ui::{main_ui, BookTextStyle, DocumentColors, Note, PanelState, UIState};

#[derive(Serialize, Deserialize)]
pub struct MyApp {
  ui_state: UIState,
  library_path: String,
  shelves: Vec<PathGroup>,
  shelf_search: String,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  book_covers: HashMap<String, RetainedImage>,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  selected_book: Option<EpubDoc<File>>,
  selected_book_path: Option<PathBuf>,
  chapter_number: usize,
  book_style: BookTextStyle,
  book_userdata: HashMap<PathBuf, LocalBookInfo>,
  goto_target: Option<Note>,
  theme: DocumentColors,
  book_cover_width: f32,
  /// Path, title
  dragged_book: Option<(PathBuf, String)>,
}

impl Default for MyApp {
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
      chapter_number: 0,
      book_style: BookTextStyle::default(),
      book_userdata: HashMap::new(),
      goto_target: None,
      theme: DocumentColors::default(),
      book_cover_width: 140.0,
      dragged_book: None,
    }
  }
}

impl epi::App for MyApp {
  fn setup(
    &mut self,
    ctx: &egui::Context,
    _frame: &epi::Frame,
    _storage: Option<&dyn epi::Storage>,
  ) {
    // Load memory
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
      // text_styles: todo!(),
      // wrap: todo!(),
      // spacing: todo!(),
      // interaction: todo!(),
      visuals: egui::Visuals {
        widgets: egui::style::Widgets {
          noninteractive: WidgetVisuals {
            rounding: Rounding::from(1.5),
            ..ctx.style().visuals.widgets.noninteractive
          },
          inactive: WidgetVisuals {
            rounding: Rounding::from(1.5),
            ..ctx.style().visuals.widgets.inactive
          },
          hovered: WidgetVisuals {
            rounding: Rounding::from(1.5),
            ..ctx.style().visuals.widgets.hovered
          },
          active: WidgetVisuals {
            rounding: Rounding::from(1.5),
            ..ctx.style().visuals.widgets.active
          },
          open: WidgetVisuals {
            rounding: Rounding::from(1.5),
            ..ctx.style().visuals.widgets.open
          },
        },
        // selection: todo!(),
        // hyperlink_color: todo!(),
        window_rounding: Rounding::from(5.0),
        // window_shadow: todo!(),
        resize_corner_size: 8.0,
        ..Default::default()
      },
      // debug: egui::style::DebugOptions {
      // 	debug_on_hover: true,
      // 	show_expand_width: true,
      // 	show_expand_height: true,
      // 	show_resize: true,
      // },
      ..Default::default()
    });

    // Font setup
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
      "work_sans".into(),
      egui::FontData::from_static(include_bytes!(
        "../compiletime_resources/WorkSans-Medium.ttf"
      )),
    );

    fonts.font_data.insert(
      "merriweather_regular".into(),
      egui::FontData::from_static(include_bytes!(
        "../compiletime_resources/Merriweather-Regular.ttf"
      )),
    );

    fonts
      .families
      .entry(eframe::epaint::FontFamily::Proportional)
      .or_default()
      .insert(0, "work_sans".into());

    fonts
      .families
      .entry(FontFamily::Name(Arc::from("Merriweather")))
      .or_default()
      .insert(0, "merriweather_regular".into());

    ctx.set_fonts(fonts);

    // Some fields of the program state do not support (de)serialization, so they must be rebuilt manually
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
    main_ui(ctx, self);
  }

  fn save(&mut self, storage: &mut dyn epi::Storage) {
    epi::set_value(storage, epi::APP_KEY, self);
  }

  // Name of the process
  fn name(&self) -> &str {
    "Swag book reading software beta 0.1"
  }
  // Prevents single instance of un-layedout text
  fn warm_up_enabled(&self) -> bool {
    true
  }
}

fn main() {
  let app = MyApp {
    ..Default::default()
  };
  let native_options = NativeOptions {
    min_window_size: Some(vec2(960.0, 540.0)),
    ..Default::default()
  };
  run_native(Box::new(app), native_options)
}
