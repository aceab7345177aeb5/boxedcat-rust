use std::net::{UdpSocket, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

static RUNNING: AtomicBool = AtomicBool::new(false);

struct DnsHeader {
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

impl DnsHeader {
    fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 12 { return None; }
        Some(DnsHeader {
            id: u16::from_be_bytes([buf[0], buf[1]]),
            flags: u16::from_be_bytes([buf[2], buf[3]]),
            qdcount: u16::from_be_bytes([buf[4], buf[5]]),
            ancount: u16::from_be_bytes([buf[6], buf[7]]),
            nscount: u16::from_be_bytes([buf[8], buf[9]]),
            arcount: u16::from_be_bytes([buf[10], buf[11]]),
        })
    }

    fn is_query(&self) -> bool {
        (self.flags & 0x8000) == 0
    }
}

fn parse_query_name(buf: &[u8], offset: usize) -> Option<(String, usize)> {
    let mut name = String::new();
    let mut pos = offset;
    loop {
        if pos >= buf.len() { return None; }
        let len = buf[pos] as usize;
        if len == 0 {
            pos += 1;
            break;
        }
        if pos + 1 + len > buf.len() { return None; }
        if !name.is_empty() { name.push('.'); }
        name.push_str(&String::from_utf8_lossy(&buf[pos+1..pos+1+len]));
        pos += 1 + len;
    }
    Some((name, pos))
}

fn build_dns_response(header: &DnsHeader, query_name: &str, query_end: usize, original: &[u8]) -> Vec<u8> {
    let mut resp = Vec::with_capacity(512);

    let response_flags: u16 = 0x8180;
    resp.extend_from_slice(&header.id.to_be_bytes());
    resp.extend_from_slice(&response_flags.to_be_bytes());
    resp.extend_from_slice(&header.qdcount.to_be_bytes());

    let is_test = query_name.ends_with(".test");

    if is_test {
        resp.extend_from_slice(&1u16.to_be_bytes());
        resp.extend_from_slice(&0u16.to_be_bytes());
        resp.extend_from_slice(&0u16.to_be_bytes());
        resp.extend_from_slice(&0u16.to_be_bytes());
    } else {
        resp.extend_from_slice(&header.ancount.to_be_bytes());
        resp.extend_from_slice(&header.nscount.to_be_bytes());
        resp.extend_from_slice(&header.arcount.to_be_bytes());
    }

    resp.extend_from_slice(&original[12..query_end]);

    if is_test {
        resp.extend_from_slice(&0xC00Cu16.to_be_bytes());
        resp.extend_from_slice(&1u16.to_be_bytes());
        resp.extend_from_slice(&1u16.to_be_bytes());
        resp.extend_from_slice(&0u16.to_be_bytes());
        resp.extend_from_slice(&60u16.to_be_bytes());
        resp.extend_from_slice(&4u16.to_be_bytes());
        resp.extend_from_slice(&[127, 0, 0, 1]);
    }

    resp
}

fn handle_query(socket: &UdpSocket, buf: &[u8], src: SocketAddr, upstream: &str) {
    let header = match DnsHeader::parse(buf) {
        Some(h) => h,
        None => return,
    };

    if !header.is_query() { return; }

    let (name, query_end) = match parse_query_name(buf, 12) {
        Some(r) => r,
        None => return,
    };

    if name.ends_with(".test") {
        let resp = build_dns_response(&header, &name, query_end, buf);
        let _ = socket.send_to(&resp, src);
    } else {
        let upstream_addr: SocketAddr = match upstream.parse() {
            Ok(a) => a,
            Err(_) => {
                let fallback: SocketAddr = "8.8.8.8:53".parse().unwrap();
                fallback
            }
        };
        let _ = socket.send_to(buf, upstream_addr);
        let mut resp_buf = [0u8; 4096];
        socket.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();
        if let Ok((n, _)) = socket.recv_from(&mut resp_buf) {
            let _ = socket.send_to(&resp_buf[..n], src);
        }
    }
}

pub fn start_dns_proxy(listen_port: u16, upstream_dns: &str) -> bool {
    if RUNNING.load(Ordering::Relaxed) { return false; }

    let bind_addr = format!("0.0.0.0:{}", listen_port);
    let socket = match UdpSocket::bind(&bind_addr) {
        Ok(s) => s,
        Err(_) => return false,
    };

    if socket.set_nonblocking(true).is_err() { return false; }

    RUNNING.store(true, Ordering::Relaxed);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let upstream = upstream_dns.to_string();

    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        while r.load(Ordering::Relaxed) {
            match socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    handle_query(&socket, &buf[..n], src, &upstream);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });

    true
}

pub fn stop_dns_proxy() {
    RUNNING.store(false, Ordering::Relaxed);
}

pub fn is_dns_running() -> bool {
    RUNNING.load(Ordering::Relaxed)
}
