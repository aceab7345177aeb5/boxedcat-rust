use std::net::TcpStream;
use std::time::Duration;

/// Port range for a service — primary + fallbacks.
#[derive(Debug, Clone)]
pub struct PortRange {
    pub service: &'static str,
    pub ports: &'static [u16],
}

impl PortRange {
    pub const fn new(service: &'static str, ports: &'static [u16]) -> Self {
        Self { service, ports }
    }

    pub fn primary(&self) -> u16 {
        self.ports[0]
    }
}

/// Port succession table — if primary is busy, try the next fallback.
pub const PORT_SUCCESSION: &[PortRange] = &[
    PortRange::new("http", &[80, 8080, 8000, 8088]),
    PortRange::new("https", &[443, 4433, 4443, 8443]),
    PortRange::new("mariadb", &[3306, 3307, 3308, 3309]),
    PortRange::new("mysql", &[3306, 3307, 3308, 3309]),
    PortRange::new("postgresql", &[5432, 5433, 5434, 5435]),
    PortRange::new("redis", &[6379, 6380, 6381, 6382]),
    PortRange::new("valkey", &[6379, 6380, 6381, 6382]),
    PortRange::new("nginx", &[80, 8080, 8000, 8088]),
    PortRange::new("tomcat", &[8080, 8443, 8081, 8082]),
    PortRange::new("deno", &[8000, 8001, 8002, 8003]),
];

/// Check if a port is available by attempting a TCP bind.
pub fn is_port_available(port: u16) -> bool {
    TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(100),
    )
    .is_err() // Connection refused = port is free
}

/// Find the first available port for a service.
pub fn find_available_port(service: &str) -> Option<u16> {
    PORT_SUCCESSION
        .iter()
        .find(|r| r.service == service)
        .and_then(|range| range.ports.iter().copied().find(|&p| is_port_available(p)))
}

/// Get availability status for all ports in a service's succession list.
pub fn get_port_status(service: &str) -> Vec<(u16, bool)> {
    PORT_SUCCESSION
        .iter()
        .find(|r| r.service == service)
        .map(|range| range.ports.iter().map(|&p| (p, is_port_available(p))).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_succession_table() {
        assert_eq!(PORT_SUCCESSION.len(), 10);
        assert_eq!(PORT_SUCCESSION[0].primary(), 80);
    }

    #[test]
    fn test_find_available_port() {
        // At least one port in the succession list should be available
        let port = find_available_port("redis");
        assert!(port.is_some());
    }
}
