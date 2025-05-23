use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use serde::de::Error;

use crate::{
    db::{self, DBPool},
    plugin_manager::PluginManager,
    post::BlogPost,
    post_flatbuffers::pagezest_markdown::{
        root_as_document, Document, DocumentArgs, Html, HtmlArgs, ListItem, Paragraph,
        ParagraphArgs, Token, TokenArgs, TokenType, TokenValue,
    },
};

pub enum RenderTokenResult {
    HTML(String),
    CustomTag,
}

const STYLE: &str = if cfg!(feature = "embed_styles") {
    concat!(
        "<style>",
        include_str!("../assets/milligram.min.css"),
        "</style>"
    )
} else {
    "<link rel=\"stylesheet\" href=\"/assets/milligram.min.css\">"
};

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn flatbuffers_prerender(
    _post: &BlogPost,
    fb_input: &Vec<u8>,
) -> Result<Vec<u8>, serde_json::Error> {
    let doc = root_as_document(&fb_input).map_err(|e| serde_json::Error::custom(e.to_string()))?;

    let mut curr_html = String::new();
    let mut fbb = FlatBufferBuilder::default();
    let mut token_buf: Vec<WIPOffset<Token>> = Vec::new();
    if let Some(tokens) = doc.tokens() {
        for node in tokens {
            match pre_render_token(&node) {
                RenderTokenResult::HTML(html) => {
                    curr_html.push_str(&html);
                }
                RenderTokenResult::CustomTag => {
                    let custom_tag_tokens: Vec<WIPOffset<Token>> = Vec::new();
                    let custom_tag_tokens = fbb.create_vector(&custom_tag_tokens);
                    let custom_tag_text =
                        fbb.create_string(node.value_as_paragraph().unwrap().text());
                    let custom_tag_elm = Paragraph::create(
                        &mut fbb,
                        &ParagraphArgs {
                            text: Some(custom_tag_text),
                            tokens: Some(custom_tag_tokens),
                        },
                    );
                    let custom_tag_token = Token::create(
                        &mut fbb,
                        &TokenArgs {
                            type_: TokenType::PARAGRAPH,
                            value_type: TokenValue::Paragraph,
                            value: Some(custom_tag_elm.as_union_value()),
                        },
                    );
                    token_buf.push(custom_tag_token);

                    let text_val = fbb.create_string(&curr_html);
                    let html_elm = Html::create(
                        &mut fbb,
                        &HtmlArgs {
                            text: Some(text_val),
                        },
                    );
                    let token = Token::create(
                        &mut fbb,
                        &TokenArgs {
                            type_: TokenType::HTML,
                            value_type: TokenValue::Html,
                            value: Some(html_elm.as_union_value()),
                        },
                    );
                    token_buf.push(token);
                    curr_html = String::new();
                }
            }
        }
    } else {
        println!("no tokens!");
    }

    if curr_html.len() > 0 {
        let text_val = fbb.create_string(&curr_html);
        let html_elm = Html::create(
            &mut fbb,
            &HtmlArgs {
                text: Some(text_val),
            },
        );
        let token = Token::create(
            &mut fbb,
            &TokenArgs {
                type_: TokenType::HTML,
                value_type: TokenValue::Html,
                value: Some(html_elm.as_union_value()),
            },
        );
        token_buf.push(token);
    }
    let tokens_vector = fbb.create_vector(&token_buf);
    let prerendered_doc = Document::create(
        &mut fbb,
        &DocumentArgs {
            tokens: Some(tokens_vector),
        },
    );
    fbb.finish(prerendered_doc, None);

    Ok(fbb.finished_data().to_vec())
}

pub async fn flatbuffers_to_html(
    post: &BlogPost,
    fb_input: &Vec<u8>,
    conn: &DBPool,
    plugin_manager: &Arc<RwLock<PluginManager>>,
) -> Result<String, serde_json::Error> {
    let doc = root_as_document(&fb_input).map_err(|e| serde_json::Error::custom(e.to_string()))?;
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
            html.push_str(&render_token(&node, &doc, post, conn, plugin_manager).await);
        }
    }
    html.push_str("</body>");
    html.push_str("</html>");
    Ok(html.to_string())
}

fn pre_render_token(token: &Token) -> RenderTokenResult {
    return match token.type_() {
        TokenType::PARAGRAPH => {
            let paragraph = token.value_as_paragraph().unwrap();
            if is_custom_tag(paragraph.text()) {
                print!("custom tag {:?}", token);
                return RenderTokenResult::CustomTag;
            }
            RenderTokenResult::HTML(format!(
                "<p>{}</p>\n",
                render_inlines(paragraph.tokens().unwrap())
            ))
        }
        TokenType::BLOCKQUOTE => {
            let blockquote = token.value_as_block_quote().unwrap();
            RenderTokenResult::HTML(format!(
                "<blockquote>{}</blockquote>\n",
                render_inlines(blockquote.tokens())
            ))
        }
        TokenType::HEADING => {
            let heading = token.value_as_heading().unwrap();
            RenderTokenResult::HTML(format!(
                "<h{d}>{}</h{d}>\n",
                render_inlines(heading.tokens()),
                d = heading.depth()
            ))
        }
        TokenType::LIST => {
            let list = token.value_as_list().unwrap();
            let tag = if list.ordered() { "ol" } else { "ul" };
            let items_html = list
                .items()
                .iter()
                .map(|item| render_list_item(&item.value_as_list_item().unwrap()))
                .collect::<Vec<_>>()
                .join("\n");
            RenderTokenResult::HTML(format!("<{tag}>\n{items_html}\n</{tag}>\n"))
        }
        TokenType::SPACE => RenderTokenResult::HTML("\n".to_string()),
        TokenType::CODE => {
            let code = token.value_as_code().unwrap();
            RenderTokenResult::HTML(format!(
                "<pre><code class=\"language-{}\">{}</code></pre>\n",
                html_escape(code.lang()),
                html_escape(code.text())
            ))
        }
        TokenType::CODESPAN => {
            let codespan = token.value_as_codespan().unwrap();
            RenderTokenResult::HTML(format!("<code>{}</code>\n", html_escape(codespan.text())))
        }
        TokenType::HTML => {
            let html = token.value_as_html().unwrap();
            RenderTokenResult::HTML(format!("{}", html.text()))
        }
        TokenType::TABLE => {
            let table = token.value_as_table().unwrap();
            let header_html = table
                .header()
                .iter()
                .map(|cell| format!("<th>{}</th>", render_inlines(cell.tokens())))
                .collect::<Vec<_>>()
                .join("");
            let rows_html = table
                .rows()
                .iter()
                .map(|row| {
                    let row_html = row
                        .cells()
                        .unwrap()
                        .iter()
                        .map(|cell| format!("<td>{}</td>", render_inlines(cell.tokens())))
                        .collect::<Vec<_>>()
                        .join("");
                    format!("<tr>{}</tr>", row_html)
                })
                .collect::<Vec<_>>()
                .join("\n");
            RenderTokenResult::HTML(format!(
                "<table><thead><tr>{}</tr></thead><tbody>{}</tbody></table>\n",
                header_html, rows_html
            ))
        }
        TokenType::HR => RenderTokenResult::HTML("<hr/>".to_string()),
        TokenType::BR => RenderTokenResult::HTML("<br/>".to_string()),
        _ => RenderTokenResult::HTML(format!("<pre>TODO N/A****: {:?}</pre>", token.type_())),
    };
}

async fn render_token(
    token: &Token<'_>,
    root: &Document<'_>,
    post: &BlogPost,
    conn: &DBPool,
    plugin_manager: &Arc<RwLock<PluginManager>>,
) -> String {
    return match token.type_() {
        TokenType::PARAGRAPH => {
            let paragraph = token.value_as_paragraph().unwrap();
            if let Some(custom) =
                try_handle_custom_tag(paragraph.text(), root, post, conn, plugin_manager).await
            {
                return custom;
            } else {
                return format!("<p>{}</p>\n", render_inlines(paragraph.tokens().unwrap()));
            }
        }
        TokenType::BLOCKQUOTE => {
            let blockquote = token.value_as_block_quote().unwrap();
            return format!(
                "<blockquote>{}</blockquote>\n",
                render_inlines(blockquote.tokens())
            );
        }
        TokenType::HEADING => {
            let heading = token.value_as_heading().unwrap();
            format!(
                "<h{d}>{}</h{d}>\n",
                render_inlines(heading.tokens()),
                d = heading.depth()
            )
        }
        TokenType::LIST => {
            let list = token.value_as_list().unwrap();
            let tag = if list.ordered() { "ol" } else { "ul" };
            let items_html = list
                .items()
                .iter()
                .map(|item| render_list_item(&item.value_as_list_item().unwrap()))
                .collect::<Vec<_>>()
                .join("\n");
            format!("<{tag}>\n{items_html}\n</{tag}>\n")
        }
        TokenType::SPACE => "\n".to_string(),
        TokenType::CODE => {
            let code = token.value_as_code().unwrap();
            format!(
                "<pre><code class=\"language-{}\">{}</code></pre>\n",
                html_escape(code.lang()),
                html_escape(code.text())
            )
        }
        TokenType::CODESPAN => {
            let codespan = token.value_as_codespan().unwrap();
            format!("<code>{}</code>\n", html_escape(codespan.text()))
        }
        TokenType::HTML => {
            let html = token.value_as_html().unwrap();
            format!("{}", html.text())
        }
        TokenType::TABLE => {
            let table = token.value_as_table().unwrap();
            let header_html = table
                .header()
                .iter()
                .map(|cell| format!("<th>{}</th>", render_inlines(cell.tokens())))
                .collect::<Vec<_>>()
                .join("");
            let rows_html = table
                .rows()
                .iter()
                .map(|row| {
                    let row_html = row
                        .cells()
                        .unwrap()
                        .iter()
                        .map(|cell| format!("<td>{}</td>", render_inlines(cell.tokens())))
                        .collect::<Vec<_>>()
                        .join("");
                    format!("<tr>{}</tr>", row_html)
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "<table><thead><tr>{}</tr></thead><tbody>{}</tbody></table>\n",
                header_html, rows_html
            )
        }
        TokenType::HR => "<hr/>".to_string(),
        TokenType::BR => "<br/>".to_string(),
        _ => format!("<pre>TODO N/A****: {:?}</pre>", token.type_()),
    };
}

fn is_custom_tag(text: &str) -> bool {
    let trimmed = text.trim();
    if let Some(_) = trimmed.find("[[") {
        if let Some(_) = trimmed.find("]]") {
            return true;
        }
    }
    false
}

async fn try_handle_custom_tag(
    text: &str,
    root: &Document<'_>,
    post: &BlogPost,
    conn: &DBPool,
    plugin_manager: &Arc<RwLock<PluginManager>>,
) -> Option<String> {
    let trimmed = text.trim();
    if let Some(start) = trimmed.find("[[") {
        if let Some(end) = trimmed.find("]]") {
            let open_tag = &trimmed[start + 2..end];
            let (tag_name, attributes) = parse_tag_and_attributes(open_tag);
            let close_tag = format!("[[/{}]]", tag_name);
            if let Some(close_start) = trimmed.find(&close_tag) {
                let content = &trimmed[end + 2..close_start];
                return Some(
                    handle_custom_tag(
                        &tag_name,
                        content,
                        attributes,
                        root,
                        post,
                        conn,
                        plugin_manager,
                    )
                    .await,
                )?;
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

async fn handle_custom_tag(
    tag: &str,
    content: &str,
    _attribs: HashMap<String, String>,
    _root: &Document<'_>,
    post: &BlogPost,
    conn: &DBPool,
    plugin_manager: &Arc<RwLock<PluginManager>>,
) -> Option<String> {
    if tag == "custom-html" {
        return Some(content.to_string());
    }

    if tag == "posts" {
        let mut recent_posts = String::new();
        recent_posts.push_str("<h1>Recent Posts</h1>");
        recent_posts.push_str("<ul>");
        match db::get_all_post(&conn).await {
            Ok(posts) => {
                for post in posts {
                    if post.slug == "" {
                        continue;
                    }
                    recent_posts.push_str(&format!(
                        "<li><a href='/{}'> {} | {} </a></li>",
                        html_escape(&post.slug),
                        html_escape(&post.title),
                        html_escape(&post.created_at),
                    ));
                }
            }
            Err(e) => {
                recent_posts.push_str(&format!("Error getting posts: {}", e.to_string()));
            }
        }
        recent_posts.push_str("</ul>");
        return Some(recent_posts.to_string());
    }

    let mut plugin_manager = plugin_manager.write().unwrap();
    match plugin_manager.run_plugin(tag, &post.slug, &post.content_flatbuffer) {
        Ok(resp) => Some(resp),
        Err(e) => format!("plugin error {e}").into(),
    }
}

fn render_inlines(tokens: Vector<'_, ForwardsUOffset<Token<'_>>>) -> String {
    tokens
        .iter()
        .map(|token| {
            match token.type_() {
                //TokenType::TEXT => html_escape(token.value_as_text().unwrap().text()),
                TokenType::TEXT => {
                    let text = token.value_as_text().unwrap();
                    if let Some(tokens) = text.tokens() {
                        if tokens.len() > 0 {
                            return format!("{}", render_inlines(tokens));
                        }
                    }
                    format!("{}", html_escape(text.text()))
                }
                TokenType::STRONG => format!(
                    "<strong>{}</strong>",
                    render_inlines(token.value_as_strong().unwrap().tokens())
                ),
                TokenType::DEL => format!(
                    "<del>{}</del>",
                    render_inlines(token.value_as_del().unwrap().tokens())
                ),
                TokenType::EM => format!(
                    "<em>{}</em>",
                    render_inlines(token.value_as_em().unwrap().tokens())
                ),
                TokenType::CODE => {
                    let code = token.value_as_code().unwrap();
                    format!(
                        "<pre><code class=\"language-{}\">{}</code></pre>\n",
                        html_escape(code.lang()),
                        html_escape(code.text())
                    )
                }
                TokenType::CODESPAN => {
                    let codespan = token.value_as_codespan().unwrap();
                    format!("<code>{}</code>\n", html_escape(codespan.text()))
                }
                TokenType::LINK => {
                    let link = token.value_as_link().unwrap();
                    if let Some(title) = link.title() {
                        format!(
                            "<a href=\"{}\" title=\"{}\">{}</a>",
                            html_escape(link.href().unwrap()),
                            html_escape(title),
                            render_inlines(link.tokens())
                        )
                    } else {
                        format!(
                            "<a href=\"{}\">{}</a>",
                            html_escape(link.href().unwrap()),
                            render_inlines(link.tokens())
                        )
                    }
                }
                TokenType::HTML => {
                    let html = token.value_as_html().unwrap();
                    format!("{}", html.text())
                }
                TokenType::IMAGE => {
                    let img = token.value_as_image().unwrap();
                    if let Some(title) = img.title() {
                        format!(
                            "<img src=\"{}\" title=\"{}\" alt=\"{}\"</>",
                            html_escape(img.href().unwrap()),
                            html_escape(title),
                            render_inlines(img.tokens())
                        )
                    } else {
                        format!("<img src=\"{}\" />", html_escape(img.href().unwrap()))
                    }
                }
                TokenType::UNKNOWN => "<h2>UNKNOWN</h2>".to_string(),
                TokenType::TABLE => {
                    let table = token.value_as_table().unwrap();
                    let header_html = table
                        .header()
                        .iter()
                        .map(|cell| format!("<th>{}</th>", render_inlines(cell.tokens())))
                        .collect::<Vec<_>>()
                        .join("");
                    let rows_html = table
                        .rows()
                        .iter()
                        .map(|row| {
                            let row_html = row
                                .cells()
                                .unwrap()
                                .iter()
                                .map(|cell| format!("<td>{}</td>", render_inlines(cell.tokens())))
                                .collect::<Vec<_>>()
                                .join("");
                            format!("<tr>{}</tr>", row_html)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!(
                        "<table><thead><tr>{}</tr></thead><tbody>{}</tbody></table>\n",
                        header_html, rows_html
                    )
                }
                TokenType::LIST => {
                    let list = token.value_as_list().unwrap();
                    let tag = if list.ordered() { "ol" } else { "ul" };
                    let items_html = list
                        .items()
                        .iter()
                        .map(|item| render_list_item(&item.value_as_list_item().unwrap()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("<{tag}>\n{items_html}\n</{tag}>\n")
                }
                TokenType::HR => "<hr/>".to_string(),
                TokenType::BR => "<br/>".to_string(),
                TokenType::LIST_ITEM => "<h1>LIST ITEM</h1>".to_string(),
                TokenType::PARAGRAPH => {
                    let paragraph = token.value_as_paragraph().unwrap();
                    return format!("<p>{}</p>\n", render_inlines(paragraph.tokens().unwrap()));
                }
                TokenType::BLOCKQUOTE => {
                    let blockquote = token.value_as_block_quote().unwrap();
                    return format!(
                        "<blockquote>{}</blockquote>\n",
                        render_inlines(blockquote.tokens())
                    );
                }
                _ => format!("TODO: {:?}", token.type_()),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_list_item(item: &ListItem) -> String {
    let task_input = if item.task() {
        &format!(
            "<input type=\"checkbox\" {}/>",
            if item.checked() { "checked" } else { "" }
        )
    } else {
        ""
    };

    format!(
        "<li>{}{}</li>",
        task_input,
        render_inlines(item.tokens().unwrap())
    )
}
