//! First-party crawl throughput.
//!
//! Commercial crawler names belong only in this bench tree, and only as
//! optional external baselines. This file measures Weavatrix SEO itself.

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

fn main() {
    let a = serve(pages());
    let b = serve(alt_pages());
    let report = measure("origin-a", &a.site, 32);
    measure("origin-b", &b.site, 8);
    measure_query(&report);
    probe_external_crawlers(&a.site);
    print_first_party(&report);
    println!("live fixture origins are not probed from this bench");
    a.stop.store(true, Ordering::SeqCst);
    b.stop.store(true, Ordering::SeqCst);
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

fn measure(label: &str, site: &str, max_pages: usize) -> weavatrix_seo::AuditReport {
    let started = Instant::now();
    let report = run_audit(&AuditRequest {
        site: Some(site.to_owned()),
        max_pages: Some(max_pages),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("audit");
    let elapsed = started.elapsed();
    println!(
        "weavatrix-seo {label} pages={} findings={} opportunities={} in {elapsed:?}",
        report.inventory.counts.crawled,
        report.findings.len(),
        report.opportunities.len()
    );
    black_box(report.clone());
    report
}

fn measure_query(report: &weavatrix_seo::AuditReport) {
    let started = Instant::now();
    let rows = weavatrix_seo::run_on_report(
        "FROM urls WHERE indexable = true RETURN url, inbound_links LIMIT 50",
        report,
    )
    .expect("query");
    let hits = weavatrix_seo::retrieve(report, "home page", 8);
    println!(
        "weavatrix-seo query+retrieve rows={} hits={} in {:?}",
        rows.rows.len(),
        hits.len(),
        started.elapsed()
    );
    black_box((rows, hits));
}

fn print_first_party(report: &weavatrix_seo::AuditReport) {
    let scored = weavatrix_seo_competitor::score_artifacts(report);
    let (have, total) = weavatrix_seo_competitor::tally(report);
    println!("weavatrix-seo first-party artifacts {have}/{total}");
    for item in scored {
        println!(
            "  artifact {:<24} present={:<5} {}",
            item.id, item.present, item.note
        );
    }
}

fn probe_external_crawlers(site: &str) {
    // Names of other crawlers belong only in this bench tree.
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
            _ => println!("{binary} not installed; skip baseline"),
        }
    }
}

fn pages() -> BTreeMap<String, String> {
    let mut pages = BTreeMap::new();
    pages.insert("/robots.txt".into(), "User-agent: *\nAllow: /\n".into());
    pages.insert(
        "/sitemap.xml".into(),
        "<?xml version=\"1.0\"?><urlset></urlset>".into(),
    );
    let mut links = String::new();
    for index in 0..16 {
        let _ = write!(links, "<a href=\"/p{index}\">p{index}</a>");
        pages.insert(
            format!("/p{index}"),
            format!(
                "<html><head><title>P{index}</title></head><body><h1>P{index}</h1></body></html>"
            ),
        );
    }
    pages.insert(
        "/".into(),
        format!("<html><head><title>Home</title></head><body>{links}</body></html>"),
    );
    pages
}

fn alt_pages() -> BTreeMap<String, String> {
    let mut pages = BTreeMap::new();
    pages.insert("/robots.txt".into(), "User-agent: *\nAllow: /\n".into());
    pages.insert(
        "/".into(),
        "<html><head><title>Alt</title></head><body><h1>Alt</h1><a href=\"/x\">x</a></body></html>"
            .into(),
    );
    pages.insert(
        "/x".into(),
        "<html><head><title>X</title></head><body><h1>X</h1></body></html>".into(),
    );
    pages
}
