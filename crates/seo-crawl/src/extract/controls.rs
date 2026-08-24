//! Accessible-name recording for form controls.

use super::tag::{Tag, attr, attr_raw};
use std::collections::BTreeSet;

/// Collects labels and whether each control already has a name.
#[derive(Default)]
pub struct ControlRecorder {
    label_for: BTreeSet<String>,
    controls: Vec<(Option<String>, bool)>,
    button_depth: u32,
    button_text: String,
}

impl ControlRecorder {
    pub fn open_label(&mut self, tag: &Tag) {
        if let Some(for_id) = attr(tag, "for") {
            self.label_for.insert(for_id);
        }
    }

    pub fn open_control(&mut self, tag: &Tag, in_label: bool) {
        if tag.name == "input"
            && attr(tag, "type").is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
        {
            return;
        }
        let mut named = in_label
            || attr(tag, "aria-label").is_some()
            || attr(tag, "aria-labelledby").is_some()
            || attr(tag, "title").is_some();
        let input_type = attr(tag, "type").map(|value| value.to_ascii_lowercase());
        if tag.name == "input" {
            match input_type.as_deref() {
                Some("submit" | "button" | "reset") => {
                    named |= attr(tag, "value").is_some();
                }
                Some("image") => named |= attr_raw(tag, "alt").is_some(),
                _ => {}
            }
        }
        if tag.name == "button" {
            self.button_depth += 1;
            if self.button_depth == 1 {
                self.button_text.clear();
            }
        }
        self.controls.push((attr(tag, "id"), named));
    }

    pub fn text(&mut self, text: &str) {
        if self.button_depth > 0 {
            self.button_text.push_str(text);
        }
    }

    pub fn close_button(&mut self) {
        if self.button_depth == 0 {
            return;
        }
        self.button_depth -= 1;
        if self.button_depth == 0
            && !self.button_text.split_whitespace().collect::<String>().is_empty()
            && let Some((_, named)) = self.controls.last_mut()
        {
            *named = true;
        }
    }

    pub fn unlabeled(&self) -> usize {
        self.controls
            .iter()
            .filter(|(id, named)| {
                !named && id.as_ref().is_none_or(|id| !self.label_for.contains(id))
            })
            .count()
    }
}
