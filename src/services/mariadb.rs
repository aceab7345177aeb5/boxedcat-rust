use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::ServiceManager;

pub struct MariaDbService {
    pub root: PathBuf,
}

impl MariaDbService {
    pub fn new(root: &PathBuf) -> Self {
        Self { root: root.clone() }
    }
}

impl ServiceManager for MariaDbService {
    fn name(&self) -> &str { "mariadb" }
    fn display_name(&self) -> &str { "MariaDB" }
    fn service_type(&self) -> &str { "mysqld" }
    fn default_port(&self) -> u16 { 3306 }

    fn exe_path(&self) -> Option<PathBuf> {
        let p = self.root.join("mysql").join("bin").join("mysqld.exe");
        if p.exists() { Some(p) } else { None }
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(self.root.join("mysql").join("my.ini"))
    }

    fn start(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("MariaDB not installed".into());
        }
        let exe = self.exe_path().unwrap();
        Command::new(&exe)
            .args(["--defaults-file", &self.root.join("mysql").join("my.ini").to_string_lossy()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn stop(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("MariaDB not installed".into());
        }
        let mysql = self.root.join("mysql").join("bin").join("mysql.exe");
        Command::new(&mysql)
            .args(["-u", "root", "--password=", "-e", "SHUTDOWN;"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn is_running(&self) -> bool {
        let task = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq mysqld.exe", "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("mysqld.exe")
            }
            Err(_) => false,
        }
    }
}

