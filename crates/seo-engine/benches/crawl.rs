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
use weavatrix_seo::{AnalysisMode, AuditRequest, run_audit};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let pages = pages();
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
    let site = format!("http://{addr}/");
    let started = Instant::now();
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Site,
        site: Some(site),
        repo: None,
        competitors: Vec::new(),
        max_pages: Some(32),
    })
    .expect("audit");
    stop.store(true, Ordering::SeqCst);
    let elapsed = started.elapsed();
    println!(
        "weavatrix-seo site-only {} pages in {elapsed:?}",
        report.inventory.counts.crawled
    );
    black_box(report);
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
