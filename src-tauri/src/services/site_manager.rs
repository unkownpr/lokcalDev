use crate::config::paths;
use crate::error::AppError;
use crate::services::dns_manager::DnsManager;
use crate::services::nginx_config::NginxConfigGenerator;
use crate::services::nginx_manager::NginxManager;
use crate::services::utils;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub document_root: String,
    pub php_version: String,
    pub ssl: bool,
    pub active: bool,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSiteRequest {
    pub name: String,
    pub domain: String,
    pub document_root: String,
    pub php_version: String,
    pub ssl: bool,
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSiteRequest {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub document_root: Option<String>,
    pub php_version: Option<String>,
    pub ssl: Option<bool>,
    pub active: Option<bool>,
}

pub struct SiteManager;

impl SiteManager {
    fn get_site_file(id: &str) -> std::path::PathBuf {
        paths::get_sites_dir().join(format!("{}.toml", id))
    }

    pub fn list() -> Result<Vec<Site>, AppError> {
        let sites_dir = paths::get_sites_dir();
        std::fs::create_dir_all(&sites_dir)?;

        let mut sites = Vec::new();
        for entry in std::fs::read_dir(&sites_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(site) = toml::from_str::<Site>(&content) {
                    sites.push(site);
                }
            }
        }
        sites.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(sites)
    }

    pub fn get(id: &str) -> Result<Site, AppError> {
        let path = Self::get_site_file(id);
        if !path.exists() {
            return Err(AppError::NotFound(format!("Site '{}' not found", id)));
        }
        let content = std::fs::read_to_string(&path)?;
        toml::from_str(&content).map_err(|e| AppError::Config(e.to_string()))
    }

    pub fn create(req: CreateSiteRequest) -> Result<Site, AppError> {
        // Ensure sites directory exists
        let sites_dir = paths::get_sites_dir();
        std::fs::create_dir_all(&sites_dir)?;

        let has_template = req.template.is_some();
        let id = Uuid::new_v4().to_string();
        let site = Site {
            id: id.clone(),
            name: req.name,
            domain: req.domain.clone(),
            document_root: req.document_root.clone(),
            php_version: req.php_version.clone(),
            ssl: req.ssl,
            active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            template_status: if has_template { Some("pending".to_string()) } else { None },
            template: req.template,
        };

        // Save site config
        let toml_str = toml::to_string_pretty(&site)
            .map_err(|e| AppError::Config(e.to_string()))?;
        std::fs::write(Self::get_site_file(&id), &toml_str)?;

        // Create document root if it doesn't exist
        let doc_root = std::path::Path::new(&req.document_root);
        if !doc_root.exists() {
            std::fs::create_dir_all(doc_root)?;
        }

        // Place default index.php only for blank sites (no template)
        if !has_template {
            let index_php = doc_root.join("index.php");
            let index_html = doc_root.join("index.html");
            if !index_php.exists() && !index_html.exists() {
                let default_page = include_str!("../../resources/index.php");
                std::fs::write(&index_php, default_page)?;
            }
        }

        // For Laravel, nginx root should point to {document_root}/public
        let nginx_root = if site.template.as_deref() == Some("laravel") {
            let public_dir = doc_root.join("public");
            std::fs::create_dir_all(&public_dir)?;
            public_dir.to_string_lossy().to_string()
        } else {
            req.document_root.clone()
        };

        // Generate nginx config
        let php_port = utils::php_version_to_port(&req.php_version);
        let ssl_dir = paths::get_ssl_dir();
        let (ssl_cert, ssl_key) = if req.ssl {
            (
                Some(ssl_dir.join(format!("{}.pem", req.domain)).to_string_lossy().to_string()),
                Some(ssl_dir.join(format!("{}-key.pem", req.domain)).to_string_lossy().to_string()),
            )
        } else {
            (None, None)
        };

        let nginx_info = NginxManager::get_info();
        let nginx_config = NginxConfigGenerator::generate_site_config(
            &req.domain,
            &nginx_root,
            php_port,
            req.ssl,
            ssl_cert.as_deref(),
            ssl_key.as_deref(),
            nginx_info.port,
            nginx_info.ssl_port,
        );
        NginxConfigGenerator::write_site_config(&req.domain, &nginx_config)?;

        // Add DNS entry for the domain (hosts file fallback when dnsmasq not configured)
        Self::ensure_dns_entry(&req.domain);

        log::info!("Created site: {} ({})", site.name, site.domain);
        Ok(site)
    }

    pub fn update(id: &str, req: UpdateSiteRequest) -> Result<Site, AppError> {
        let mut site = Self::get(id)?;
        let old_domain = site.domain.clone();

        if let Some(name) = req.name {
            site.name = name;
        }
        if let Some(domain) = req.domain {
            site.domain = domain;
        }
        if let Some(document_root) = req.document_root {
            site.document_root = document_root;
        }
        if let Some(php_version) = req.php_version {
            site.php_version = php_version;
        }
        if let Some(ssl) = req.ssl {
            site.ssl = ssl;
        }
        if let Some(active) = req.active {
            site.active = active;
        }

        // Save updated site
        let toml_str = toml::to_string_pretty(&site)
            .map_err(|e| AppError::Config(e.to_string()))?;
        std::fs::write(Self::get_site_file(id), &toml_str)?;

        // Remove old nginx config if domain changed
        if old_domain != site.domain {
            NginxConfigGenerator::remove_site_config(&old_domain)?;
        }

        // If site is inactive, remove nginx config
        if !site.active {
            NginxConfigGenerator::remove_site_config(&site.domain)?;
        } else {
            let php_port = utils::php_version_to_port(&site.php_version);
            let ssl_dir = paths::get_ssl_dir();
            let (ssl_cert, ssl_key) = if site.ssl {
                (
                    Some(ssl_dir.join(format!("{}.pem", site.domain)).to_string_lossy().to_string()),
                    Some(ssl_dir.join(format!("{}-key.pem", site.domain)).to_string_lossy().to_string()),
                )
            } else {
                (None, None)
            };

            let nginx_info = NginxManager::get_info();
            let nginx_config = NginxConfigGenerator::generate_site_config(
                &site.domain,
                &site.document_root,
                php_port,
                site.ssl,
                ssl_cert.as_deref(),
                ssl_key.as_deref(),
                nginx_info.port,
                nginx_info.ssl_port,
            );
            NginxConfigGenerator::write_site_config(&site.domain, &nginx_config)?;
        }

        log::info!("Updated site: {} ({})", site.name, site.domain);
        Ok(site)
    }

    pub fn update_template_status(id: &str, status: &str) -> Result<(), AppError> {
        let mut site = Self::get(id)?;
        site.template_status = Some(status.to_string());
        let toml_str = toml::to_string_pretty(&site)
            .map_err(|e| AppError::Config(e.to_string()))?;
        std::fs::write(Self::get_site_file(id), &toml_str)?;
        log::info!("Updated template status for site {}: {}", id, status);
        Ok(())
    }

    /// Regenerate nginx configs for ALL active sites.
    /// Called on nginx start to ensure configs match current settings (ports, etc.).
    pub fn regenerate_all_configs() -> Result<(), AppError> {
        let sites = Self::list()?;
        let nginx_info = NginxManager::get_info();
        let ssl_dir = paths::get_ssl_dir();

        for site in &sites {
            if !site.active {
                continue;
            }

            let php_port = utils::php_version_to_port(&site.php_version);

            // For Laravel, nginx root should point to {document_root}/public
            let nginx_root = if site.template.as_deref() == Some("laravel") {
                let public_dir = std::path::Path::new(&site.document_root).join("public");
                public_dir.to_string_lossy().to_string()
            } else {
                site.document_root.clone()
            };

            let (ssl_cert, ssl_key) = if site.ssl {
                (
                    Some(ssl_dir.join(format!("{}.pem", site.domain)).to_string_lossy().to_string()),
                    Some(ssl_dir.join(format!("{}-key.pem", site.domain)).to_string_lossy().to_string()),
                )
            } else {
                (None, None)
            };

            let config = NginxConfigGenerator::generate_site_config(
                &site.domain,
                &nginx_root,
                php_port,
                site.ssl,
                ssl_cert.as_deref(),
                ssl_key.as_deref(),
                nginx_info.port,
                nginx_info.ssl_port,
            );
            let _ = NginxConfigGenerator::write_site_config(&site.domain, &config);
        }

        log::info!("Regenerated nginx configs for all active sites");
        Ok(())
    }

    pub fn delete(id: &str) -> Result<(), AppError> {
        let site = Self::get(id)?;

        // Remove nginx config
        NginxConfigGenerator::remove_site_config(&site.domain)?;

        // Remove SSL certificate files if they exist
        let ssl_dir = paths::get_ssl_dir();
        let cert = ssl_dir.join(format!("{}.pem", site.domain));
        let key = ssl_dir.join(format!("{}-key.pem", site.domain));
        if cert.exists() {
            let _ = std::fs::remove_file(&cert);
        }
        if key.exists() {
            let _ = std::fs::remove_file(&key);
        }

        // Remove DNS entry
        Self::remove_dns_entry(&site.domain);

        // Remove site file
        let path = Self::get_site_file(id);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        log::info!("Deleted site: {} ({})", site.name, site.domain);
        Ok(())
    }

    /// Add DNS entry for a domain.
    /// On macOS: if dnsmasq resolver is configured, reload dnsmasq (wildcard handles all .tld).
    /// Otherwise add to hosts file as fallback.
    fn ensure_dns_entry(domain: &str) {
        #[cfg(target_os = "macos")]
        {
            let tld = domain.rsplit('.').next().unwrap_or("");
            let status = DnsManager::get_resolver_status(tld);
            if status.configured {
                // dnsmasq wildcard already handles all *.tld — reload to pick up any config changes
                DnsManager::reload_dnsmasq();
                return;
            }
        }

        // Fallback: add to hosts file (may need admin on macOS/Windows)
        if let Err(e) = DnsManager::add_entry(domain, "127.0.0.1") {
            log::warn!("Failed to add DNS entry for {}: {}", domain, e);
        }
    }

    /// Remove DNS entry for a domain.
    fn remove_dns_entry(domain: &str) {
        #[cfg(target_os = "macos")]
        {
            let tld = domain.rsplit('.').next().unwrap_or("");
            let status = DnsManager::get_resolver_status(tld);
            if status.configured {
                // dnsmasq wildcard handles removal automatically — reload to confirm
                DnsManager::reload_dnsmasq();
                return;
            }
        }

        if let Err(e) = DnsManager::remove_entry(domain) {
            log::warn!("Failed to remove DNS entry for {}: {}", domain, e);
        }
    }
}
