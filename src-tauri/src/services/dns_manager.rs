use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsEntry {
    pub domain: String,
    pub ip: String,
}

pub struct DnsManager;

impl DnsManager {
    fn get_hosts_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
        }
        #[cfg(not(target_os = "windows"))]
        {
            PathBuf::from("/etc/hosts")
        }
    }

    const MARKER_START: &'static str = "# LokcalDev START";
    const MARKER_END: &'static str = "# LokcalDev END";

    fn read_hosts() -> Result<String, AppError> {
        std::fs::read_to_string(Self::get_hosts_path())
            .map_err(|e| AppError::Io(e))
    }

    fn get_lokcal_entries(content: &str) -> Vec<DnsEntry> {
        let mut entries = Vec::new();
        let mut in_block = false;

        for line in content.lines() {
            if line.trim() == Self::MARKER_START {
                in_block = true;
                continue;
            }
            if line.trim() == Self::MARKER_END {
                in_block = false;
                continue;
            }
            if in_block {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    entries.push(DnsEntry {
                        ip: parts[0].to_string(),
                        domain: parts[1].to_string(),
                    });
                }
            }
        }

        entries
    }

    fn build_hosts_content(content: &str, entries: &[DnsEntry]) -> String {
        let mut lines: Vec<String> = Vec::new();
        let mut skip = false;
        for line in content.lines() {
            if line.trim() == Self::MARKER_START {
                skip = true;
                continue;
            }
            if line.trim() == Self::MARKER_END {
                skip = false;
                continue;
            }
            if !skip {
                lines.push(line.to_string());
            }
        }

        // Add our block at the end
        if !entries.is_empty() {
            lines.push(String::new());
            lines.push(Self::MARKER_START.to_string());
            for entry in entries {
                lines.push(format!("{} {}", entry.ip, entry.domain));
            }
            lines.push(Self::MARKER_END.to_string());
        }

        lines.join("\n") + "\n"
    }

    /// Write content to /etc/hosts using elevated privileges.
    /// On macOS uses osascript, on Windows uses the file directly (app should be run as admin).
    fn write_hosts(content: &str) -> Result<(), AppError> {
        let hosts_path = Self::get_hosts_path();

        #[cfg(target_os = "macos")]
        {
            // Write to a temp file first, then copy with admin privileges
            let temp_path = "/tmp/lokcaldev_hosts_tmp";
            std::fs::write(temp_path, content)?;

            let script = format!(
                "do shell script \"cp {} {}\" with administrator privileges",
                temp_path,
                hosts_path.display()
            );

            let output = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output()
                .map_err(|e| AppError::Process(format!("Failed to run osascript: {}", e)))?;

            // Clean up temp file
            let _ = std::fs::remove_file(temp_path);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Process(format!(
                    "Failed to update hosts file: {}",
                    stderr.trim()
                )));
            }

            // Flush DNS cache
            let _ = std::process::Command::new("dscacheutil")
                .arg("-flushcache")
                .output();
            let _ = std::process::Command::new("killall")
                .args(["-HUP", "mDNSResponder"])
                .output();

            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            std::fs::write(&hosts_path, content)?;
            // Flush DNS cache
            let _ = std::process::Command::new("ipconfig")
                .arg("/flushdns")
                .output();
            Ok(())
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            // Linux: try pkexec for graphical sudo
            let temp_path = "/tmp/lokcaldev_hosts_tmp";
            std::fs::write(temp_path, content)?;

            let output = std::process::Command::new("pkexec")
                .args(["cp", temp_path, &hosts_path.to_string_lossy()])
                .output()
                .map_err(|e| AppError::Process(format!("Failed to run pkexec: {}", e)))?;

            let _ = std::fs::remove_file(temp_path);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Process(format!(
                    "Failed to update hosts file: {}",
                    stderr.trim()
                )));
            }
            Ok(())
        }
    }

    pub fn add_entry(domain: &str, ip: &str) -> Result<(), AppError> {
        let content = Self::read_hosts()?;
        let mut entries = Self::get_lokcal_entries(&content);

        // Remove existing entry for this domain
        entries.retain(|e| e.domain != domain);

        entries.push(DnsEntry {
            domain: domain.to_string(),
            ip: ip.to_string(),
        });

        let new_content = Self::build_hosts_content(&content, &entries);
        Self::write_hosts(&new_content)?;

        log::info!("Added DNS entry: {} -> {}", domain, ip);
        Ok(())
    }

    pub fn remove_entry(domain: &str) -> Result<(), AppError> {
        let content = Self::read_hosts()?;
        let mut entries = Self::get_lokcal_entries(&content);

        let before = entries.len();
        entries.retain(|e| e.domain != domain);

        // Only write if something changed
        if entries.len() != before {
            let new_content = Self::build_hosts_content(&content, &entries);
            Self::write_hosts(&new_content)?;
            log::info!("Removed DNS entry for {}", domain);
        }

        Ok(())
    }

    pub fn list_entries() -> Result<Vec<DnsEntry>, AppError> {
        let content = Self::read_hosts()?;
        Ok(Self::get_lokcal_entries(&content))
    }

    pub fn setup_resolver(tld: &str) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        {
            let resolver_dir = PathBuf::from("/etc/resolver");
            let resolver_file = resolver_dir.join(tld);
            let content = "nameserver 127.0.0.1\n";

            // Write to temp, then copy with admin privileges
            let temp_path = "/tmp/lokcaldev_resolver_tmp";
            std::fs::write(temp_path, content)?;

            let script = format!(
                "do shell script \"mkdir -p /etc/resolver && cp {} {}\" with administrator privileges",
                temp_path,
                resolver_file.display()
            );

            let output = std::process::Command::new("osascript")
                .args(["-e", &script])
                .output()
                .map_err(|e| AppError::Process(format!("Failed to run osascript: {}", e)))?;

            let _ = std::fs::remove_file(temp_path);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Process(format!(
                    "Failed to set up resolver: {}",
                    stderr.trim()
                )));
            }

            log::info!("Set up resolver for .{}", tld);
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::info!("Resolver setup not supported on this platform, using hosts file only");
        }
        Ok(())
    }
}
