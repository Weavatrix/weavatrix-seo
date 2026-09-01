//! Policy packs. Kablay is the first fixture, not core engine behaviour.

/// Declared or inferred public market of a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    /// Southwest Washington / US.
    UsWa,
    /// Israel.
    Israel,
    /// Not classified.
    Unknown,
}

/// Named entity that belongs to one market pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityRule {
    /// Match token.
    pub token: &'static str,
    /// Display label.
    pub label: &'static str,
}

/// Public-language claim that requires a domain fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimRule {
    /// Rule id.
    pub id: &'static str,
    /// Phrases on the public surface.
    pub phrases: &'static [&'static str],
    /// Fact field this claim requires to be true.
    pub requires_fact: &'static str,
}

/// Domain fact extracted from source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactRule {
    /// Field name, for example `license_verified`.
    pub field: &'static str,
    /// Compact literals that mean false.
    pub false_literals: &'static [&'static str],
}

/// One jurisdiction/market policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyPack {
    /// Stable pack id.
    pub id: &'static str,
    /// Market this pack owns.
    pub market: Market,
    /// Jurisdiction label.
    pub jurisdiction: &'static str,
    /// Path or source markers that bind a file to this pack.
    pub markers: &'static [&'static str],
    /// Entities that belong here.
    pub entities: &'static [EntityRule],
    /// Public claims.
    pub claims: &'static [ClaimRule],
    /// Domain facts.
    pub facts: &'static [FactRule],
}

const US_WA_ENTITIES: &[EntityRule] = &[
    EntityRule {
        token: "Washington State L&I",
        label: "Washington L&I",
    },
    EntityRule {
        token: "Clark County",
        label: "Clark County",
    },
    EntityRule {
        token: "Southwest Washington",
        label: "Southwest Washington",
    },
    EntityRule {
        token: "Vancouver WA",
        label: "Vancouver WA",
    },
    EntityRule {
        token: "Vancouver, WA",
        label: "Vancouver WA",
    },
    EntityRule {
        token: "Camas WA",
        label: "Camas WA",
    },
    EntityRule {
        token: "Battle Ground WA",
        label: "Battle Ground WA",
    },
    EntityRule {
        token: "Ridgefield WA",
        label: "Ridgefield WA",
    },
    EntityRule {
        token: "L&I verify",
        label: "Washington L&I verify",
    },
];

const IL_ENTITIES: &[EntityRule] = &[
    EntityRule {
        token: "IEC",
        label: "Israel Electric Corporation",
    },
    EntityRule {
        token: "Hevrat HaHashmal",
        label: "Hevrat HaHashmal",
    },
    EntityRule {
        token: "Gush Dan",
        label: "Gush Dan",
    },
    EntityRule {
        token: "Shabbat",
        label: "Shabbat",
    },
    EntityRule {
        token: "Israel Electric",
        label: "Israel Electric",
    },
    EntityRule {
        token: "חשמלאי",
        label: "Hebrew electrician title",
    },
    EntityRule {
        token: "מוסמך",
        label: "Hebrew licensed-trade title",
    },
    EntityRule {
        token: "Negev",
        label: "Negev",
    },
    EntityRule {
        token: "Ministry of Energy",
        label: "Israeli energy regulator phrasing",
    },
];

const LICENSE_CLAIMS: &[ClaimRule] = &[
    ClaimRule {
        id: "license_verified",
        phrases: &[
            "license verified",
            "licenseverification",
            "licenseverified",
            "licensed professional",
            "licensed electrician",
            "licensed contractor",
            "document/license verification",
            "license verification badges",
        ],
        requires_fact: "license_verified",
    },
    ClaimRule {
        id: "insured",
        phrases: &["fully insured", "insured contractor", "liability insurance"],
        requires_fact: "insurance_verified",
    },
];

const LICENSE_FACTS: &[FactRule] = &[
    FactRule {
        field: "license_verified",
        false_literals: &[
            "license_verified:false",
            "license_verified=false",
            "licenseverified:false",
        ],
    },
    FactRule {
        field: "insurance_verified",
        false_literals: &[
            "insurance_verified:false",
            "insured:false",
            "insuranceverified:false",
        ],
    },
    FactRule {
        field: "years_experience",
        false_literals: &["years_experience:0", "years_experience:null"],
    },
];

/// Southwest Washington contractor marketplace. First fixture pack.
pub const US_WA: PolicyPack = PolicyPack {
    id: "marketplace.contractor.us-wa",
    market: Market::UsWa,
    jurisdiction: "US-WA",
    markers: &[
        "washington",
        "us-wa",
        "southwest washington",
        "clark county",
    ],
    entities: US_WA_ENTITIES,
    claims: LICENSE_CLAIMS,
    facts: LICENSE_FACTS,
};

/// Israeli contractor marketplace. Used as the foreign pack for US-WA.
pub const ISRAEL: PolicyPack = PolicyPack {
    id: "marketplace.contractor.il",
    market: Market::Israel,
    jurisdiction: "IL",
    markers: &["israel", "co.il", "gush dan", "hevrat"],
    entities: IL_ENTITIES,
    claims: &[],
    facts: &[],
};

/// All shipped packs.
#[must_use]
pub fn all() -> &'static [PolicyPack] {
    &[US_WA, ISRAEL]
}

/// Pack for a classified market.
#[must_use]
pub fn for_market(market: Market) -> Option<&'static PolicyPack> {
    all().iter().find(|pack| pack.market == market)
}

/// Whether a repository path/source belongs to this pack.
#[must_use]
pub fn file_belongs(pack: &PolicyPack, path: &str, source: &str) -> bool {
    let hay = format!("{path}\n{source}").to_ascii_lowercase();
    pack.markers.iter().any(|marker| hay.contains(marker))
}
