//! Loopback HTTP fixture for site-only audits.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Unique scratch directory. Process id alone collides under parallel tests.
#[must_use]
#[allow(dead_code)]
pub fn unique_temp(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}-{seq}", std::process::id()))
}

pub struct Page {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub struct Site {
    pub base: String,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for Site {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Ok(mut stream) =
            std::net::TcpStream::connect(self.base.trim_start_matches("http://"))
        {
            let _ = stream.write_all(b"GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Starts a loopback HTTP server.
///
/// # Panics
///
/// Panics when the local listener cannot bind.
#[must_use]
pub fn spawn(pages: BTreeMap<String, Page>) -> Site {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("nonblocking");
    let addr = listener.local_addr().expect("addr");
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        loop {
            if flag.load(Ordering::SeqCst) {
                break;
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    serve_one(&mut stream, &pages);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    });
    Site {
        base: format!("http://{addr}"),
        stop,
        handle: Some(handle),
    }
}

fn serve_one(stream: &mut std::net::TcpStream, pages: &BTreeMap<String, Page>) {
    let mut buffer = [0_u8; 4096];
    let _ = stream.read(&mut buffer);
    let request = String::from_utf8_lossy(&buffer);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let path = path.split('?').next().unwrap_or(path);
    let page = pages.get(path);
    let (status, headers, body) = match page {
        Some(page) => (page.status, page.headers.as_slice(), page.body.as_str()),
        None => (404, &[][..], "missing"),
    };
    let reason = match status {
        301 => "Moved Permanently",
        404 => "Not Found",
        500 => "Error",
        _ => "OK",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        let _ = write!(response, "{name}: {value}\r\n");
    }
    response.push_str("\r\n");
    response.push_str(body);
    let _ = stream.write_all(response.as_bytes());
}

#[must_use]
pub fn html(title: &str, extra_head: &str, body: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><title>{title}</title>{extra_head}</head><body>{body}</body></html>"
    )
}

#[must_use]
pub fn page(status: u16, body: impl Into<String>) -> Page {
    Page {
        status,
        headers: vec![("Content-Type".into(), "text/html".into())],
        body: body.into(),
    }
}
