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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenameState {
  Active,
  Inactive,
  Error,
}

#[derive(Serialize, Deserialize, Clone, PartialOrd, Eq, Ord)]
pub struct PathGroup {
  pub name: String,
  pub paths: Vec<PathBuf>,
  pub renaming: RenameState,
  pub desired_name: String,
}

impl PartialEq for PathGroup {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

impl PathGroup {
  pub fn new<S: Into<String>>(name: S) -> Self {
    Self {
      name: name.into(),
      paths: Vec::new(),
      renaming: RenameState::Inactive,
      desired_name: String::new(),
    }
  }

  pub fn new_with_contents<S: Into<String>>(
    name: S,
    paths: Vec<PathBuf>,
  ) -> Self {
    Self {
      name: name.into(),
      paths,
      renaming: RenameState::Inactive,
      desired_name: String::new(),
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

    // Processes the parsed HTML, (Using my custom, bad, parsing, which is very
    // imperfect to say the least) and converts it into formatting info
    for captures in rx.captures_iter(line) {
      for capture in captures.iter().flatten() {
        if let Some(format) = match capture.as_str() {
          x if x.contains("title") => Some(FormattingInfo::Title),
          x if x.contains("<h1") => Some(FormattingInfo::Heading),
          x if x.contains("<h2") => Some(FormattingInfo::Heading2),
          x if x.contains("<i") => Some(FormattingInfo::Italic),
          x if x.contains("<b") => Some(FormattingInfo::Bold),
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

    if processed.is_empty() {
      lines_removed += 1;
    } else {
      output.push_str(processed);
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
    if state.shelves.is_empty() {
      state.shelves.push(PathGroup::new("Books"));
    }

    // Add file to library if not already added
    if !state
      .shelves
      .iter()
      .flat_map(|g| g.paths.clone())
      .any(|x| x == file_path)
    {
      state.shelves[0].paths.push(file_path.clone());
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
