use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::ServiceManager;

pub struct PostgresqlService {
    pub root: PathBuf,
}

impl PostgresqlService {
    pub fn new(root: &PathBuf) -> Self {
        Self { root: root.clone() }
    }
}

impl ServiceManager for PostgresqlService {
    fn name(&self) -> &str { "postgresql" }
    fn display_name(&self) -> &str { "PostgreSQL" }
    fn service_type(&self) -> &str { "pg_ctl" }
    fn default_port(&self) -> u16 { 5432 }

    fn exe_path(&self) -> Option<PathBuf> {
        let p = self.root.join("postgresql").join("bin").join("pg_ctl.exe");
        if p.exists() { Some(p) } else { None }
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(self.root.join("postgresql").join("data").join("postgresql.conf"))
    }

    fn start(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("PostgreSQL not installed".into());
        }
        let exe = self.exe_path().unwrap();
        let data_dir = self.root.join("postgresql").join("data");
        Command::new(&exe)
            .args(["start", "-D", &data_dir.to_string_lossy()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn stop(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err("PostgreSQL not installed".into());
        }
        let exe = self.exe_path().unwrap();
        let data_dir = self.root.join("postgresql").join("data");
        Command::new(&exe)
            .args(["stop", "-D", &data_dir.to_string_lossy()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn is_running(&self) -> bool {
        let task = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq postgres.exe", "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("postgres.exe")
            }
            Err(_) => false,
        }
    }
}

