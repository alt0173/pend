use std::collections::HashMap;

use egui::Color32;
use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use glob::glob;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{ui::Note, MyApp};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FormattingInfo {
  Title,
  Heading,
  Heading2,
  Bold,
  Italic,
}

// Contains custom content a user creates for each book (notes, highlighted lines, etc.)
#[derive(Serialize, Deserialize, Debug)]
pub struct LocalBookInfo {
  pub notes: Vec<Note>,
  /// (Chapter, Line), Color of the highlight
  pub highlights: HashMap<(usize, usize), Color32>,
  /// (Chapter, Line), info
  pub formatting_info: HashMap<(usize, usize), FormattingInfo>,
}

impl LocalBookInfo {
  pub fn new() -> Self {
    Self {
      notes: Vec::new(),
      highlights: HashMap::new(),
      formatting_info: HashMap::new(),
    }
  }
}

pub fn parse_calibre(
  input: &str,
  chapter: usize,
  book_info: &mut LocalBookInfo,
) -> String {
  let mut output = String::new();
  let mut lines_removed = 0;

  for (line_number, line) in input.lines().enumerate() {
    let rx = Regex::new(r"<(.*?)>").unwrap();

    // Important
    for captures in rx.captures_iter(line) {
      for capture in captures.iter().flatten() {
        if let Some(format) = match capture.as_str() {
          x if x.contains("title") => Some(FormattingInfo::Title),
          x if x.contains("h") => Some(FormattingInfo::Heading),
          x if x.contains("h2") => Some(FormattingInfo::Heading),
          _ => None,
        } {
          book_info
            .formatting_info
            .insert((chapter, line_number - lines_removed), format);
        }
      }
    }

    let processed = rx.replace_all(line, "");
    let processed = processed.trim();

    if !processed.is_empty() {
      output.push_str(processed);
      output.push('\n');
    } else {
      lines_removed += 1;
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
    }
    // Same thing for the book cover
    let mut doc = EpubDoc::new(&file_path).unwrap();
    let title = doc.mdata("title").unwrap();

    if doc.get_cover().is_ok() {
      let cover = doc.get_cover().unwrap();
      let cover = RetainedImage::from_image_bytes(&title, &cover).unwrap();

      state.book_covers.insert(title, cover);
    }

    // If the book in question does not have userdata already: create some
    state
      .book_userdata
      .entry(file_path)
      .or_insert_with(LocalBookInfo::new);
  }
}
