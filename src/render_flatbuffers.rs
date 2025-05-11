use std::{collections::HashMap, sync::MutexGuard};

use flatbuffers::{ForwardsUOffset, Vector};
use rusqlite::Connection;
use serde::de::Error;
use serde_json::json;

use crate::{db, plugin_manager::PluginManager, post::BlogPost, post_flatbuffers::pagezest_markdown::{root_as_document, Document, ListItem, Token, TokenType}};

const STYLE: &str = if cfg!(feature = "embed_styles") {
    concat!("<style>", include_str!("../assets/milligram.min.css"), "</style>")
} else {
    "<link rel=\"stylesheet\" href=\"/assets/milligram.min.css\">"
};

fn html_escape(input: &str) -> String {
  input.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

pub fn flatbuffers_to_html(post: &BlogPost, fb_input: &Vec<u8>, conn: &Connection, mut plugin_manager: MutexGuard<'_, PluginManager>) -> Result<String, serde_json::Error> {
    let doc = root_as_document(&fb_input)
        .map_err(|e| serde_json::Error::custom(e.to_string()))?;
    let mut html = String::new();
    html.push_str("<html>");
    html.push_str("<head>");
    html.push_str("<meta charset=\"utf-8\"/>");
    html.push_str(&format!("<title>{}</title>", html_escape(&post.title)));

    html.push_str(STYLE);
    html.push_str("</head>");
    html.push_str("<body>");
    if let Some(tokens) = doc.tokens() {
        for node in tokens {
            html.push_str(&render_token(&node, &doc, conn, &mut plugin_manager));
        }
    }
    html.push_str("</body>");
    html.push_str("</html>");
    Ok(html.to_string())
}

fn render_token(token: &Token, root: &Document<'_>, conn: &Connection, plugin_manager: &mut PluginManager) -> String {
    return match token.type_() {
        TokenType::PARAGRAPH => {
            let paragraph = token.value_as_paragraph().unwrap();
            if let Some(custom) = try_handle_custom_tag(paragraph.text(), root, conn, plugin_manager) {
                return custom
            } else {
                return format!("<p>{}</p>\n", render_inlines(paragraph.tokens().unwrap()))
            }
        },
        TokenType::HEADING => {
            let heading = token.value_as_heading().unwrap();
            if let Some(custom) = try_handle_custom_tag(heading.text(), root, conn, plugin_manager) {
                custom
            } else {
                format!("<h{d}>{}</h{d}>\n", render_inlines(heading.tokens()), d = heading.depth())
            }
        },
        TokenType::LIST => {
            let list = token.value_as_list().unwrap();
            let tag = if list.ordered() { "ol" } else { "ul" };
            let items_html = list.items().iter()
                .map(|item| render_list_item(&item.value_as_list_item().unwrap()))
                .collect::<Vec<_>>()
                .join("\n");
            format!("<{tag}>\n{items_html}\n</{tag}>\n")

        },
        TokenType::SPACE => "\n".to_string(),
        TokenType::CODE => {
            let code = token.value_as_code().unwrap();
            format!("<pre><code class=\"language-{}\">{}</code></pre>\n", html_escape(code.lang()), html_escape(code.text()))
        },
        TokenType::CODESPAN => {
            let codespan = token.value_as_codespan().unwrap();
            format!("<code>{}</code>\n", html_escape(codespan.text()))
        },
        TokenType::HTML => {
            let html = token.value_as_html().unwrap();
            format!("{}", html.text())
        },
        TokenType::TABLE => {
            let table = token.value_as_table().unwrap();
            let header_html = table.header().iter()
                .map(|cell| format!("<th>{}</th>", render_inlines(cell.tokens())))
                .collect::<Vec<_>>()
                .join("");
            let rows_html = table.rows().iter()
                .map(|row| {
                    let row_html = row.cells().unwrap().iter()
                        .map(|cell| format!("<td>{}</td>", render_inlines(cell.tokens())))
                        .collect::<Vec<_>>()
                        .join("");
                    format!("<tr>{}</tr>", row_html)
                })
            .collect::<Vec<_>>()
                .join("\n");
            format!("<table><thead><tr>{}</tr></thead><tbody>{}</tbody></table>\n", header_html, rows_html)
        },
        TokenType::HR => "<hr/>".to_string(),
        _ => format!("<pre>TODO N/A****: {:?}</pre>", token.type_())

    }
}

fn try_handle_custom_tag(text: &str, root: &Document<'_>, conn: &Connection, plugin_manager: &mut PluginManager) -> Option<String> {
  let trimmed = text.trim();
  if let Some(start) = trimmed.find("[[") {
    if let Some(end) = trimmed.find("]]") {
      let open_tag = &trimmed[start + 2..end];
      let (tag_name, attributes) = parse_tag_and_attributes(open_tag);
      let close_tag = format!("[[/{}]]", tag_name);
      if let Some(close_start) = trimmed.find(&close_tag) {
        let content = &trimmed[end + 2..close_start];
        return Some(handle_custom_tag(&tag_name, content, attributes, root, conn, plugin_manager))?;
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

fn handle_custom_tag(
    tag: &str, content: &str, attribs: HashMap<String, String>, root: &Document<'_>, conn: &Connection, plugin_manager: &mut PluginManager
    ) -> Option<String> {
  if tag == "custom-html" {
    return Some(content.to_string())
  }

  if tag == "posts" {
      let mut recent_posts = String::new();
      recent_posts.push_str("<h1>Recent Posts</h1>");
      recent_posts.push_str("<ul>");
      match db::get_all_post(&conn) {
        Ok(posts) => {
            for post in posts {
                let slug = post.get("slug").and_then(|s| s.as_str()).unwrap();
                let title = post.get("title").and_then(|s| s.as_str()).unwrap();
                let created_at = post.get("created_at").and_then(|s| s.as_str()).unwrap();
                if slug == "" {
                    continue;
                }
                recent_posts.push_str(&format!(
                        "<li><a href='/{}'> {} | {} </a></li>",
                        html_escape(&slug),
                        html_escape(&title),
                        html_escape(&created_at),
                        ));
            }
        },
        Err(e) => {
            recent_posts.push_str(&format!("Error getting posts: {}", e.to_string()));
        }
      }
      recent_posts.push_str("</ul>");
      return Some(recent_posts.to_string());
  }

  let plugin_input = json!({
      "content": content,
      "tag": tag,
      //"root": root,
      "attributes": attribs,
  });
  
  if plugin_manager.has_plugin_handler(tag) {
    match plugin_manager.get_plugin_by_tag(tag) {
      Ok(func) => {
        let mut func = func.lock().unwrap();
        match func.call(&plugin_input.to_string()) {
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

fn render_inlines(tokens: Vector<'_, ForwardsUOffset<Token<'_>>>) -> String {
  tokens.iter().map(|token| {
    match token.type_() {

        //TokenType::TEXT => html_escape(token.value_as_text().unwrap().text()),
        TokenType::TEXT => {
            let text = token.value_as_text().unwrap();
            if let Some(tokens) = text.tokens() {
                if tokens.len() > 0 {
                    return format!("{}", render_inlines(tokens))
                }
            }
            format!("{}", html_escape(text.text()))
        },
        TokenType::STRONG => format!("<strong>{}</strong>", render_inlines(token.value_as_strong().unwrap().tokens())),
        TokenType::DEL => format!("<del>{}</del>", render_inlines(token.value_as_del().unwrap().tokens())),
        TokenType::EM => format!("<em>{}</em>", render_inlines(token.value_as_em().unwrap().tokens())),
        TokenType::CODE => {
            let code = token.value_as_code().unwrap();
            format!("<pre><code class=\"language-{}\">{}</code></pre>\n", html_escape(code.lang()), html_escape(code.text()))
        },
        TokenType::CODESPAN => {
            let codespan = token.value_as_codespan().unwrap();
            format!("<code>{}</code>\n", html_escape(codespan.text()))
        },
        TokenType::LINK => {
            let link = token.value_as_link().unwrap();
            if let Some(title) = link.title() {
                format!("<a href=\"{}\" title=\"{}\">{}</a>", html_escape(link.href().unwrap()), html_escape(title), render_inlines(link.tokens()))
            } else {
                format!("<a href=\"{}\">{}</a>", html_escape(link.href().unwrap()), render_inlines(link.tokens()))
            }
        },
        TokenType::HTML => {
            let html = token.value_as_html().unwrap();
            format!("{}", html.text())
        },
        TokenType::IMAGE => {
            let img = token.value_as_image().unwrap();
            if let Some(title) = img.title() {
                format!("<img src=\"{}\" title=\"{}\" alt=\"{}\"</>", html_escape(img.href().unwrap()), html_escape(title), render_inlines(img.tokens()))
            } else {
                format!("<img src=\"{}\" />", html_escape(img.href().unwrap()))
            }
        },
        TokenType::UNKNOWN => "<h2>UNKNOWN</h2>".to_string(),
        TokenType::TABLE => {
            let table = token.value_as_table().unwrap();
            let header_html = table.header().iter()
                .map(|cell| format!("<th>{}</th>", render_inlines(cell.tokens())))
                .collect::<Vec<_>>()
                .join("");
            let rows_html = table.rows().iter()
                .map(|row| {
                    let row_html = row.cells().unwrap().iter()
                        .map(|cell| format!("<td>{}</td>", render_inlines(cell.tokens())))
                        .collect::<Vec<_>>()
                        .join("");
                    format!("<tr>{}</tr>", row_html)
                })
            .collect::<Vec<_>>()
                .join("\n");
            format!("<table><thead><tr>{}</tr></thead><tbody>{}</tbody></table>\n", header_html, rows_html)
        },
        TokenType::LIST => {
            let list = token.value_as_list().unwrap();
            let tag = if list.ordered() { "ol" } else { "ul" };
            let items_html = list.items().iter()
                .map(|item| render_list_item(&item.value_as_list_item().unwrap()))
                .collect::<Vec<_>>()
                .join("\n");
            format!("<{tag}>\n{items_html}\n</{tag}>\n")

        },
        TokenType::HR => "<hr/>".to_string(),
        TokenType::LIST_ITEM => "<h1>LIST ITEM</h1>".to_string(),
        _ => format!("TODO: {:?}", token.type_()),

    }
  }).collect::<Vec<_>>().join("")
}

fn render_list_item(item: &ListItem) -> String {
    let task_input = if item.task() {
        &format!("<input type=\"checkbox\" {}/>", if item.checked() { "checked" } else {""})
    } else {
        ""
    };

    format!("<li>{}{}</li>", task_input, render_inlines(item.tokens().unwrap()))
}
