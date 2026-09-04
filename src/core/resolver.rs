use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

/// Lightweight DNS proxy for *.test domains.
/// Resolves *.test queries to 127.0.0.1 without hosts file changes.
pub struct DnsProxy {
    listen_port: u16,
    projects: Arc<Mutex<HashMap<String, String>>>,
    running: Arc<Mutex<bool>>,
}

impl DnsProxy {
    pub fn new(listen_port: u16) -> Self {
        Self {
            listen_port,
            projects: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn add_project(&self, name: &str, ip: &str) {
        self.projects.lock().unwrap().insert(name.to_lowercase(), ip.to_string());
    }

    pub fn set_projects(&self, projects: HashMap<String, String>) {
        *self.projects.lock().unwrap() = projects;
    }

    pub fn start(&self) -> bool {
        let mut running = self.running.lock().unwrap();
        if *running {
            return true;
        }

        let addr = format!("127.0.0.1:{}", self.listen_port);
        let socket = match UdpSocket::bind(&addr) {
            Ok(s) => s,
            Err(_) => return false,
        };
        socket.set_read_timeout(Some(std::time::Duration::from_secs(1))).ok();

        *running = true;
        let projects = Arc::clone(&self.projects);
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            let mut buf = [0u8; 512];
            while *running.lock().unwrap() {
                match socket.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        if let Some(response) = handle_query(&buf[..len], &projects) {
                            let _ = socket.send_to(&response, addr);
                        }
                    }
                    Err(_) => continue,
                }
            }
        });

        true
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    pub fn listen_port(&self) -> u16 {
        self.listen_port
    }
}

fn handle_query(data: &[u8], projects: &Arc<Mutex<HashMap<String, String>>>) -> Option<Vec<u8>> {
    if data.len() < 12 {
        return None;
    }

    let tx_id = u16::from_be_bytes([data[0], data[1]]);
    let _flags = u16::from_be_bytes([data[2], data[3]]);
    let qdcount = u16::from_be_bytes([data[4], data[5]]);

    // Parse question
    let (qname, offset) = parse_qname(data, 12)?;
    if offset + 4 > data.len() {
        return None;
    }
    let qtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
    let qclass = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);

    // Check if it's a .test domain
    let parts: Vec<&str> = qname.trim_end_matches('.').split('.').collect();
    if parts.len() < 2 || parts.last() != Some(&"test") {
        return None;
    }

    let project_name = parts[parts.len() - 2].to_lowercase();
    let ip = {
        let projs = projects.lock().unwrap();
        projs.get(&project_name).cloned().unwrap_or_else(|| "127.0.0.1".to_string())
    };

    // Build response
    let response_flags: u16 = 0x8180; // Standard response, no error
    let mut response = Vec::new();

    // Header
    response.extend_from_slice(&tx_id.to_be_bytes());
    response.extend_from_slice(&response_flags.to_be_bytes());
    response.extend_from_slice(&qdcount.to_be_bytes()); // question count
    response.extend_from_slice(&1u16.to_be_bytes());    // answer count
    response.extend_from_slice(&0u16.to_be_bytes());    // ns count
    response.extend_from_slice(&0u16.to_be_bytes());    // ar count

    // Question (copy original)
    response.extend_from_slice(data);

    // Answer
    response.extend_from_slice(&pack_name(&qname));
    response.extend_from_slice(&qtype.to_be_bytes());
    response.extend_from_slice(&qclass.to_be_bytes());
    response.extend_from_slice(&300u32.to_be_bytes()); // TTL
    response.extend_from_slice(&4u16.to_be_bytes());   // RDLENGTH

    // IP address
    for octet in ip.split('.') {
        if let Ok(n) = octet.parse::<u8>() {
            response.push(n);
        }
    }

    Some(response)
}

fn parse_qname(data: &[u8], mut offset: usize) -> Option<(String, usize)> {
    let mut parts = Vec::new();
    loop {
        if offset >= data.len() {
            return None;
        }
        let len = data[offset] as usize;
        if len == 0 {
            offset += 1;
            break;
        }
        if len >= 192 {
            // Pointer
            if offset + 2 > data.len() { return None; }
            let pointer = u16::from_be_bytes([data[offset], data[offset + 1]]) & 0x3FFF;
            offset += 2;
            if let Some((name, _)) = parse_qname(data, pointer as usize) {
                parts.push(name);
            }
            break;
        }
        offset += 1;
        if offset + len > data.len() { return None; }
        let label = String::from_utf8_lossy(&data[offset..offset + len]).to_string();
        parts.push(label);
        offset += len;
    }
    Some((format!("{}.", parts.join(".")), offset))
}

fn pack_name(name: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for part in name.trim_end_matches('.').split('.') {
        let bytes = part.as_bytes();
        result.push(bytes.len() as u8);
        result.extend_from_slice(bytes);
    }
    result.push(0);
    result
}
