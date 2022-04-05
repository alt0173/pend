use std::{collections::HashMap, fmt::Display, fs, io::Cursor};

use egui::Color32;
use egui_extras::RetainedImage;
use epub::doc::EpubDoc;
use glob::glob;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{ui::Note, Pend};

/// Denotes type of formatting to be applied to a line group
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FormattingInfo {
  Title,
  Heading,
  Heading2,
  Bold,
  Italic,
}

/// Contains custom content a user creates for each book (notes, highlighted lines, etc.)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalBookInfo {
  pub notes: Vec<Note>,
  /// Last page the user viewed
  pub chapter: usize,
  /// (Chapter, Line), Color of the highlight
  pub highlights: HashMap<(usize, usize), Color32>,
  /// (Chapter, Line), info
  pub formatting_info: HashMap<(usize, usize), FormattingInfo>,
}

impl LocalBookInfo {
	#[must_use]
  pub fn default() -> Self {
    Self {
      notes: Vec::new(),
      chapter: 1,
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

/// Group of books uuids for lookup, with some metadata.
///
/// Note that when using `PartialEq`, only the `name` field is compared
#[derive(Serialize, Deserialize, Clone, PartialOrd, Eq, Ord)]
pub struct Shelf {
  pub name: String,
  // Unique identifiers of epubs in this group,
  // used to look them up in the cache
  pub uuids: Vec<String>,
  pub renaming: RenameState,
  pub desired_name: String,
}

impl PartialEq for Shelf {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

impl Shelf {
  pub fn new<S: Into<String>>(name: S) -> Self {
    Self {
      name: name.into(),
      uuids: Vec::new(),
      renaming: RenameState::Inactive,
      desired_name: String::new(),
    }
  }

  pub fn new_with_contents<S: Into<String>>(
    name: S,
    uuids: Vec<String>,
  ) -> Self {
    Self {
      name: name.into(),
      uuids,
      renaming: RenameState::Inactive,
      desired_name: String::new(),
    }
  }
}

/// Turns calibre html into usable text / formatting info
pub fn parse_calibre(
  input: &str,
  chapter: usize,
  book_info: &mut LocalBookInfo,
) -> String {
  let mut output = String::new();
  let mut lines_removed = 0;

  for (line_number, line) in input.lines().enumerate() {
    let rx = Regex::new(r"<(.*?)>").unwrap();

    // Best I could come up with for handling breaks; improve this later
    let line = line.replace("<br/>", "  [break]  ");
    let processed = rx.replace_all(&line, "");
    let processed_line = processed.trim();

    // Processes the parsed HTML using my custom parsing (could be improved ;)
    // and converts it into formatting info
    if !processed_line.is_empty() {
      for captures in rx.captures_iter(&line) {
        for capture in captures.iter().flatten() {
          if let Some(capture) = match capture.as_str() {
            // Not (currently) implimented and/or explicitly nothing
            // This avoids `<img` being read as `<i` for example
            x if x.contains("<img") => None,
            x if x.contains("<body") => None,
            x if x.contains("<p>") => None,
            // Working formatting
            x if x.contains("<title") => Some(FormattingInfo::Title),
            x if x.contains("<h1") => Some(FormattingInfo::Heading),
            x if x.contains("<h2") => Some(FormattingInfo::Heading2),
            x if x.contains("<i") => Some(FormattingInfo::Italic),
            x if x.contains("<b") => Some(FormattingInfo::Bold),
            _ => None,
          } {
            book_info
              .formatting_info
              .insert((chapter, line_number - lines_removed), capture);
          }
        }
      }
    }

    if processed_line.is_empty() {
      lines_removed += 1;
    } else {
      output.push_str(processed_line);
      output.push('\n');
    }
  }

  output
}

/// Loads all epubs in a given directory (and all subfolders)
pub fn load_directory<P: Into<String> + Display>(
  state: &mut Pend,
  directory: P,
) {
  // Finds all epub files in the user's library directory
  for file_path in glob(&format!("{}/**/*.epub", directory)).unwrap().flatten()
  {
    let epub =
      EpubDoc::from_reader(Cursor::new(fs::read(file_path).unwrap())).unwrap();

    register_epub(state, epub);
  }
}

/// Performs the neccesary steps to load an epub into the program and set up
/// metadata / cover / etc
pub fn register_epub(state: &mut Pend, mut epub: EpubDoc<Cursor<Vec<u8>>>) {
  let uuid = epub.unique_identifier.as_ref().unwrap().clone();

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
    state.shelves.push(Shelf::new("Books"));
  }

  // Add book cover to cache of book covers
  if let Ok(cover) = epub.get_cover() {
    state.book_covers.entry(uuid.clone()).or_insert_with(|| {
      RetainedImage::from_image_bytes(&uuid, &cover).unwrap()
    });
  }

  // Add file uuid to library if not already added
  if !state
    .shelves
    .iter()
    .flat_map(|g| g.uuids.clone())
    .any(|x| x == uuid)
  {
    state.shelves[0].uuids.push(uuid.clone());

    state.epub_cache.insert(uuid.clone(), epub);
  }

  // If the book in question does not have userdata already: create an empty
  state
    .book_userdata
    .entry(uuid)
    .or_insert_with(|| LocalBookInfo::default());
}
