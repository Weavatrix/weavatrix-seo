//! Query DSL and retrieve throughput after a loopback audit.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::hint::black_box;
use std::io::{Read, Write as IoWrite};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use weavatrix_seo::{AuditRequest, retrieve, run_audit, run_on_report};

fn main() {
    let origin = serve(pages());
    let report = run_audit(&AuditRequest {
        site: Some(origin.site.clone()),
        max_pages: Some(24),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("audit");
    let started = Instant::now();
    let mut total_rows = 0;
    for _ in 0..32 {
        let result = run_on_report(
            "FROM urls WHERE indexable = true RETURN url, inbound_links LIMIT 20",
            &report,
        )
        .expect("query");
        total_rows += result.rows.len();
        black_box(result);
    }
    let query_elapsed = started.elapsed();
    let started = Instant::now();
    let mut hits = 0;
    for query in [
        "electrician vancouver",
        "home page",
        "licensed permit",
        "service city",
    ] {
        let found = retrieve(&report, query, 8);
        hits += found.len();
        black_box(found);
    }
    println!(
        "weavatrix-seo query 32x rows={total_rows} in {query_elapsed:?}; retrieve 4q hits={hits} in {:?}",
        started.elapsed()
    );
    origin.stop.store(true, Ordering::SeqCst);
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

fn pages() -> BTreeMap<String, String> {
    let mut pages = BTreeMap::new();
    pages.insert("/robots.txt".into(), "User-agent: *\nAllow: /\n".into());
    let mut links = String::new();
    for index in 0..12 {
        let _ = write!(links, "<a href=\"/p{index}\">p{index}</a>");
        pages.insert(
            format!("/p{index}"),
            format!(
                "<html lang=\"en\"><head><title>P{index}</title></head>\
                 <body><h1>Electrician {index}</h1><p>Licensed electrician city {index} permit.</p></body></html>"
            ),
        );
    }
    pages.insert(
        "/".into(),
        format!("<html><head><title>Home</title></head><body><h1>Home</h1>{links}</body></html>"),
    );
    pages
}
