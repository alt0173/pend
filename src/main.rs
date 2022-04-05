#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

use eframe::{egui, epi::IconData, run_native, NativeOptions};
use egui::vec2;
use pend::app::Pend;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
  let app = Pend::default();
  let native_options = NativeOptions {
    min_window_size: Some(vec2(960.0, 540.0)),
    drag_and_drop_support: true,
    icon_data: Some(IconData {
      rgba: image::load_from_memory(include_bytes!(
        "../compiletime_resources/pend.png"
      ))
      .unwrap()
      .into_bytes(),
      width: 64,
      height: 64,
    }),
    ..eframe::NativeOptions::default()
  };
  run_native(Box::new(app), native_options)
}
