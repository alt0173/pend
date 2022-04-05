#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

pub mod app;
pub mod backend;
pub mod panels;
pub mod ui;
use app::Pend;

/// WASM entry point
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
  // Make sure panics are logged using `console.error`.
  console_error_panic_hook::set_once();

  // Redirect tracing to console.log and friends:
  tracing_wasm::set_as_global_default();

  let app = Pend::default();
  eframe::start_web(canvas_id, Box::new(app))
}
