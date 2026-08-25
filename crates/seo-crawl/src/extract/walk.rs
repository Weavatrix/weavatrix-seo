//! Token-stream walk over HTML.

use super::controls::ControlRecorder;
use super::document::ExtractedPageDraft;
use super::jsonld::parse_json_ld;
use super::meta::{apply_link, apply_meta};
use super::tag::{
    Tag, attr, attr_raw, in_heading, in_main, is_app_data, is_boilerplate, is_skip_text, is_void,
    looks_like_rsc, read_tag, rel_tokens,
};
use weavatrix_parse::token::{Token, TokenKind};
use weavatrix_seo_model::{Heading, ImageRef, LinkLocation, LinkRef};

struct OpenLink {
    href: String,
    rel: Vec<String>,
    location: LinkLocation,
    anchor: String,
}

pub struct Walker<'source> {
    source: &'source str,
    tokens: &'source [Token],
    index: usize,
    skip_depth: usize,
    in_title: bool,
    json_ld: bool,
    app_data: bool,
    script_buf: String,
    text_buf: String,
    draft: ExtractedPageDraft,
    stack: Vec<String>,
    controls: ControlRecorder,
    open_link: Option<OpenLink>,
    last_heading: String,
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
            app_data: false,
            script_buf: String::new(),
            text_buf: String::new(),
            draft: ExtractedPageDraft::default(),
            stack: Vec::new(),
            controls: ControlRecorder::default(),
            open_link: None,
            last_heading: String::new(),
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
        self.draft.text = collapse(&self.draft.text);
        self.draft.heading_text = collapse(&self.draft.heading_text);
        self.draft.main_text = collapse(&self.draft.main_text);
        self.draft.unlabeled_controls = self.controls.unlabeled();
    }

    pub fn finish(self) -> ExtractedPageDraft {
        self.draft
    }

    fn tag(&mut self) {
        let Some((tag, next)) = read_tag(self.source, self.tokens, self.index) else {
            self.index += 1;
            return;
        };
        self.index = next;
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
        if matches!(tag.name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            self.flush_text();
        }
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
            "label" => self.controls.open_label(tag),
            "script" => self.open_script(tag),
            "style" | "noscript" => {
                self.skip_depth += 1;
            }
            "a" => self.open_anchor(tag),
            "img" => self.draft.images.push(image(tag)),
            "input" | "select" | "textarea" | "button" => {
                let in_label = self.stack.iter().any(|name| name == "label");
                self.controls.open_control(tag, in_label);
            }
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
            self.script_buf.clear();
            return;
        }
        self.skip_depth += 1;
        self.app_data = is_app_data(tag);
        self.script_buf.clear();
    }

    fn open_anchor(&mut self, tag: &Tag) {
        let Some(href) = attr(tag, "href") else {
            return;
        };
        self.draft.links.push(href.clone());
        let rel = rel_tokens(tag);
        let location = if rel.iter().any(|token| token == "breadcrumb") {
            LinkLocation::Breadcrumb
        } else {
            LinkLocation::from_stack(&self.stack)
        };
        self.open_link = Some(OpenLink {
            href,
            rel,
            location,
            anchor: String::new(),
        });
    }

    fn close(&mut self, name: &str) {
        if name == "button" {
            self.controls.close_button();
        }
        if name == "title" {
            self.in_title = false;
        }
        if name == "a" {
            self.close_anchor();
        }
        if name == "script" {
            self.close_script();
        }
        if matches!(name, "script" | "style" | "noscript") && self.skip_depth > 0 && !self.json_ld {
            self.skip_depth -= 1;
        }
        if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            let level = name.as_bytes()[1] - b'0';
            let text = collapse(&std::mem::take(&mut self.text_buf));
            if !text.is_empty() {
                self.draft.heading_text.push_str(&text);
                self.draft.heading_text.push(' ');
                self.last_heading.clone_from(&text);
                self.draft.headings.push(Heading { level, text });
            }
        }
        if let Some(position) = self.stack.iter().rposition(|item| item == name) {
            self.stack.truncate(position);
        }
    }

    fn close_script(&mut self) {
        let body = std::mem::take(&mut self.script_buf);
        if self.json_ld {
            self.json_ld = false;
            self.draft.json_ld.push(parse_json_ld(body));
            return;
        }
        if self.app_data || looks_like_rsc(&body) {
            self.draft.payload.push_str(&body);
        } else {
            self.draft.arbitrary_script.push_str(&body);
        }
        self.app_data = false;
    }

    fn close_anchor(&mut self) {
        let Some(open) = self.open_link.take() else {
            return;
        };
        let anchor = collapse(&open.anchor);
        let context = if self.last_heading.is_empty() {
            None
        } else {
            Some(self.last_heading.clone())
        };
        self.draft.link_refs.push(LinkRef {
            href: open.href,
            anchor: if anchor.is_empty() { None } else { Some(anchor) },
            context,
            rel: open.rel,
            location: open.location,
        });
    }

    fn text_token(&mut self) {
        let Some(token) = self.tokens.get(self.index) else {
            return;
        };
        let text = token.text(self.source);
        if self.json_ld || self.in_script() {
            self.script_buf.push_str(text);
            return;
        }
        if self.skip_depth > 0 {
            return;
        }
        if token.kind == TokenKind::Punctuation {
            return;
        }
        if self.in_title {
            let current = self.draft.title.get_or_insert_with(String::new);
            current.push_str(text);
            *current = collapse(current);
            return;
        }
        if let Some(open) = &mut self.open_link {
            open.anchor.push_str(text);
            open.anchor.push(' ');
        }
        self.controls.text(text);
        if in_heading(&self.stack) || !is_boilerplate(&self.stack) {
            self.text_buf.push_str(text);
            self.text_buf.push(' ');
        }
    }

    fn flush_text(&mut self) {
        if !self.text_buf.trim().is_empty() && !in_heading(&self.stack) {
            self.draft.text.push_str(&self.text_buf);
            self.draft.text.push(' ');
            if in_main(&self.stack) {
                self.draft.main_text.push_str(&self.text_buf);
                self.draft.main_text.push(' ');
            }
        }
        self.text_buf.clear();
    }

    fn in_script(&self) -> bool {
        self.stack.last().is_some_and(|name| name == "script")
    }

    fn punct(&self, mark: &str) -> bool {
        self.tokens.get(self.index).is_some_and(|token| {
            token.kind == TokenKind::Punctuation && token.text(self.source) == mark
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

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
