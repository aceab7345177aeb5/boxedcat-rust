use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::{ServiceManager, ServiceStatus};

pub struct ApacheService {
    pub root: PathBuf,
}

impl ApacheService {
    pub fn new(root: &PathBuf) -> Self {
        Self { root: root.clone() }
    }
}

impl ServiceManager for ApacheService {
    fn name(&self) -> &str { "apache" }
    fn display_name(&self) -> &str { "Apache HTTP Server" }
    fn service_type(&self) -> &str { "httpd" }
    fn default_port(&self) -> u16 { 80 }

    fn exe_path(&self) -> Option<PathBuf> {
        let p = self.root.join("apache").join("bin").join("httpd.exe");
        if p.exists() { Some(p) } else { None }
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(self.root.join("apache").join("conf").join("httpd.conf"))
    }

    fn status(&self) -> ServiceStatus {
        if !self.is_installed() {
            return ServiceStatus::Error("Apache not installed".into());
        }
        match Command::new(self.exe_path().unwrap())
            .args(["-k", "start"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => {
                if status.success() { ServiceStatus::Running } else { ServiceStatus::Stopped }
            }
            Err(e) => ServiceStatus::Error(e.to_string()),
        }
    }

    fn start(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("Apache not installed".into());
        }
        let exe = self.exe_path().unwrap();
        Command::new(&exe)
            .args(["-k", "start"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn stop(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("Apache not installed".into());
        }
        let exe = self.exe_path().unwrap();
        Command::new(&exe)
            .args(["-k", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn is_running(&self) -> bool {
        let task = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq httpd.exe", "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("httpd.exe")
            }
            Err(_) => false,
        }
    }
}
