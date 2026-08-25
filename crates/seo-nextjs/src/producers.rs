//! Metadata, sitemap, JSON-LD, and helper producers with source spans.

use weavatrix_parse::{Language, extract};
use weavatrix_seo_source::SourceSymbol;

/// Producers discovered in one page/sitemap module.
#[derive(Debug, Clone, Default)]
pub struct Producers {
    /// Default page component.
    pub page: Option<SourceSymbol>,
    /// `generateMetadata` / `metadata`.
    pub metadata: Option<SourceSymbol>,
    /// `generateStaticParams`.
    pub static_params: Option<SourceSymbol>,
    /// JSON-LD helpers.
    pub json_ld: Vec<SourceSymbol>,
    /// Imported SEO helpers.
    pub helpers: Vec<SourceSymbol>,
}

/// Inspects one TypeScript/JavaScript module.
#[must_use]
pub fn inspect(path: &str, source: &str) -> Producers {
    let facts = extract(source, Language::TypeScript);
    let mut out = Producers::default();
    for decl in &facts.declarations {
        let symbol = SourceSymbol {
            path: path.to_owned(),
            name: decl.name.clone(),
            start_line: Some(decl.span.line),
            end_line: Some(decl.extent.end_line),
        };
        match decl.name.as_str() {
            "generateMetadata" | "metadata" => out.metadata = Some(symbol),
            "generateStaticParams" => out.static_params = Some(symbol),
            name if looks_like_json_ld(name) => out.json_ld.push(symbol),
            name if decl.exported && out.page.is_none() && !is_reserved(name) => {
                out.page = Some(symbol);
            }
            _ => {}
        }
    }
    for import in &facts.imports {
        let seo = import.specifier.to_ascii_lowercase().contains("seo")
            || import
                .names
                .iter()
                .any(|name| name.to_ascii_lowercase().contains("seo") || looks_like_json_ld(name));
        if !seo {
            continue;
        }
        let name = import
            .names
            .first()
            .cloned()
            .unwrap_or_else(|| import.specifier.clone());
        out.helpers.push(SourceSymbol {
            path: path.to_owned(),
            name,
            start_line: Some(import.span.line),
            end_line: Some(import.span.end_line),
        });
    }
    out
}

fn is_reserved(name: &str) -> bool {
    matches!(
        name,
        "generateMetadata" | "metadata" | "generateStaticParams" | "viewport" | "revalidate"
    )
}

fn looks_like_json_ld(name: &str) -> bool {
    let lower = name.to_ascii_lowercase().replace(['-', '_'], "");
    lower.contains("jsonld") || lower.contains("structureddata")
}

#[cfg(test)]
mod tests {
    use super::inspect;

    #[test]
    fn finds_metadata_span() {
        let source = "export async function generateMetadata() { return { title: 'X' }; }\nexport default function Page() { return null; }\n";
        let producers = inspect("src/app/page.tsx", source);
        assert_eq!(
            producers.metadata.as_ref().map(|item| item.name.as_str()),
            Some("generateMetadata")
        );
        assert_eq!(
            producers.metadata.as_ref().and_then(|item| item.start_line),
            Some(1)
        );
        assert_eq!(
            producers.page.as_ref().map(|item| item.name.as_str()),
            Some("Page")
        );
    }
}
