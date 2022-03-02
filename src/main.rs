#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

mod backend;
mod ui;

use std::{collections::HashMap, fs::File, path::PathBuf, sync::Arc};

use backend::UserBookInfo;
use eframe::{
  egui::{self, style::WidgetVisuals, FontDefinitions},
  epaint::{FontFamily, Rounding},
  epi, run_native, NativeOptions,
};
use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use ui::{main_ui, BookTextStyle, Note, PanelState, UIState, ThemeInfo};

#[derive(Serialize, Deserialize)]
pub struct MyApp {
  ui_state: UIState,
  library: Vec<PathBuf>,
  library_path: String,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  book_covers: HashMap<String, RetainedImage>,
  #[serde(skip_serializing)]
  #[serde(skip_deserializing)]
  selected_book: Option<EpubDoc<File>>,
  selected_book_path: Option<PathBuf>,
  chapter_number: usize,
  book_style: BookTextStyle,
  book_userdata: HashMap<PathBuf, UserBookInfo>,
  goto_target: Option<Note>,
	theme: ThemeInfo,
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      ui_state: UIState {
        left_panel_state: PanelState::Library,
        right_panel_state: PanelState::Reader,
        display_ofl_popup: false,
        display_raw_text: false,
      },
      library: Vec::new(),
      library_path: "./library".into(),
      book_covers: HashMap::new(),
      selected_book: None,
      selected_book_path: None,
      chapter_number: 0,
      book_style: BookTextStyle {
        font_family: FontFamily::Name("Merriweather".into()),
        line_spacing_multiplier: 1.0,
        ..Default::default()
      },
      book_userdata: HashMap::new(),
      goto_target: None,
			theme: ThemeInfo::default(),
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
    for path in &self.library {
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
    "Lisci"
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
    // decorated: todo!(),
    // drag_and_drop_support: todo!(),
    // icon_data: todo!(),
    // initial_window_size: todo!(),
    ..Default::default()
  };
  run_native(Box::new(app), native_options)
}
