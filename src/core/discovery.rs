use std::path::{Path, PathBuf};
use std::process::Command;

/// Information about an installed component.
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub display_name: String,
    pub path: PathBuf,
    pub version: String,
    pub exe_name: String,
    pub is_installed: bool,
}

impl ComponentInfo {
    pub fn exe_path(&self) -> Option<PathBuf> {
        if self.exe_name.is_empty() {
            return None;
        }
        let p = self.path.join(&self.exe_name);
        if p.exists() { Some(p) } else { None }
    }
}

/// Full installation discovery result.
#[derive(Debug, Clone)]
pub struct Installation {
    pub root: PathBuf,
    pub www: PathBuf,
    pub apache: Option<ComponentInfo>,
    pub nginx: Option<ComponentInfo>,
    pub mariadb: Option<ComponentInfo>,
    pub mysql: Option<ComponentInfo>,
    pub postgresql: Option<ComponentInfo>,
    pub php: Option<ComponentInfo>,
    pub redis: Option<ComponentInfo>,
    pub valkey: Option<ComponentInfo>,
    pub deno: Option<ComponentInfo>,
    pub composer: Option<ComponentInfo>,
}

impl Installation {
    pub fn components(&self) -> Vec<&ComponentInfo> {
        let mut list = Vec::new();
        if let Some(ref c) = self.apache { list.push(c); }
        if let Some(ref c) = self.nginx { list.push(c); }
        if let Some(ref c) = self.mariadb { list.push(c); }
        if let Some(ref c) = self.mysql { list.push(c); }
        if let Some(ref c) = self.postgresql { list.push(c); }
        if let Some(ref c) = self.php { list.push(c); }
        if let Some(ref c) = self.redis { list.push(c); }
        if let Some(ref c) = self.valkey { list.push(c); }
        if let Some(ref c) = self.deno { list.push(c); }
        if let Some(ref c) = self.composer { list.push(c); }
        list
    }
}

/// Detect version from an executable's --version output.
/// Extracts just the version number (e.g. "11.4.13" from "mysqld.exe  Ver 11.4.13-MariaDB...").
fn detect_version(exe_path: &Path, _keywords: &[&str]) -> String {
    if !exe_path.exists() {
        return "not found".to_string();
    }
    let args: Vec<&str> = if _keywords.contains(&"deno") {
        vec!["--version"]
    } else {
        vec!["--version"]
    };
    Command::new(exe_path)
        .args(&args)
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);

            // For Deno, output is multi-line: "deno 2.1.4\n..."
            if _keywords.contains(&"deno") {
                if let Some(line) = combined.lines().next() {
                    if let Some(ver) = line.split_whitespace().last() {
                        if ver.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                            return ver.to_string();
                        }
                    }
                }
            }

            // Try to extract version-like pattern: X.Y.Z or X.Y.Z.Z
            for line in combined.lines() {
                let words: Vec<&str> = line.split_whitespace().collect();
                for word in words {
                    let trimmed = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
                    let parts: Vec<&str> = trimmed.split('.').collect();
                    if parts.len() >= 2 && parts.iter().all(|p| !p.is_empty() && p.parse::<u32>().is_ok()) {
                        if trimmed.len() <= 20 {
                            return trimmed.to_string();
                        }
                    }
                }
            }
            // Fallback: first 60 chars of first line
            combined.lines().next().unwrap_or("unknown").trim().chars().take(60).collect()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Discover a BoxedCat installation at the given root path.
pub fn discover_installation(root: &Path) -> Installation {
    let www = root.join("www");

    let apache = {
        let path = root.join("apache");
        if path.exists() {
            let exe = path.join("bin").join("httpd.exe");
            Some(ComponentInfo {
                name: "apache".into(),
                display_name: "Apache HTTP Server".into(),
                version: detect_version(&exe, &["apache"]),
                exe_name: "bin/httpd.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let nginx = {
        let path = root.join("nginx");
        if path.exists() {
            let exe = path.join("nginx.exe");
            Some(ComponentInfo {
                name: "nginx".into(),
                display_name: "Nginx".into(),
                version: detect_version(&exe, &["nginx"]),
                exe_name: "nginx.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let mariadb = {
        let path = root.join("mysql"); // BoxedCat stores MariaDB under mysql/
        if path.exists() {
            let exe = path.join("bin").join("mysqld.exe");
            Some(ComponentInfo {
                name: "mariadb".into(),
                display_name: "MariaDB".into(),
                version: detect_version(&exe, &["mariadb", "mysql"]),
                exe_name: "bin/mysqld.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let mysql = {
        let path = root.join("mysql");
        if path.exists() {
            let exe = path.join("bin").join("mysqld.exe");
            Some(ComponentInfo {
                name: "mysql".into(),
                display_name: "MySQL".into(),
                version: detect_version(&exe, &["mysql"]),
                exe_name: "bin/mysqld.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let postgresql = {
        let path = root.join("postgresql");
        if path.exists() {
            let exe = path.join("bin").join("pg_ctl.exe");
            Some(ComponentInfo {
                name: "postgresql".into(),
                display_name: "PostgreSQL".into(),
                version: detect_version(&exe, &["postgres"]),
                exe_name: "bin/pg_ctl.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let php = {
        let path = root.join("php");
        if path.exists() {
            let exe = path.join("php.exe");
            Some(ComponentInfo {
                name: "php".into(),
                display_name: "PHP".into(),
                version: detect_version(&exe, &["php"]),
                exe_name: "php.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let redis = {
        let path = root.join("redis");
        if path.exists() {
            let exe = path.join("redis-server.exe");
            Some(ComponentInfo {
                name: "redis".into(),
                display_name: "Redis".into(),
                version: detect_version(&exe, &["redis"]),
                exe_name: "redis-server.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let valkey = {
        let path = root.join("valkey");
        if path.exists() {
            let exe = path.join("valkey-server.exe");
            Some(ComponentInfo {
                name: "valkey".into(),
                display_name: "Valkey".into(),
                version: detect_version(&exe, &["valkey"]),
                exe_name: "valkey-server.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let deno = {
        let path = root.join("deno");
        if path.exists() {
            let exe = path.join("deno.exe");
            Some(ComponentInfo {
                name: "deno".into(),
                display_name: "Deno".into(),
                version: detect_version(&exe, &["deno"]),
                exe_name: "deno.exe".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    let composer = {
        let path = root.join("composer");
        if path.exists() {
            let exe = path.join("composer.bat");
            Some(ComponentInfo {
                name: "composer".into(),
                display_name: "Composer".into(),
                version: detect_version(&exe, &["composer"]),
                exe_name: "composer.bat".into(),
                path,
                is_installed: true,
            })
        } else { None }
    };

    Installation { root: root.to_path_buf(), www, apache, nginx, mariadb, mysql, postgresql, php, redis, valkey, deno, composer }
}

/// Search common paths for a BoxedCat installation.
pub fn auto_discover() -> Option<Installation> {
    // First check exe directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("apache").exists() {
                return Some(discover_installation(dir));
            }
        }
    }
    // Fallback to common paths
    let candidates = vec![
        PathBuf::from("C:/xampp"),
        PathBuf::from("D:/xampp"),
    ];
    for path in &candidates {
        if path.exists() && path.join("apache").exists() {
            return Some(discover_installation(path));
        }
    }
    None
}
