//! Declarative extra policy packs. Built-in Rust packs stay; these extend them.

use crate::pack::Market;
use std::path::Path;

/// One extra claim loaded from `.weavatrix/seo.pack.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedClaim {
    /// Claim id.
    pub id: String,
    /// Public phrases.
    pub phrases: Vec<String>,
    /// Required fact field.
    pub requires: String,
}

/// One extra fact loaded from a pack file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedFact {
    /// Field name.
    pub field: String,
    /// Compact literals that mean false.
    pub false_literals: Vec<String>,
}

/// One extra entity token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedEntity {
    /// Match token.
    pub token: String,
    /// Display label.
    pub label: String,
}

/// Owned pack. Same model as the shipped `'static` packs, loadable from YAML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedPack {
    /// Pack id.
    pub id: String,
    /// Market.
    pub market: Market,
    /// Jurisdiction label.
    pub jurisdiction: String,
    /// Path/source markers.
    pub markers: Vec<String>,
    /// Entities.
    pub entities: Vec<OwnedEntity>,
    /// Claims.
    pub claims: Vec<OwnedClaim>,
    /// Facts.
    pub facts: Vec<OwnedFact>,
}

/// Loads `.weavatrix/seo.pack.yaml` when present. Missing file is empty, not an error.
#[must_use]
pub fn load(repo: &str) -> Vec<OwnedPack> {
    let path = Path::new(repo).join(".weavatrix").join("seo.pack.yaml");
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    parse_yaml(&raw).into_iter().collect()
}

#[allow(clippy::too_many_lines)]
fn parse_yaml(raw: &str) -> Option<OwnedPack> {
    let mut pack = OwnedPack {
        id: String::new(),
        market: Market::Unknown,
        jurisdiction: String::new(),
        markers: Vec::new(),
        entities: Vec::new(),
        claims: Vec::new(),
        facts: Vec::new(),
    };
    let mut section = "";
    let mut current_claim: Option<OwnedClaim> = None;
    let mut current_fact: Option<OwnedFact> = None;
    let mut current_entity: Option<OwnedEntity> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            let item = item.trim().trim_matches('"');
            match section {
                "markers" => pack.markers.push(item.to_owned()),
                "phrases" => {
                    if let Some(claim) = current_claim.as_mut() {
                        claim.phrases.push(item.to_owned());
                    }
                }
                "false" => {
                    if let Some(fact) = current_fact.as_mut() {
                        fact.false_literals.push(compact(item));
                    }
                }
                "claims" if item.contains(':') => {
                    flush_claim(&mut pack, &mut current_claim);
                    let id = item
                        .strip_prefix("id:")
                        .unwrap_or(item)
                        .trim()
                        .trim_matches('"');
                    current_claim = Some(OwnedClaim {
                        id: id.to_owned(),
                        phrases: Vec::new(),
                        requires: String::new(),
                    });
                }
                "facts" if item.contains(':') => {
                    flush_fact(&mut pack, &mut current_fact);
                    let field = item
                        .strip_prefix("field:")
                        .or_else(|| item.strip_prefix("id:"))
                        .unwrap_or(item)
                        .trim()
                        .trim_matches('"');
                    current_fact = Some(OwnedFact {
                        field: field.to_owned(),
                        false_literals: Vec::new(),
                    });
                }
                "entities" => {
                    flush_entity(&mut pack, &mut current_entity);
                    current_entity = Some(OwnedEntity {
                        token: item.to_owned(),
                        label: item.to_owned(),
                    });
                }
                _ => {}
            }
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        match key {
            "id" if section.is_empty() || section == "pack" => value.clone_into(&mut pack.id),
            "jurisdiction" => value.clone_into(&mut pack.jurisdiction),
            "market" => {
                pack.market = match value.to_ascii_uppercase().as_str() {
                    "US-WA" | "USWA" => Market::UsWa,
                    "IL" | "ISRAEL" => Market::Israel,
                    _ => Market::Unknown,
                };
                if pack.jurisdiction.is_empty() {
                    value.clone_into(&mut pack.jurisdiction);
                }
            }
            "markers" => section = "markers",
            "claims" => {
                flush_claim(&mut pack, &mut current_claim);
                section = "claims";
            }
            "facts" => {
                flush_fact(&mut pack, &mut current_fact);
                section = "facts";
            }
            "entities" => {
                flush_entity(&mut pack, &mut current_entity);
                section = "entities";
            }
            "phrases" => section = "phrases",
            "false" | "false_literals" => section = "false",
            "requires" | "requires_fact" => {
                if let Some(claim) = current_claim.as_mut() {
                    value.clone_into(&mut claim.requires);
                }
            }
            "field" if section == "facts" => {
                flush_fact(&mut pack, &mut current_fact);
                current_fact = Some(OwnedFact {
                    field: value.to_owned(),
                    false_literals: Vec::new(),
                });
            }
            "token" => {
                if let Some(entity) = current_entity.as_mut() {
                    value.clone_into(&mut entity.token);
                }
            }
            "label" => {
                if let Some(entity) = current_entity.as_mut() {
                    value.clone_into(&mut entity.label);
                }
            }
            _ => {}
        }
    }
    flush_claim(&mut pack, &mut current_claim);
    flush_fact(&mut pack, &mut current_fact);
    flush_entity(&mut pack, &mut current_entity);
    if pack.id.is_empty() { None } else { Some(pack) }
}

fn flush_claim(pack: &mut OwnedPack, current: &mut Option<OwnedClaim>) {
    if let Some(claim) = current.take()
        && !claim.id.is_empty()
    {
        pack.claims.push(claim);
    }
}

fn flush_fact(pack: &mut OwnedPack, current: &mut Option<OwnedFact>) {
    if let Some(fact) = current.take()
        && !fact.field.is_empty()
    {
        pack.facts.push(fact);
    }
}

fn flush_entity(pack: &mut OwnedPack, current: &mut Option<OwnedEntity>) {
    if let Some(entity) = current.take()
        && !entity.token.is_empty()
    {
        pack.entities.push(entity);
    }
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::parse_yaml;
    use crate::pack::Market;

    #[test]
    fn reads_a_minimal_pack() {
        let raw = r"
id: marketplace.contractor.us-wa
market: US-WA
claims:
  - id: licensed_contractor
    phrases:
      - licensed contractor
    requires: license_verified
facts:
  - field: license_verified
    false:
      - license_verified:false
";
        let pack = parse_yaml(raw).expect("pack");
        assert_eq!(pack.id, "marketplace.contractor.us-wa");
        assert_eq!(pack.market, Market::UsWa);
        assert_eq!(pack.claims[0].requires, "license_verified");
        assert_eq!(pack.facts[0].false_literals[0], "license_verified:false");
    }
}
