//! Token-stream walk over HTML.

use super::document::ExtractedPageDraft;
use super::jsonld::parse_json_ld;
use super::meta::{apply_link, apply_meta};
use super::tag::{
    Tag, attr, attr_raw, in_heading, is_boilerplate, is_skip_text, is_void, unquote,
};
use std::collections::BTreeSet;
use weavatrix_parse::token::{Token, TokenKind};
use weavatrix_seo_model::{Heading, ImageRef};

pub struct Walker<'source> {
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
    label_for: BTreeSet<String>,
    controls: Vec<(Option<String>, bool)>,
}

impl<'source> Walker<'source> {
    pub fn new(source: &'source str, tokens: &'source [Token]) -> Self {
        Self {
            source,
            tokens,
            index: 0,
            skip_depth: 0,
            in_title: false,
            json_ld: false,
            capture_script: false,
            json_buf: String::new(),
            text_buf: String::new(),
            draft: ExtractedPageDraft::default(),
            stack: Vec::new(),
            label_for: BTreeSet::new(),
            controls: Vec::new(),
        }
    }

    pub fn run(&mut self) {
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
        self.draft.unlabeled_controls = self
            .controls
            .iter()
            .filter(|(id, named)| {
                !named && id.as_ref().is_none_or(|id| !self.label_for.contains(id))
            })
            .count();
    }

    pub fn finish(self) -> ExtractedPageDraft {
        self.draft
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
            "main" => self.draft.has_main = true,
            "label" => {
                if let Some(for_id) = attr(tag, "for") {
                    self.label_for.insert(for_id);
                }
            }
            "script" => self.open_script(tag),
            "style" | "noscript" => {
                self.skip_depth += 1;
                self.capture_script = false;
            }
            "a" => {
                if let Some(href) = attr(tag, "href") {
                    self.draft.links.push(href);
                }
            }
            "img" => self.draft.images.push(image(tag)),
            "input" | "select" | "textarea" | "button" => self.note_control(tag),
            _ => {
                if attr(tag, "role").is_some_and(|role| role.eq_ignore_ascii_case("main")) {
                    self.draft.has_main = true;
                }
            }
        }
        if is_skip_text(&tag.name) {
            self.flush_text();
        }
    }

    fn open_script(&mut self, tag: &Tag) {
        if attr(tag, "type").is_some_and(|value| value.eq_ignore_ascii_case("application/ld+json"))
        {
            self.json_ld = true;
            self.json_buf.clear();
        } else {
            self.skip_depth += 1;
            self.capture_script = true;
        }
    }

    fn note_control(&mut self, tag: &Tag) {
        if tag.name == "input"
            && attr(tag, "type").is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
        {
            return;
        }
        let named = attr(tag, "aria-label").is_some()
            || attr(tag, "aria-labelledby").is_some()
            || attr(tag, "title").is_some()
            || self.stack.iter().any(|name| name == "label");
        self.controls.push((attr(tag, "id"), named));
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

fn image(tag: &Tag) -> ImageRef {
    let role = attr(tag, "role").unwrap_or_default();
    let hidden = attr(tag, "aria-hidden").is_some_and(|value| value.eq_ignore_ascii_case("true"))
        || role.eq_ignore_ascii_case("presentation")
        || role.eq_ignore_ascii_case("none");
    ImageRef {
        src: attr(tag, "src").unwrap_or_default(),
        alt: attr_raw(tag, "alt"),
        hidden,
    }
}
