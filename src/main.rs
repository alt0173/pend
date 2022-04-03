#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

mod app;
mod backend;
mod panels;
pub use crate::panels::reader;
mod ui;

use app::Pend;
use eframe::{egui, epi::IconData, run_native, NativeOptions};
use egui::vec2;

fn main() {
  let app = Pend { ..Pend::default() };
  let native_options = NativeOptions {
    min_window_size: Some(vec2(960.0, 540.0)),
    icon_data: Some(IconData {
      rgba: image::load_from_memory(include_bytes!("../compiletime_resources/pend.png")).unwrap().into_bytes(),
      width: 64,
      height: 64,
    }),
    ..eframe::NativeOptions::default()
  };
  run_native(Box::new(app), native_options)
}
