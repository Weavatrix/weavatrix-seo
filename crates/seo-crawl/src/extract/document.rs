//! Token-stream HTML extraction via weavatrix-parse.

use super::jsonld::parse_json_ld;
use weavatrix_parse::Language;
use weavatrix_parse::token::{Mode, Token, TokenKind, Tokenizer};
use weavatrix_seo_model::{Alternate, Heading, ImageRef};

struct Tag {
    name: String,
    attrs: Vec<(String, String)>,
    closing: bool,
}

/// Extracts SEO-visible fields from HTML.
#[must_use]
pub fn extract_html(html: &str) -> ExtractedPageDraft {
    let tokens: Vec<Token> = Tokenizer::new(html, Language::Html)
        .mode(Mode::Lite)
        .collect();
    let mut walker = Walker {
        source: html,
        tokens: &tokens,
        index: 0,
        skip_depth: 0,
        in_title: false,
        json_ld: false,
        capture_script: false,
        json_buf: String::new(),
        text_buf: String::new(),
        draft: ExtractedPageDraft::default(),
        stack: Vec::new(),
    };
    walker.run();
    walker.draft
}

/// Extraction before URL/status are attached.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtractedPageDraft {
    /// `html[lang]`
    pub html_lang: Option<String>,
    /// Document title.
    pub title: Option<String>,
    /// Meta description.
    pub description: Option<String>,
    /// Canonical href.
    pub canonical: Option<String>,
    /// Robots meta content.
    pub robots: Vec<String>,
    /// Hreflang alternates.
    pub alternates: Vec<Alternate>,
    /// Headings.
    pub headings: Vec<Heading>,
    /// `a[href]` values.
    pub links: Vec<String>,
    /// Images.
    pub images: Vec<ImageRef>,
    /// JSON-LD blocks.
    pub json_ld: Vec<weavatrix_seo_model::JsonLd>,
    /// Visible main text.
    pub text: String,
    /// Script/RSC payload text used for market and claim integrity.
    pub payload: String,
    /// Open Graph title.
    pub og_title: Option<String>,
    /// Open Graph description.
    pub og_description: Option<String>,
    /// Open Graph image.
    pub og_image: Option<String>,
}

struct Walker<'source> {
    source: &'source str,
    tokens: &'source [Token],
    index: usize,
    skip_depth: usize,
    in_title: bool,
    json_ld: bool,
    capture_script: bool,
    json_buf: String,
    text_buf: String,
    draft: ExtractedPageDraft,
    stack: Vec<String>,
}

impl Walker<'_> {
    fn run(&mut self) {
        while self.index < self.tokens.len() {
            if self.punct("<") {
                self.tag();
                continue;
            }
            self.text_token();
            self.index += 1;
        }
        self.flush_text();
        self.draft.text = self
            .draft
            .text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    }

    fn tag(&mut self) {
        let Some(tag) = self.read_tag() else {
            self.index += 1;
            return;
        };
        if tag.closing {
            self.close(&tag.name);
            return;
        }
        self.open(&tag);
        if is_void(&tag.name) {
            self.close(&tag.name);
        }
    }

    fn open(&mut self, tag: &Tag) {
        self.stack.push(tag.name.clone());
        match tag.name.as_str() {
            "html" => {
                if let Some(lang) = attr(tag, "lang") {
                    self.draft.html_lang = Some(lang);
                }
            }
            "title" => self.in_title = true,
            "meta" => apply_meta(&mut self.draft, tag),
            "link" => apply_link(&mut self.draft, tag),
            "script" => {
                if attr(tag, "type")
                    .is_some_and(|value| value.eq_ignore_ascii_case("application/ld+json"))
                {
                    self.json_ld = true;
                    self.json_buf.clear();
                } else {
                    self.skip_depth += 1;
                    self.capture_script = true;
                }
            }
            "style" | "noscript" => {
                self.skip_depth += 1;
                self.capture_script = false;
            }
            "a" => {
                if let Some(href) = attr(tag, "href") {
                    self.draft.links.push(href);
                }
            }
            "img" => self.draft.images.push(ImageRef {
                src: attr(tag, "src").unwrap_or_default(),
                alt: attr(tag, "alt"),
            }),
            _ => {}
        }
        if is_skip_text(&tag.name) {
            self.flush_text();
        }
    }

    fn close(&mut self, name: &str) {
        if name == "title" {
            self.in_title = false;
        }
        if name == "script" && self.json_ld {
            self.json_ld = false;
            self.draft
                .json_ld
                .push(parse_json_ld(std::mem::take(&mut self.json_buf)));
        }
        if matches!(name, "script" | "style" | "noscript") && self.skip_depth > 0 && !self.json_ld {
            self.skip_depth -= 1;
        }
        if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            let level = name.as_bytes()[1] - b'0';
            let text = std::mem::take(&mut self.text_buf);
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !text.is_empty() {
                self.draft.headings.push(Heading { level, text });
            }
        }
        if let Some(position) = self.stack.iter().rposition(|item| item == name) {
            self.stack.truncate(position);
        }
    }

    fn text_token(&mut self) {
        let Some(token) = self.tokens.get(self.index) else {
            return;
        };
        let text = token.text(self.source);
        if self.json_ld {
            self.json_buf.push_str(text);
            return;
        }
        if self.skip_depth > 0 {
            if self.capture_script {
                self.draft.payload.push_str(text);
            }
            return;
        }
        if token.kind == TokenKind::Punctuation {
            return;
        }
        if self.in_title {
            let current = self.draft.title.get_or_insert_with(String::new);
            current.push_str(text);
            *current = current.split_whitespace().collect::<Vec<_>>().join(" ");
            return;
        }
        if in_heading(&self.stack) || !is_boilerplate(&self.stack) {
            self.text_buf.push_str(text);
            self.text_buf.push(' ');
        }
    }

    fn flush_text(&mut self) {
        if !self.text_buf.trim().is_empty() && !in_heading(&self.stack) {
            self.draft.text.push_str(&self.text_buf);
            self.draft.text.push(' ');
        }
        self.text_buf.clear();
    }

    fn punct(&self, mark: &str) -> bool {
        self.tokens.get(self.index).is_some_and(|token| {
            token.kind == TokenKind::Punctuation && token.text(self.source) == mark
        })
    }

    fn text_at(&self, index: usize) -> &str {
        self.tokens
            .get(index)
            .map_or("", |token| token.text(self.source))
    }

    fn read_tag(&mut self) -> Option<Tag> {
        let mut index = self.index + 1;
        let closing = self.tokens.get(index).is_some_and(|token| {
            token.kind == TokenKind::Punctuation && token.text(self.source) == "/"
        });
        if closing {
            index += 1;
        }
        let name = self.text_at(index).to_ascii_lowercase();
        if name.is_empty() {
            return None;
        }
        index += 1;
        let mut attrs = Vec::new();
        while index < self.tokens.len() {
            if self.tokens[index].kind == TokenKind::Punctuation && self.text_at(index) == ">" {
                index += 1;
                break;
            }
            if self.tokens[index].kind == TokenKind::Identifier {
                let key = self.text_at(index).to_ascii_lowercase();
                index += 1;
                let mut value = String::new();
                if self.tokens.get(index).is_some_and(|token| {
                    token.kind == TokenKind::Punctuation && token.text(self.source) == "="
                }) {
                    index += 1;
                    value = unquote(self.text_at(index));
                    index += 1;
                }
                attrs.push((key, value));
                continue;
            }
            index += 1;
        }
        self.index = index;
        Some(Tag {
            name,
            attrs,
            closing,
        })
    }
}

fn attr(tag: &Tag, name: &str) -> Option<String> {
    tag.attrs
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
        .filter(|value| !value.is_empty())
}

fn apply_meta(draft: &mut ExtractedPageDraft, tag: &Tag) {
    let content = attr(tag, "content");
    if let Some(name) = attr(tag, "name") {
        match name.to_ascii_lowercase().as_str() {
            "description" => draft.description.clone_from(&content),
            "robots" => {
                if let Some(content) = content.clone() {
                    draft.robots.push(content);
                }
            }
            _ => {}
        }
    }
    if let Some(property) = attr(tag, "property").or_else(|| attr(tag, "name")) {
        match property.to_ascii_lowercase().as_str() {
            "og:title" => draft.og_title = content,
            "og:description" => draft.og_description = content,
            "og:image" => draft.og_image = content,
            _ => {}
        }
    }
}

fn apply_link(draft: &mut ExtractedPageDraft, tag: &Tag) {
    let rel = attr(tag, "rel").map(|value| value.to_ascii_lowercase());
    let href = attr(tag, "href");
    match (rel.as_deref(), href) {
        (Some("canonical"), Some(href)) => draft.canonical = Some(href),
        (Some("alternate"), Some(href)) => {
            if let Some(hreflang) = attr(tag, "hreflang") {
                draft.alternates.push(Alternate { hreflang, href });
            }
        }
        _ => {}
    }
}

fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch| ch == '"' || ch == '\'')
        .to_owned()
}

fn is_void(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
    )
}

fn is_skip_text(name: &str) -> bool {
    matches!(name, "nav" | "footer" | "header" | "script" | "style")
}

fn is_boilerplate(stack: &[String]) -> bool {
    stack
        .iter()
        .any(|name| matches!(name.as_str(), "nav" | "footer" | "header" | "noscript"))
}

fn in_heading(stack: &[String]) -> bool {
    stack
        .iter()
        .any(|name| matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6"))
}
