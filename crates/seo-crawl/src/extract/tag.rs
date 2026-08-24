//! HTML tag tokens.

pub struct Tag {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub closing: bool,
}

pub fn attr(tag: &Tag, name: &str) -> Option<String> {
    attr_raw(tag, name).filter(|value| !value.is_empty())
}

pub fn attr_raw(tag: &Tag, name: &str) -> Option<String> {
    tag.attrs
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
}

pub fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch| ch == '"' || ch == '\'')
        .to_owned()
}

pub fn is_void(name: &str) -> bool {
    matches!(
        name,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "source"
    )
}

pub fn is_skip_text(name: &str) -> bool {
    matches!(name, "nav" | "footer" | "header" | "script" | "style")
}

pub fn is_boilerplate(stack: &[String]) -> bool {
    stack
        .iter()
        .any(|name| matches!(name.as_str(), "nav" | "footer" | "header" | "noscript"))
}

pub fn in_heading(stack: &[String]) -> bool {
    stack
        .iter()
        .any(|name| matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6"))
}
