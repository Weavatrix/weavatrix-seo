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

pub fn in_main(stack: &[String]) -> bool {
    stack.iter().any(|name| name == "main")
}

/// True when a script is recognized application data, not arbitrary JS.
pub fn is_app_data(tag: &Tag) -> bool {
    let id = attr(tag, "id").unwrap_or_default();
    let typ = attr(tag, "type").unwrap_or_default();
    id.eq_ignore_ascii_case("__NEXT_DATA__")
        || typ.eq_ignore_ascii_case("application/json")
        || typ.eq_ignore_ascii_case("text/x-component")
}

/// Next.js flight / RSC markers inside an otherwise anonymous script.
pub fn looks_like_rsc(text: &str) -> bool {
    text.contains("self.__next_f") || text.contains("__NEXT_DATA__") || text.contains("$Sreact")
}

pub fn rel_tokens(tag: &Tag) -> Vec<String> {
    attr(tag, "rel")
        .map(|value| {
            value
                .split_whitespace()
                .map(str::to_ascii_lowercase)
                .collect()
        })
        .unwrap_or_default()
}

pub fn read_tag(
    source: &str,
    tokens: &[weavatrix_parse::token::Token],
    start: usize,
) -> Option<(Tag, usize)> {
    use weavatrix_parse::token::TokenKind;
    let text_at = |index: usize| tokens.get(index).map_or("", |token| token.text(source));
    let mut index = start + 1;
    let closing = tokens
        .get(index)
        .is_some_and(|token| token.kind == TokenKind::Punctuation && token.text(source) == "/");
    if closing {
        index += 1;
    }
    let name = text_at(index).to_ascii_lowercase();
    if name.is_empty() {
        return None;
    }
    index += 1;
    let mut attrs = Vec::new();
    while index < tokens.len() {
        if tokens[index].kind == TokenKind::Punctuation && text_at(index) == ">" {
            index += 1;
            break;
        }
        if tokens[index].kind == TokenKind::Identifier {
            let key = text_at(index).to_ascii_lowercase();
            index += 1;
            let mut value = String::new();
            if tokens.get(index).is_some_and(|token| {
                token.kind == TokenKind::Punctuation && token.text(source) == "="
            }) {
                index += 1;
                value = unquote(text_at(index));
                index += 1;
            }
            attrs.push((key, value));
            continue;
        }
        index += 1;
    }
    Some((
        Tag {
            name,
            attrs,
            closing,
        },
        index,
    ))
}
