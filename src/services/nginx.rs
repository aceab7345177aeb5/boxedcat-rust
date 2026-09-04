use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::ServiceManager;

pub struct NginxService {
    pub root: PathBuf,
}

impl NginxService {
    pub fn new(root: &PathBuf) -> Self {
        Self { root: root.clone() }
    }
}

impl ServiceManager for NginxService {
    fn name(&self) -> &str { "nginx" }
    fn display_name(&self) -> &str { "Nginx" }
    fn service_type(&self) -> &str { "nginx" }
    fn default_port(&self) -> u16 { 80 }

    fn exe_path(&self) -> Option<PathBuf> {
        let p = self.root.join("nginx").join("nginx.exe");
        if p.exists() { Some(p) } else { None }
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(self.root.join("nginx").join("conf").join("nginx.conf"))
    }

    fn start(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("Nginx not installed".into());
        }
        let exe = self.exe_path().unwrap();
        Command::new(&exe)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn stop(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("Nginx not installed".into());
        }
        let exe = self.exe_path().unwrap();
        Command::new(&exe)
            .args(["-s", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn is_running(&self) -> bool {
        let task = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq nginx.exe", "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("nginx.exe")
            }
            Err(_) => false,
        }
    }
}

