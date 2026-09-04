use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::ServiceManager;

pub struct RedisService {
    pub root: PathBuf,
}

impl RedisService {
    pub fn new(root: &PathBuf) -> Self {
        Self { root: root.clone() }
    }
}

impl ServiceManager for RedisService {
    fn name(&self) -> &str { "redis" }
    fn display_name(&self) -> &str { "Redis" }
    fn service_type(&self) -> &str { "redis-server" }
    fn default_port(&self) -> u16 { 6379 }

    fn exe_path(&self) -> Option<PathBuf> {
        let p = self.root.join("redis").join("redis-server.exe");
        if p.exists() { Some(p) } else { None }
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(self.root.join("redis").join("redis.conf"))
    }

    fn start(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("Redis not installed".into());
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
            return Err("Redis not installed".into());
        }
        let cli = self.root.join("redis").join("redis-cli.exe");
        Command::new(&cli)
            .arg("shutdown")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn is_running(&self) -> bool {
        let task = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq redis-server.exe", "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("redis-server.exe")
            }
            Err(_) => false,
        }
    }
}

