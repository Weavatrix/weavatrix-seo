//! AI-crawler surface: `/llms.txt` and robots groups for known agents.

use weavatrix_seo_model::{
    AiAgentRole, Evidence, Finding, FindingFamily, Inventory, Locator, RuleAuthority, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let Some(surface) = &inventory.ai_surface else {
        return;
    };
    if surface.llms_txt_status == Some(404) {
        let host = inventory
            .site
            .clone()
            .unwrap_or_else(|| inventory.hosts.first().cloned().unwrap_or_default());
        findings.push(
            Finding::from_rule(
                FindingFamily::Ai,
                4,
                &host,
                format!("{host} has no /llms.txt"),
                Locator::Url(format!("{}/llms.txt", host.trim_end_matches('/'))),
                Evidence::http(),
            )
            .with_authority(RuleAuthority::ExperimentalHeuristic)
            .explained(
                "llms.txt is an emerging AI-crawler hint, not a search-engine requirement.",
                "Publish /llms.txt only when the origin wants to describe itself to AI fetchers.",
                "GET /llms.txt returns 200, or the origin intentionally has none.",
            ),
        );
    }
    for agent in &surface.robots_disallow_all {
        let host = inventory
            .site
            .clone()
            .unwrap_or_else(|| inventory.hosts.first().cloned().unwrap_or_default());
        let definition = weavatrix_seo_model::ai_agent(agent);
        let impact = definition.map_or("unclassified", |item| item.search_visibility_effect);
        let role = definition.and_then(|item| item.roles.first()).copied();
        let role_label = match role {
            Some(AiAgentRole::SearchDiscovery) => "search_discovery",
            Some(AiAgentRole::CitationFetch) => "citation_fetch",
            Some(AiAgentRole::UserInitiatedFetch) => "user_fetch",
            Some(AiAgentRole::Training) => "training",
            Some(AiAgentRole::GroundingControl) => "grounding_control",
            Some(AiAgentRole::Archive) => "archive",
            _ => "other",
        };
        let intent = surface
            .agent_matrix
            .iter()
            .find(|row| row.agent == *agent)
            .map_or("BLOCK", |row| row.policy_intent.as_str());
        let mut finding = Finding::from_rule(
            FindingFamily::Ai,
            5,
            &host,
            format!(
                "{host} robots.txt disallows `{agent}` ({role_label}; intent: {intent}; impact: {impact}) from the whole origin"
            ),
            Locator::Url(format!("{}/robots.txt", host.trim_end_matches('/'))),
            Evidence::http(),
        )
        .explained(
            "AI crawler tokens have different roles. Training disallow is not a Google Search indexing change.",
            "Allow discovery/citation agents on the public surface, or keep the block if that is the policy.",
            "The agent group is not Disallow: / unless the origin intends to opt out of that role.",
        );
        if !matches!(role, Some(AiAgentRole::SearchDiscovery)) {
            finding = finding.with_severity_override(Severity::Info);
        }
        findings.push(finding);
    }
}
