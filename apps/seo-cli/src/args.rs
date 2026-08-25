//! Argv split and [`AuditRequest`] construction.

use std::collections::BTreeMap;
use weavatrix_seo::{AnalysisMode, AuditRequest};

pub type ParsedArgs = (String, BTreeMap<String, String>, Vec<String>);

pub fn request(
    command: &str,
    flags: &BTreeMap<String, String>,
    _positionals: &[String],
) -> Result<AuditRequest, String> {
    let site = flags.get("site").cloned();
    let repo = flags.get("repo").cloned();
    let max_pages = flags
        .get("max-pages")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "invalid --max-pages".to_owned())
        })
        .transpose()?;
    let workers = flags
        .get("workers")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "invalid --workers".to_owned())
        })
        .transpose()?;
    let ci = flags.contains_key("ci");
    let baseline = flags.get("baseline").cloned();
    let allow_private = !flags.contains_key("public-only");
    let gsc = flags.get("gsc").cloned();
    let observations = flags.get("observations").cloned();
    let history = flags.get("history").cloned();
    let render = flags.get("render").cloned();
    let competitors = flags
        .get("competitor")
        .map(|value| {
            value
                .lines()
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mode = if command == "compare" {
        AnalysisMode::Compare
    } else if site.is_some() && repo.is_some() {
        AnalysisMode::Hybrid
    } else if repo.is_some() {
        AnalysisMode::Repo
    } else {
        AnalysisMode::Site
    };
    if site.is_none() && repo.is_none() {
        return Err("provide --site URL and/or --repo PATH".into());
    }
    Ok(AuditRequest {
        mode,
        site,
        repo,
        competitors,
        max_pages,
        workers,
        ci,
        baseline,
        allow_private,
        gsc,
        observations,
        history,
        render,
    })
}

pub fn split(args: &[String]) -> Result<ParsedArgs, String> {
    if args.is_empty() {
        return Err(super::usage());
    }
    let command = args[0].clone();
    if !matches!(
        command.as_str(),
        "audit" | "inventory" | "opportunities" | "plan" | "compare" | "explain" | "diff"
    ) {
        return Err(format!("unknown command `{command}`\n{}", super::usage()));
    }
    let mut flags = BTreeMap::new();
    let mut positionals = Vec::new();
    let mut index = 1;
    while index < args.len() {
        let item = &args[index];
        if item == "--json" || item == "--ci" || item == "--public-only" {
            flags.insert(item.trim_start_matches('-').to_owned(), "true".into());
            index += 1;
            continue;
        }
        if let Some(name) = item.strip_prefix("--") {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| format!("flag --{name} requires a value"))?;
            if name == "competitor" {
                let mut current: String = flags.remove("competitor").unwrap_or_default();
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(value);
                flags.insert("competitor".into(), current);
            } else if flags.insert(name.to_owned(), value.clone()).is_some() {
                return Err(format!("flag --{name} was supplied more than once"));
            }
            index += 1;
            continue;
        }
        positionals.push(item.clone());
        index += 1;
    }
    Ok((command, flags, positionals))
}
