use regex::Regex;

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
