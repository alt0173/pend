use std::{collections::HashMap, path::PathBuf};

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

#[derive(Serialize, Deserialize, Clone, PartialOrd, Eq, Ord)]
pub struct PathGroup {
  pub name: String,
  pub paths: Vec<PathBuf>,
  pub renaming: bool,
  pub desired_name: String,
}

impl PartialEq for PathGroup {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

impl PathGroup {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      paths: Vec::new(),
      renaming: false,
      desired_name: String::new(),
    }
  }

  /// Removes an entry in paths by path
  pub fn remove_path(&mut self, path: PathBuf) {
    if let Ok(index) = self.paths.binary_search(&path) {
      self.paths.remove(index);
    }
  }
}

#[derive(PartialEq, Clone)]
pub struct DraggedBook {
  pub path: PathBuf,
  pub title: String,
  pub source_shelf_name: String,
}

impl DraggedBook {
  pub fn new(path: PathBuf, title: String, source_shelf_title: String) -> Self {
    Self {
      path,
      title,
      source_shelf_name: source_shelf_title,
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
          x if x.contains('h') => Some(FormattingInfo::Heading),
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
    // Fallback image
    if !state.book_covers.contains_key("fallback") {
      state.book_covers.insert(
        "fallback".to_string(),
        RetainedImage::from_image_bytes(
          "fallback",
          include_bytes!("../compiletime_resources/fallback.png"),
        )
        .unwrap(),
      );
    }

    // Create a default "folder" / PathGroup if one is not already present
    if state.shelf.is_empty() {
      state.shelf.push(PathGroup::new("Books"));
    }

    // Add file to library if not already added
    if !state
      .shelf
      .iter()
      .flat_map(|g| g.paths.clone())
      .any(|x| x == file_path)
    {
      state.shelf[0].paths.push(file_path.clone());
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
