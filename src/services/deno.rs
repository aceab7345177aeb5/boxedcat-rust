use std::path::PathBuf;

use super::ServiceManager;

pub struct DenoService {
    pub root: PathBuf,
}

impl DenoService {
    pub fn new(root: &PathBuf) -> Self {
        Self { root: root.clone() }
    }
}

impl ServiceManager for DenoService {
    fn name(&self) -> &str { "deno" }
    fn display_name(&self) -> &str { "Deno" }
    fn service_type(&self) -> &str { "deno" }
    fn default_port(&self) -> u16 { 8000 }

    fn exe_path(&self) -> Option<PathBuf> {
        let p = self.root.join("deno").join("deno.exe");
        if p.exists() { Some(p) } else { None }
    }

    fn config_path(&self) -> Option<PathBuf> {
        None
    }

    fn is_running(&self) -> bool {
        let task = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq deno.exe", "/NH"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output();
        match task {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("deno.exe")
            }
            Err(_) => false,
        }
    }
}
