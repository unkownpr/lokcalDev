use crate::config::paths;
use crate::error::AppError;
use crate::services::utils;

pub struct NginxConfigGenerator;

impl NginxConfigGenerator {
    pub fn generate_site_config(
        domain: &str,
        document_root: &str,
        php_port: u16,
        ssl: bool,
        ssl_cert: Option<&str>,
        ssl_key: Option<&str>,
    ) -> String {
        let fastcgi_params_path = paths::get_nginx_config_dir().join("fastcgi_params");
        let log_dir = paths::get_logs_dir();

        // Ensure all paths use forward slashes for nginx compatibility
        let fastcgi_params = utils::to_forward_slash(&fastcgi_params_path);
        let log_dir_str = utils::to_forward_slash(&log_dir);
        let doc_root = document_root.replace('\\', "/");

        let mut config = String::new();

        if ssl {
            // HTTP -> HTTPS redirect
            config.push_str(&format!(
                r#"server {{
    listen 80;
    server_name {domain};
    return 301 https://$host$request_uri;
}}

"#
            ));
        }

        config.push_str(&format!(
            "server {{\n    listen {};\n    server_name {};\n",
            if ssl { "443 ssl" } else { "80" },
            domain,
        ));

        if ssl {
            if let (Some(cert), Some(key)) = (ssl_cert, ssl_key) {
                let cert_path = cert.replace('\\', "/");
                let key_path = key.replace('\\', "/");
                config.push_str(&format!(
                    "    ssl_certificate \"{}\";\n    ssl_certificate_key \"{}\";\n    ssl_protocols TLSv1.2 TLSv1.3;\n    ssl_ciphers HIGH:!aNULL:!MD5;\n",
                    cert_path, key_path,
                ));
            }
        }

        config.push_str(&format!(
            r#"    root "{}";
    index index.php index.html index.htm;

    access_log "{}/{}-access.log";
    error_log "{}/{}-error.log";

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location ~ \.php$ {{
        try_files $uri =404;
        fastcgi_pass 127.0.0.1:{};
        fastcgi_index index.php;
        include "{}";
    }}

    location ~ /\.ht {{
        deny all;
    }}
"#,
            doc_root,
            log_dir_str,
            domain,
            log_dir_str,
            domain,
            php_port,
            fastcgi_params,
        ));

        // Add phpMyAdmin location if installed
        // Use root (parent dir) instead of alias to avoid nginx's known issue
        // with alias + nested PHP location blocks
        let pma_dir = paths::get_phpmyadmin_dir();
        if pma_dir.join("index.php").exists() {
            let pma_root = utils::to_forward_slash(
                &pma_dir.parent().unwrap_or(&pma_dir).to_path_buf(),
            );
            config.push_str(&format!(
                r#"
    location ^~ /phpmyadmin {{
        root "{}";
        index index.php;

        location ~ \.php$ {{
            root "{}";
            fastcgi_pass 127.0.0.1:{};
            fastcgi_index index.php;
            include "{}";
        }}
    }}
"#,
                pma_root, pma_root, php_port, fastcgi_params,
            ));
        }

        config.push_str("}\n");

        config
    }

    pub fn write_site_config(domain: &str, config_content: &str) -> Result<(), AppError> {
        let config_dir = paths::get_nginx_config_dir().join("sites-enabled");
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join(format!("{}.conf", domain));
        std::fs::write(&config_path, config_content)?;
        log::info!("Wrote nginx config for {}", domain);
        Ok(())
    }

    pub fn remove_site_config(domain: &str) -> Result<(), AppError> {
        let config_path = paths::get_nginx_config_dir()
            .join("sites-enabled")
            .join(format!("{}.conf", domain));

        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
            log::info!("Removed nginx config for {}", domain);
        }
        Ok(())
    }
}
