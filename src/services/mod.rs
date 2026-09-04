pub mod apache;
pub mod deno;
pub mod mariadb;
pub mod nginx;
pub mod php;
pub mod postgresql;
pub mod redis;

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Service operational status.
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error(String),
    Unknown,
}

impl ServiceStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Running => "Running",
            Self::Stopped => "Stopped",
            Self::Error(_) => "Error",
            Self::Unknown => "Unknown",
        }
    }
}

/// Unified interface for all services.
pub trait ServiceManager: Send + Sync {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn service_type(&self) -> &str;
    fn default_port(&self) -> u16;

    fn exe_path(&self) -> Option<PathBuf>;
    fn config_path(&self) -> Option<PathBuf>;

    fn status(&self) -> ServiceStatus {
        match self.exe_path() {
            Some(exe) if exe.exists() => {
                if self.is_running() { ServiceStatus::Running } else { ServiceStatus::Stopped }
            }
            Some(_) => ServiceStatus::Error("Executable not found".into()),
            None => ServiceStatus::Unknown,
        }
    }

    fn is_installed(&self) -> bool {
        self.exe_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn is_running(&self) -> bool {
        let name = self.service_type().to_lowercase();
        let task = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}.exe", name), "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&name.to_lowercase())
            }
            Err(_) => false,
        }
    }

    fn start(&self) -> Result<(), String> {
        Err("Not implemented".into())
    }

    fn stop(&self) -> Result<(), String> {
        Err("Not implemented".into())
    }

    fn restart(&self) -> Result<(), String> {
        self.stop().ok();
        self.start()
    }
}

/// Service info for display.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub service_type: String,
    pub status: ServiceStatus,
    pub port: u16,
    pub version: String,
    pub is_installed: bool,
    pub is_running: bool,
}

impl ServiceInfo {
    pub fn from_manager(mgr: &dyn ServiceManager) -> Self {
        let status = mgr.status();
        Self {
            name: mgr.name().to_string(),
            display_name: mgr.display_name().to_string(),
            service_type: mgr.service_type().to_string(),
            port: mgr.default_port(),
            version: "detected".into(),
            is_installed: mgr.is_installed(),
            is_running: mgr.is_running(),
            status,
        }
    }
}
