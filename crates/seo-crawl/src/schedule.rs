//! Worker batching over the frontier.

use crate::frontier::Frontier;
use crate::{CrawlError, FetchResponse, Fetcher, Result, Robots};
use std::thread;
use weavatrix_seo_model::AbsoluteUrl;

pub fn pop_allowed(
    frontier: &mut Frontier,
    robots: &Robots,
    count: usize,
) -> Vec<(AbsoluteUrl, u32)> {
    frontier
        .pop_batch(count)
        .into_iter()
        .filter(|(url, _)| robots.allows(url))
        .collect()
}

pub fn fetch_batch(
    fetcher: &Fetcher,
    batch: Vec<(AbsoluteUrl, u32)>,
) -> Vec<(AbsoluteUrl, u32, Result<FetchResponse>)> {
    if batch.len() <= 1 {
        return batch
            .into_iter()
            .map(|(url, depth)| {
                let response = fetcher.get(&url).map_err(CrawlError::from);
                (url, depth, response)
            })
            .collect();
    }
    thread::scope(|scope| {
        let jobs: Vec<_> = batch
            .into_iter()
            .map(|(url, depth)| {
                let fetch_url = url.clone();
                let handle = scope.spawn(move || fetcher.get(&fetch_url).map_err(CrawlError::from));
                (url, depth, handle)
            })
            .collect();
        jobs.into_iter()
            .map(|(url, depth, handle)| {
                let response = handle.join().unwrap_or_else(|_| {
                    Err(CrawlError::Transport(format!(
                        "worker panicked fetching {url}"
                    )))
                });
                (url, depth, response)
            })
            .collect()
    })
}
