use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use glob::glob;
use regex::Regex;

use crate::MyApp;

pub fn parse_calibre(input: &str) -> String {
  let mut output = String::new();

  for line in input.lines() {
    let rx = Regex::new(r"<.*?>").unwrap();

    let processed = rx.replace_all(line, "");
    let processed = processed.trim();

    if processed != "" {
      output.push_str(&processed);
      output.push('\n');
    }
  }

  output
}

pub fn load_library(state: &mut MyApp) {
  // Finds all epub files in the user's library directory
  for file_path in glob(&format!("{}/**/*.epub", state.library_path))
    .unwrap()
    .flatten()
  {
    // Add file to library if not already added
    if !state.library.contains(&file_path) {
      state.library.push(file_path.clone());
      println!("{:?}", &file_path);
    }
    // Same thing for the book cover
    let mut doc = EpubDoc::new(&file_path).unwrap();
    let title = doc.mdata("title").unwrap();

    if doc.get_cover().is_ok() {
      let cover = doc.get_cover().unwrap();
      let cover = RetainedImage::from_image_bytes(&title, &cover).unwrap();

      state.book_covers.insert(title, cover);
    }

    // If the book in question does not have a vec of notes already: create one
    if !state.notes.contains_key(&file_path) {
      state.notes.insert(file_path, Vec::new());
    }
  }
}
