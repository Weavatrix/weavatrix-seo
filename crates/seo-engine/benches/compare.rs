//! Compare two loopback origins and optional commercial crawler binaries.
//!
//! Commercial crawler names belong only in this bench tree.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::hint::black_box;
use std::io::{Read, Write as IoWrite};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use weavatrix_seo::{AuditRequest, run_audit};
use weavatrix_seo_competitor::{compare_inventories, score_artifacts, tally};

fn main() {
    let owned = serve(owned_pages());
    let other = serve(competitor_pages());
    let started = Instant::now();
    let ours = run_audit(&AuditRequest {
        site: Some(owned.site.clone()),
        max_pages: Some(16),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("owned");
    let theirs = run_audit(&AuditRequest {
        site: Some(other.site.clone()),
        max_pages: Some(24),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("competitor");
    let crawl_elapsed = started.elapsed();
    let started = Instant::now();
    let gaps = compare_inventories(
        &ours.inventory,
        &[(other.site.clone(), theirs.inventory.clone())],
    );
    println!(
        "weavatrix-seo compare crawl in {crawl_elapsed:?}; shape-diff {} gaps in {:?}",
        gaps.len(),
        started.elapsed()
    );
    for kind in [
        "schema_gap",
        "market_gap",
        "cluster_gap",
        "content_gap",
        "link_gap",
    ] {
        let count = gaps.iter().filter(|item| item.kind == kind).count();
        println!("  {kind}={count}");
    }
    let (have, total) = tally(&ours);
    println!("weavatrix-seo first-party {have}/{total} versus a URL-list crawler");
    for item in score_artifacts(&ours) {
        if weavatrix_seo_competitor::site_backed_ids().contains(&item.id) {
            println!("  {} present={}", item.id, item.present);
        }
    }
    probe_external_crawlers(&owned.site);
    black_box(gaps);
    owned.stop.store(true, Ordering::SeqCst);
    other.stop.store(true, Ordering::SeqCst);
}

struct Origin {
    site: String,
    stop: Arc<AtomicBool>,
}

fn serve(pages: BTreeMap<String, String>) -> Origin {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    thread::spawn(move || {
        listener.set_nonblocking(true).expect("nonblocking");
        while !flag.load(Ordering::SeqCst) {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0_u8; 2048];
                let _ = stream.read(&mut buffer);
                let request = String::from_utf8_lossy(&buffer);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");
                let body = pages.get(path).map_or("", String::as_str);
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            } else {
                thread::sleep(std::time::Duration::from_millis(2));
            }
        }
    });
    Origin {
        site: format!("http://{addr}/"),
        stop,
    }
}

fn probe_external_crawlers(site: &str) {
    for binary in ["siteone-crawler", "screamingfrogseospider"] {
        match std::process::Command::new(binary).arg("--help").output() {
            Ok(output) if output.status.success() => {
                let started = Instant::now();
                let run = std::process::Command::new(binary).arg(site).output();
                println!(
                    "{binary} present; spawn {:?} in {:?}",
                    run.map(|item| item.status.code()),
                    started.elapsed()
                );
            }
            _ => {
                println!(
                    "{binary} not installed; skip baseline (URL list only, no evidence graph)"
                );
            }
        }
    }
}

fn owned_pages() -> BTreeMap<String, String> {
    let mut pages = BTreeMap::new();
    pages.insert("/robots.txt".into(), "User-agent: *\nAllow: /\n".into());
    pages.insert(
        "/".into(),
        "<html lang=\"en\"><head><title>Home</title></head>\
         <body><h1>Home</h1><p>Owned electrician.</p><a href=\"/service/one\">one</a></body></html>"
            .into(),
    );
    pages.insert(
        "/service/one".into(),
        "<html><head><title>One</title></head><body><p>No H1.</p></body></html>".into(),
    );
    pages
}

fn competitor_pages() -> BTreeMap<String, String> {
    let mut pages = BTreeMap::new();
    pages.insert("/robots.txt".into(), "User-agent: *\nAllow: /\n".into());
    let mut links = String::from(
        "<html lang=\"he-IL\"><head><title>Home</title>\
         <link rel=\"alternate\" hreflang=\"he-IL\" href=\"/\">\
         <script type=\"application/ld+json\">{\"@type\":\"FAQPage\"}</script></head>\
         <body><h1>Home</h1><a href=\"/faq\">faq</a>",
    );
    pages.insert(
        "/faq".into(),
        "<html><head><title>FAQ</title></head><body><h1>FAQ</h1></body></html>".into(),
    );
    for index in 0..6 {
        let _ = write!(links, "<a href=\"/service/{index}\">s{index}</a>");
        pages.insert(
            format!("/service/{index}"),
            format!(
                "<html><head><title>S{index}</title></head><body><h1>S{index}</h1></body></html>"
            ),
        );
    }
    links.push_str("</body></html>");
    pages.insert("/".into(), links);
    pages
}
