//! DNS / timeout / body-limit stay typed failures.

use std::time::Duration;
use weavatrix_seo_http::{FetchBudget, Fetcher, HttpError, NetworkPolicy};
use weavatrix_seo_model::{AbsoluteUrl, FetchOutcome};

#[test]
fn dns_failure_is_dns_outcome() {
    let url = AbsoluteUrl::parse("http://no-such-host-wvx-seo.test/").unwrap();
    let error = Fetcher::new(FetchBudget {
        timeout: Duration::from_millis(400),
        ..FetchBudget::default()
    })
    .get(&url)
    .expect_err("dns");
    assert_eq!(error.outcome(), FetchOutcome::Dns);
}

#[test]
fn timeout_is_timeout_outcome() {
    let url = AbsoluteUrl::parse("http://192.0.2.1/").unwrap();
    let error = Fetcher::new(FetchBudget {
        timeout: Duration::from_millis(200),
        policy: NetworkPolicy::public_only(),
        max_retries: 0,
        ..FetchBudget::default()
    })
    .get(&url)
    .expect_err("timeout");
    assert!(
        matches!(error, HttpError::Timeout(_) | HttpError::Transport(_)),
        "{error}"
    );
    assert!(
        matches!(
            error.outcome(),
            FetchOutcome::Timeout | FetchOutcome::Transport
        ),
        "{error:?}"
    );
}

#[test]
fn public_policy_blocks_metadata() {
    let url = AbsoluteUrl::parse("http://169.254.169.254/latest/meta-data").unwrap();
    let error = Fetcher::new(FetchBudget {
        policy: NetworkPolicy::public_only(),
        ..FetchBudget::default()
    })
    .get(&url)
    .expect_err("blocked");
    assert_eq!(error.outcome(), FetchOutcome::Blocked);
}
