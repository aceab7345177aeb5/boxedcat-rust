use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{ServiceManager, ServiceStatus};

pub struct PhpService {
    pub root: PathBuf,
}

impl PhpService {
    pub fn new(root: &PathBuf) -> Self {
        Self { root: root.clone() }
    }

    pub fn version_string(&self) -> String {
        self.exe_path()
            .and_then(|exe| {
                Command::new(&exe)
                    .arg("--version")
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|s| s.lines().next().map(String::from))
            })
            .unwrap_or_else(|| "Not found".into())
    }

    pub fn modules(&self) -> Vec<String> {
        self.exe_path()
            .and_then(|exe| {
                Command::new(&exe)
                    .arg("-m")
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| {
                        s.lines()
                            .filter(|l| !l.trim().is_empty() && !l.starts_with('['))
                            .map(|l| l.trim().to_string())
                            .collect()
                    })
            })
            .unwrap_or_default()
    }
}

impl ServiceManager for PhpService {
    fn name(&self) -> &str { "php" }
    fn display_name(&self) -> &str { "PHP" }
    fn service_type(&self) -> &str { "php-cgi" }
    fn default_port(&self) -> u16 { 9000 }

    fn exe_path(&self) -> Option<PathBuf> {
        let p = self.root.join("php").join("php.exe");
        if p.exists() { Some(p) } else { None }
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(self.root.join("php").join("php.ini"))
    }

    fn is_running(&self) -> bool {
        let task = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq php-cgi.exe", "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("php-cgi.exe")
            }
            Err(_) => false,
        }
    }

    fn status(&self) -> ServiceStatus {
        if !self.is_installed() {
            return ServiceStatus::Error("PHP not installed".into());
        }
        ServiceStatus::Stopped // PHP runs on-demand or via FastCGI
    }
}
