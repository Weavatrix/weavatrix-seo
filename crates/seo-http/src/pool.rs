//! Idle keep-alive sockets, one checkout per worker.

use crate::origin::Origin;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Mutex;

/// Pooled TCP or TLS stream.
pub enum Conn {
    /// Plain HTTP.
    Plain(TcpStream),
    /// TLS HTTP.
    #[cfg(feature = "tls")]
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

impl Read for Conn {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            #[cfg(feature = "tls")]
            Self::Tls(stream) => stream.read(buf),
        }
    }
}

impl Write for Conn {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buf),
            #[cfg(feature = "tls")]
            Self::Tls(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            #[cfg(feature = "tls")]
            Self::Tls(stream) => stream.flush(),
        }
    }
}

/// Origin-keyed idle pool.
pub struct Pool {
    cap: usize,
    inner: Mutex<BTreeMap<Origin, Vec<Conn>>>,
}

impl Pool {
    /// Builds a pool that retains at most `cap` idle sockets.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            inner: Mutex::new(BTreeMap::new()),
        }
    }

    /// Takes an idle socket for `origin`.
    pub fn checkout(&self, origin: &Origin) -> Option<Conn> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_mut(origin)
            .and_then(Vec::pop)
    }

    /// Returns a live socket, or drops it when the pool is full.
    pub fn checkin(&self, origin: Origin, conn: Conn) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let idle: usize = inner.values().map(Vec::len).sum();
        if idle >= self.cap {
            return;
        }
        inner.entry(origin).or_default().push(conn);
    }
}
