use crate::error::AppError;
use crate::services::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsEntry {
    pub domain: String,
    pub ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverStatus {
    pub resolver_exists: bool,
    pub dnsmasq_installed: bool,
    pub dnsmasq_running: bool,
    /// Fully configured: dnsmasq installed + running + resolver file exists
    pub configured: bool,
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

    fn write_hosts(content: &str) -> Result<(), AppError> {
        let hosts_path = Self::get_hosts_path();

        #[cfg(target_os = "macos")]
        {
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

            let _ = std::fs::remove_file(temp_path);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Process(format!(
                    "Failed to update hosts file: {}",
                    stderr.trim()
                )));
            }

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
            // Try direct write first (works if app is running as admin)
            if std::fs::write(&hosts_path, content).is_ok() {
                let _ = std::process::Command::new("ipconfig")
                    .arg("/flushdns")
                    .output();
                return Ok(());
            }

            // Fallback: write via PowerShell with UAC elevation
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join("lokcaldev_hosts_tmp");
            std::fs::write(&temp_path, content)?;

            let ps_cmd = format!(
                "Copy-Item '{}' '{}' -Force",
                temp_path.display(),
                hosts_path.display()
            );
            let output = std::process::Command::new("powershell")
                .args([
                    "-Command",
                    &format!(
                        "Start-Process powershell -ArgumentList '-Command','{}' -Verb RunAs -Wait",
                        ps_cmd.replace('\'', "''")
                    ),
                ])
                .output()
                .map_err(|e| AppError::Process(format!("Failed to elevate: {}", e)))?;

            let _ = std::fs::remove_file(&temp_path);

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Process(format!(
                    "Failed to update hosts file: {}",
                    stderr.trim()
                )));
            }

            let _ = std::process::Command::new("ipconfig")
                .arg("/flushdns")
                .output();
            Ok(())
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
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

    // ── dnsmasq + resolver (port 5353, NO root needed for dnsmasq) ──

    #[cfg(target_os = "macos")]
    fn is_dnsmasq_installed() -> bool {
        utils::get_brew_prefix().join("sbin").join("dnsmasq").exists()
    }

    #[cfg(target_os = "macos")]
    fn is_dnsmasq_running() -> bool {
        // Check if dnsmasq process is running via pgrep
        std::process::Command::new("pgrep")
            .arg("dnsmasq")
            .output()
            .ok()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "macos")]
    fn resolver_file_correct(tld: &str) -> bool {
        let path = PathBuf::from(format!("/etc/resolver/{}", tld));
        if !path.exists() {
            return false;
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        content.contains("nameserver 127.0.0.1") && content.contains("port 5353")
    }

    pub fn get_resolver_status(tld: &str) -> ResolverStatus {
        #[cfg(target_os = "macos")]
        {
            let dnsmasq_installed = Self::is_dnsmasq_installed();
            let dnsmasq_running = if dnsmasq_installed {
                Self::is_dnsmasq_running()
            } else {
                false
            };
            let resolver_exists = Self::resolver_file_correct(tld);
            ResolverStatus {
                resolver_exists,
                dnsmasq_installed,
                dnsmasq_running,
                configured: dnsmasq_installed && dnsmasq_running && resolver_exists,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = tld;
            ResolverStatus {
                resolver_exists: false,
                dnsmasq_installed: false,
                dnsmasq_running: false,
                configured: false,
            }
        }
    }

    /// Start dnsmasq as user service (NO password needed).
    /// Call this on every app startup to ensure dnsmasq is running.
    pub fn ensure_dnsmasq_running(tld: &str) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        {
            if !Self::is_dnsmasq_installed() {
                return Ok(()); // Not installed yet, skip silently
            }

            // Ensure config has port=5353 and address entry
            let brew_prefix = utils::get_brew_prefix();
            let dnsmasq_conf = brew_prefix.join("etc").join("dnsmasq.conf");
            let address_entry = format!("address=/.{}/127.0.0.1", tld);
            let port_entry = "port=5353";

            let content = if dnsmasq_conf.exists() {
                std::fs::read_to_string(&dnsmasq_conf).unwrap_or_default()
            } else {
                String::new()
            };

            // Check line-by-line for uncommented active entries
            let has_address = content.lines().any(|l| {
                let t = l.trim();
                !t.starts_with('#') && t.contains(&address_entry)
            });
            let has_port = content.lines().any(|l| {
                let t = l.trim();
                !t.starts_with('#') && t == port_entry
            });

            let needs_address = !has_address;
            let needs_port = !has_port;

            if needs_address || needs_port {
                let mut new_content = content.clone();
                if !new_content.is_empty() && !new_content.ends_with('\n') {
                    new_content.push('\n');
                }
                if needs_port {
                    new_content.push_str("# LokcalDev: listen on non-privileged port\n");
                    new_content.push_str("port=5353\n");
                }
                if needs_address {
                    new_content.push_str(&format!("# LokcalDev: resolve all .{} to localhost\n", tld));
                    new_content.push_str(&address_entry);
                    new_content.push('\n');
                }
                std::fs::write(&dnsmasq_conf, new_content)?;
                log::info!("Updated dnsmasq config (port 5353, .{})", tld);
            }

            // Start dnsmasq as user service (NO sudo needed on port 5353)
            if !Self::is_dnsmasq_running() {
                // Stop any old system-level service first (ignore errors)
                let _ = std::process::Command::new("brew")
                    .args(["services", "stop", "dnsmasq"])
                    .output();

                let output = std::process::Command::new("brew")
                    .args(["services", "start", "dnsmasq"])
                    .output()
                    .map_err(|e| AppError::Process(format!("Failed to start dnsmasq: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("dnsmasq start warning: {}", stderr.trim());
                }

                // Wait a moment for dnsmasq to bind
                std::thread::sleep(std::time::Duration::from_millis(500));
                log::info!("Started dnsmasq on port 5353");
            }

            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = tld;
            Ok(())
        }
    }

    /// Full resolver setup: install dnsmasq + configure + create resolver file.
    /// The resolver file (/etc/resolver/{tld}) needs ONE password prompt.
    /// After that, dnsmasq runs on port 5353 (no root) and auto-starts.
    pub fn setup_resolver(tld: &str) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        {
            utils::ensure_homebrew()?;

            // 1. Install dnsmasq if not present (no sudo)
            if !Self::is_dnsmasq_installed() {
                log::info!("Installing dnsmasq via Homebrew...");
                let output = std::process::Command::new("brew")
                    .args(["install", "dnsmasq"])
                    .output()
                    .map_err(|e| AppError::Process(format!("Failed to install dnsmasq: {}", e)))?;

                if !Self::is_dnsmasq_installed() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(AppError::Process(format!(
                        "dnsmasq installation failed: {}",
                        stderr.chars().take(500).collect::<String>()
                    )));
                }
                log::info!("dnsmasq installed successfully");
            }

            // 2. Configure + start dnsmasq on port 5353 (no sudo)
            Self::ensure_dnsmasq_running(tld)?;

            // 3. Create/update resolver file (needs sudo ONE TIME)
            if !Self::resolver_file_correct(tld) {
                let resolver_file = format!("/etc/resolver/{}", tld);
                let resolver_content = "nameserver 127.0.0.1\\nport 5353";
                let script = format!(
                    "do shell script \"mkdir -p /etc/resolver && printf '{}' > {}\" with administrator privileges",
                    resolver_content, resolver_file
                );

                let output = std::process::Command::new("osascript")
                    .args(["-e", &script])
                    .output()
                    .map_err(|e| AppError::Process(format!("Failed to run osascript: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(AppError::Process(format!(
                        "Failed to create resolver file: {}",
                        stderr.trim()
                    )));
                }
                log::info!("Created /etc/resolver/{} with port 5353", tld);
            }

            // 4. Flush DNS cache
            let _ = std::process::Command::new("dscacheutil")
                .arg("-flushcache")
                .output();
            let _ = std::process::Command::new("killall")
                .args(["-HUP", "mDNSResponder"])
                .output();

            log::info!("DNS resolver fully configured for .{}", tld);
            Ok(())
        }

        #[cfg(not(target_os = "macos"))]
        {
            log::info!("DNS resolver setup not supported on this platform, use hosts file");
            let _ = tld;
            Ok(())
        }
    }
}
