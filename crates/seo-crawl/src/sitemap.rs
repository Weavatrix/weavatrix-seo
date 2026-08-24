//! Sitemap and sitemap-index loc extraction.

use weavatrix_parse::Language;
use weavatrix_parse::token::{Mode, TokenKind, Tokenizer};
use weavatrix_seo_model::AbsoluteUrl;

/// Extracts absolute `loc` URLs from a sitemap or sitemap index.
#[must_use]
pub fn parse_sitemap(body: &str, base: &AbsoluteUrl) -> Vec<AbsoluteUrl> {
    let tokens: Vec<_> = Tokenizer::new(body, Language::Xml)
        .mode(Mode::Lite)
        .collect();
    let mut locs = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == TokenKind::Punctuation && tokens[index].text(body) == "<" {
            let name_index = index + 1;
            if tokens
                .get(name_index)
                .is_some_and(|token| token.text(body).eq_ignore_ascii_case("loc"))
            {
                index = name_index + 1;
                while index < tokens.len()
                    && !(tokens[index].kind == TokenKind::Punctuation
                        && tokens[index].text(body) == ">")
                {
                    index += 1;
                }
                index += 1;
                let mut text = String::new();
                while index < tokens.len()
                    && !(tokens[index].kind == TokenKind::Punctuation
                        && tokens[index].text(body) == "<")
                {
                    text.push_str(tokens[index].text(body));
                    index += 1;
                }
                let text = text.trim();
                if let Ok(url) = AbsoluteUrl::parse(text).or_else(|_| base.join(text)) {
                    locs.push(url);
                }
                continue;
            }
        }
        index += 1;
    }
    locs.sort();
    locs.dedup();
    locs
}

#[cfg(test)]
mod tests {
    use super::parse_sitemap;
    use weavatrix_seo_model::AbsoluteUrl;

    #[test]
    fn reads_urlset_and_index() {
        let base = AbsoluteUrl::parse("https://x.test/").unwrap();
        let body = r#"<?xml version="1.0"?>
<urlset>
  <url><loc>https://x.test/a</loc></url>
  <url><loc>/b</loc></url>
</urlset>"#;
        let locs = parse_sitemap(body, &base);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].to_string(), "https://x.test/a");
        assert_eq!(locs[1].to_string(), "https://x.test/b");
    }
}
