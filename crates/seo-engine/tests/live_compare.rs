//! Optional live compare. Off unless `WEAVATRIX_SEO_LIVE=1`.
//!
//! Public competitor origins are crawled public-only. Bot protection is
//! unmeasured, not a failure: the test records the structural gaps we could
//! prove and refuses to copy page text.

use weavatrix_seo::{AuditRequest, run_audit};
use weavatrix_seo_competitor::compare_inventories;

#[test]
fn kablay_us_versus_public_marketplace() {
    if std::env::var("WEAVATRIX_SEO_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let owned = run_audit(&AuditRequest {
        site: Some("https://kablay.us/".into()),
        max_pages: Some(12),
        workers: Some(4),
        allow_private: false,
        ..AuditRequest::default()
    });
    let Ok(owned) = owned else {
        eprintln!("live owned origin unmeasured");
        return;
    };
    if owned.inventory.pages.is_empty() {
        eprintln!("live owned origin returned no pages");
        return;
    }
    let competitor = run_audit(&AuditRequest {
        site: Some("https://www.thumbtack.com/".into()),
        max_pages: Some(8),
        workers: Some(4),
        allow_private: false,
        ..AuditRequest::default()
    });
    let Ok(competitor) = competitor else {
        eprintln!("live competitor origin unmeasured (transport or policy)");
        return;
    };
    let items = compare_inventories(
        &owned.inventory,
        &[(
            "https://www.thumbtack.com/".into(),
            competitor.inventory.clone(),
        )],
    );
    println!(
        "live compare: owned {} pages, competitor {} pages, {} structural gaps",
        owned.inventory.pages.len(),
        competitor.inventory.pages.len(),
        items.len()
    );
    for item in items.iter().take(12) {
        println!("  {} {} {}", item.kind, item.subject, item.summary);
    }
    assert!(
        items.iter().all(|item| !item.summary.contains("<html")
            && !item.why.contains("<script")
            && !item.action.contains("thumbtack.com/")),
        "live compare must stay structural: {items:?}"
    );
}

#[test]
fn kablay_us_versus_kablay_il() {
    if std::env::var("WEAVATRIX_SEO_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let owned = run_audit(&AuditRequest {
        site: Some("https://kablay.us/".into()),
        max_pages: Some(12),
        workers: Some(4),
        allow_private: false,
        ..AuditRequest::default()
    });
    let other = run_audit(&AuditRequest {
        site: Some("https://kablay.co.il/".into()),
        max_pages: Some(12),
        workers: Some(4),
        allow_private: false,
        ..AuditRequest::default()
    });
    let (Ok(owned), Ok(other)) = (owned, other) else {
        eprintln!("live kablay origins unmeasured");
        return;
    };
    if owned.inventory.pages.is_empty() || other.inventory.pages.is_empty() {
        eprintln!(
            "live kablay pages owned={} other={}",
            owned.inventory.pages.len(),
            other.inventory.pages.len()
        );
        return;
    }
    let items = compare_inventories(
        &owned.inventory,
        &[("https://kablay.co.il/".into(), other.inventory.clone())],
    );
    println!(
        "kablay.us vs kablay.co.il: {} vs {} pages, {} structural gaps",
        owned.inventory.pages.len(),
        other.inventory.pages.len(),
        items.len()
    );
    for item in items.iter().take(16) {
        println!("  {} {} {}", item.kind, item.subject, item.summary);
    }
}
