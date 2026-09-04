/// Version registry — LTS-only policy, CVE-aware.

#[derive(Debug, Clone)]
pub struct ComponentVersion {
    pub name: &'static str,
    pub version: &'static str,
    pub channel: &'static str, // "lts", "stable", "latest"
    pub is_lts: bool,
    pub has_high_cve: bool,
    pub has_critical_cve: bool,
}

impl ComponentVersion {
    pub const fn new(name: &'static str, version: &'static str, channel: &'static str, is_lts: bool) -> Self {
        Self { name, version, channel, is_lts, has_high_cve: false, has_critical_cve: false }
    }

    pub fn is_safe(&self) -> bool {
        !self.has_high_cve && !self.has_critical_cve
    }
}

/// Version registry: only LTS and latest stable.
pub const VERSION_REGISTRY: &[ComponentVersion] = &[
    // Apache
    ComponentVersion::new("apache", "2.4.66", "stable", true),
    ComponentVersion::new("apache", "2.4.65", "stable", true),
    // Nginx
    ComponentVersion::new("nginx", "1.26.2", "stable", true),
    ComponentVersion::new("nginx", "1.24.0", "lts", true),
    // MariaDB
    ComponentVersion::new("mariadb", "11.4.13", "lts", true),
    ComponentVersion::new("mariadb", "11.2.5", "lts", true),
    ComponentVersion::new("mariadb", "10.11.11", "lts", true),
    // MySQL
    ComponentVersion::new("mysql", "8.4.5", "lts", true),
    ComponentVersion::new("mysql", "8.0.40", "stable", true),
    // PostgreSQL
    ComponentVersion::new("postgresql", "16.6", "lts", true),
    ComponentVersion::new("postgresql", "17.2", "stable", true),
    // PHP
    ComponentVersion::new("php", "8.5.10", "latest", false),
    ComponentVersion::new("php", "8.4.13", "lts", true),
    ComponentVersion::new("php", "8.3.15", "lts", true),
    // Redis
    ComponentVersion::new("redis", "7.4.2", "stable", true),
    ComponentVersion::new("redis", "7.2.7", "lts", true),
    // Valkey
    ComponentVersion::new("valkey", "7.2.7", "stable", true),
    // Deno
    ComponentVersion::new("deno", "2.1.4", "stable", false),
];

/// Get safe versions for a component.
pub fn get_safe_versions(component: &str) -> Vec<&'static ComponentVersion> {
    VERSION_REGISTRY.iter()
        .filter(|v| v.name == component && v.is_safe())
        .collect()
}

/// Get the recommended version (LTS preferred).
pub fn get_recommended_version(component: &str) -> Option<&'static ComponentVersion> {
    let safe = get_safe_versions(component);
    safe.iter().find(|v| v.is_lts).copied().or_else(|| safe.first().copied())
}

/// Check if a specific version is safe.
pub fn is_version_safe(component: &str, version: &str) -> bool {
    VERSION_REGISTRY.iter()
        .find(|v| v.name == component && v.version == version)
        .map(|v| v.is_safe())
        .unwrap_or(false) // Unknown version = unsafe
}
