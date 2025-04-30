use std::{collections::HashMap, sync::MutexGuard};

use serde::{ser, Deserialize};
use serde_json::{json, Value};

use crate::plugin_manager::PluginManager;

const STYLE: &str = include_str!("../assets/milligram.min.css");

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum InlineToken {
  #[serde(rename = "text")]
  Text { text: String },
  #[serde(rename = "strong")]
  Strong { tokens: Vec<InlineToken> },
  #[serde(rename = "em")]
  Em { tokens: Vec<InlineToken> },
  #[serde(rename = "codespan")]
  Codespan { text: String },
  #[serde(rename = "link")]
  Link { href: String, title: Option<String>, tokens: Vec<InlineToken> },
  #[serde(other)]
  Other,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Block {
  #[serde(rename = "space")]
  Space,
  #[serde(rename = "paragraph")]
  Paragraph {
    text: String,
    tokens: Vec<InlineToken>,
  },
  #[serde(rename = "heading")]
  Heading {
    depth: u8,
    text: String,
    tokens: Vec<InlineToken>,
  },
  #[serde(rename = "list")]
  List {
    ordered: bool,
    items: Vec<ListItem>,
  },
  #[serde(rename = "list_item")]
  ListItem {
    text: String,
    tokens: Vec<InlineToken>,
  },
  #[serde(rename = "code")]
  Code {
    text: String,
    lang: Option<String>,
  },
  #[serde(rename = "blockquote")]
  Blockquote {
    tokens: Vec<Block>,
  },
  #[serde(rename = "hr")]
  Hr,
  #[serde(rename = "html")]
  Html {
    text: String,
  },
  #[serde(rename = "table")]
  Table {
    header: Vec<TableCell>,
    rows: Vec<Vec<TableCell>>,
  },
  #[serde(other)]
  Other,
}

#[derive(Debug, Deserialize)]
pub struct ListItem {
  text: String,
  tokens: Vec<InlineToken>,
}

#[derive(Debug, Deserialize)]
pub struct TableCell {
  text: String,
}

#[derive(Debug, Deserialize)]
pub struct TableRow {
  cells: Vec<TableCell>,
}

pub fn json_to_html(json_input: &str, mut plugin_manager: MutexGuard<'_, PluginManager>) -> Result<String, serde_json::Error> {
  let root: Value = serde_json::from_str(json_input).unwrap();
  let blocks: Vec<Block> = serde_json::from_str(json_input)?;
  let mut html = String::new();
  html.push_str("<html>");
  html.push_str("<head>");

  html.push_str("<style>");
  html.push_str(STYLE);
  html.push_str("</style>");
  html.push_str("</head>");
  html.push_str("<body>");
  for block in blocks {
    html.push_str(&render_block(&block, &root, &mut plugin_manager));
  }

  html.push_str("</body>");
  html.push_str("</html>");
  Ok(html.to_string())
}

fn render_block(block: &Block, root: &Value, plugin_manager: &mut PluginManager) -> String {
  match block {
    Block::Space => "\n".to_string(),
    Block::Paragraph { tokens, text } => {
      if let Some(custom) = try_handle_custom_tag(text, root, plugin_manager) {
        custom
      } else {
        format!("<p>{}</p>\n", render_inlines(tokens))
      }
    }
    Block::Heading { depth, tokens, text } => {
      if let Some(custom) = try_handle_custom_tag(text, root, plugin_manager) {
        custom
      } else {
        format!("<h{d}>{}</h{d}>\n", render_inlines(tokens), d = depth)
      }
    }
    Block::List { ordered, items } => {
      let tag = if *ordered { "ol" } else { "ul" };
      let items_html = items.iter()
        .map(|item| format!("<li>{}</li>", render_inlines(&item.tokens)))
        .collect::<Vec<_>>()
        .join("\n");
        format!("<{tag}>\n{items_html}\n</{tag}>\n")
    }
    Block::ListItem { tokens, .. } => {
      format!("<li>{}</li>\n", render_inlines(tokens))
    }
    Block::Code { text, lang } => {
      if let Some(lang) = lang {
        format!("<pre><code class=\"language-{}\">{}</code></pre>\n", html_escape(lang), html_escape(text))
      } else {
        format!("<pre><code>{}</code></pre>\n", html_escape(text))
      }
    }
    Block::Blockquote { tokens } => {
      let content = tokens.iter()
        .map(|x| render_block(x, root, plugin_manager))
        .collect::<Vec<_>>()
        .join("\n");
        format!("<blockquote>\n{}</blockquote>\n", content)
    }
    Block::Hr => "<hr />\n".to_string(),
    Block::Html { text } => format!("{}\n", text),
    Block::Table { header, rows } => {
      let header_html = header.iter()
        .map(|cell| format!("<th>{}</th>", html_escape(&cell.text)))
        .collect::<Vec<_>>()
        .join("");
        let rows_html = rows.iter()
          .map(|row| {
            let row_html = row.iter()
              .map(|cell| format!("<td>{}</td>", html_escape(&cell.text)))
              .collect::<Vec<_>>()
              .join("");
              format!("<tr>{}</tr>", row_html)
          })
        .collect::<Vec<_>>()
          .join("\n");
        format!("<table><thead><tr>{}</tr></thead><tbody>{}</tbody></table>\n", header_html, rows_html)
    }

    Block::Other => "".to_string(),
  }
}

fn render_inlines(tokens: &[InlineToken]) -> String {
  tokens.iter().map(|token| {
    match token {
      InlineToken::Text { text } => html_escape(text),
      InlineToken::Strong { tokens } => format!("<strong>{}</strong>", render_inlines(tokens)),
      InlineToken::Em { tokens } => format!("<em>{}</em>", render_inlines(tokens)),
      InlineToken::Codespan { text } => format!("<code>{}</code>", html_escape(text)),
      InlineToken::Link { href, title, tokens } => {
        if let Some(title) = title {
          format!("<a href=\"{}\" title=\"{}\">{}</a>", html_escape(href), html_escape(title), render_inlines(tokens))
        } else {
          format!("<a href=\"{}\">{}</a>", html_escape(href), render_inlines(tokens))
        }
      }
      InlineToken::Other => "".to_string(),
    }
  }).collect::<Vec<_>>().join("")
}

fn html_escape(input: &str) -> String {
  input.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

fn try_handle_custom_tag(text: &str, root: &Value, plugin_manager: &mut PluginManager) -> Option<String> {
  let trimmed = text.trim();
  if let Some(start) = trimmed.find("[[") {
    if let Some(end) = trimmed.find("]]") {
      let open_tag = &trimmed[start + 2..end];
      let (tag_name, attributes) = parse_tag_and_attributes(open_tag);
      let close_tag = format!("[[/{}]]", tag_name);
      if let Some(close_start) = trimmed.find(&close_tag) {
        let content = &trimmed[end + 2..close_start];
        return Some(handle_custom_tag(&tag_name, content, attributes, root, plugin_manager))?;
      }
    }
  }
  None
}

fn parse_tag_and_attributes(tag: &str) -> (String, HashMap<String, String>) {
  let mut chars = tag.chars().peekable();
  let mut tag_name = String::new();
  let mut attributes = HashMap::new();

  // Parse tag name
  while let Some(&ch) = chars.peek() {
    if ch.is_whitespace() {
      break;
    }
    tag_name.push(ch);
    chars.next();
  }

  // Skip whitespace after tag name
  while let Some(&ch) = chars.peek() {
    if !ch.is_whitespace() {
      break;
    }
    chars.next();
  }

  // Parse attributes
  while let Some(_) = chars.peek() {
    let mut key = String::new();
    let mut value = String::new();

    // Parse key
    while let Some(&ch) = chars.peek() {
      if ch == '=' {
        chars.next();
        break;
      }
      key.push(ch);
      chars.next();
    }

    // Parse value
    if let Some(&ch) = chars.peek() {
      if ch == '"' {
        chars.next(); // skip opening quote
        while let Some(ch) = chars.next() {
          if ch == '"' {
            break;
          }
          value.push(ch);
        }
      }
    }

    if !key.is_empty() {
      attributes.insert(key, value);
    }

    // Skip whitespace before next attribute
    while let Some(&ch) = chars.peek() {
      if !ch.is_whitespace() {
        break;
      }
      chars.next();
    }
  }

  (tag_name, attributes)
}

fn handle_custom_tag(tag: &str, content: &str, attribs: HashMap<String, String>, root: &Value, plugin_manager: &mut PluginManager) -> Option<String> {
  if tag == "custom-html" {
    return Some(content.to_string())
  }
  let plugin_input = json!({
    "tag": tag,
    "root": root,
    "attributes": attribs,
  });
  
  if plugin_manager.has_plugin_handler(tag) {
    match plugin_manager.get_plugin_by_tag(tag) {
      Ok(func) => {
        println!("rendering custom tag: {}", tag);
        match func.borrow_mut().call_in_new_context(&plugin_input.to_string()) {
          Ok(v) => {
            return Some(v)
          },
          Err(e) => {
            println!("plugin call error: {}", e.to_string());
            return Some("Plugin error".to_string())
          },
        }
      },
      _ => {
        return Some("could not call function".to_string())
      }
    }
  }
  println!("no handler for: {}", tag);
  Some(format!("custom tag[{}]: {}", tag, html_escape(content)))
}

/*
#[cfg(feature_reusable_vm)]
fn handle_custom_tag(tag: &str, content: &str, attribs: HashMap<String, String>, root: Value, plugin_manager: &mut PluginManager) -> Option<String> {
  if tag == "custom-html" {
    return Some(content.to_string())
  }
  let plugin_input = json!({
    "tag": tag,
    "root": root,
    "attributes": attribs,
  });
  if plugin_manager.has_plugin_handler(tag) {
    match plugin_manager.get_plugin_by_tag(tag) {
      Ok(func) => {
        println!("rendering custom tag: {}", tag);
        match func.borrow_mut().call(&plugin_input.to_string()) {
          Ok(v) => {
            return Some(v)
          },
          Err(e) => {
            println!("plugin call error: {}", e.to_string());
            return Some("Plugin error".to_string())
          },
        }
      },
      _ => {
        return Some("could not call function".to_string())
      }
    }
  }
  println!("no handler for: {}", tag);
  Some(format!("custom tag[{}]: {}", tag, html_escape(content)))
}
*/
