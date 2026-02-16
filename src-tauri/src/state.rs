use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error,
    Starting,
    Stopping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub status: ServiceStatus,
    pub port: Option<u16>,
    pub version: Option<String>,
    pub pid: Option<u32>,
    pub installed: bool,
    pub initialized: bool,
}

pub struct AppState {
    pub services: Mutex<HashMap<String, ServiceInfo>>,
    pub child_processes: Mutex<HashMap<String, Child>>,
    pub log_tail_cancel: Mutex<Option<CancellationToken>>,
}

impl AppState {
    pub fn new(_data_dir: String) -> Self {
        let mut services = HashMap::new();

        // Only register real services that have actual managers
        let default_services = vec![
            ("nginx", "Nginx", Some(8080u16)),
            ("mariadb", "MariaDB", Some(3306u16)),
            ("phpmyadmin", "phpMyAdmin", None),
        ];

        for (id, name, port) in default_services {
            services.insert(
                id.to_string(),
                ServiceInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    status: ServiceStatus::Stopped,
                    port,
                    version: None,
                    pid: None,
                    installed: false,
                    initialized: false,
                },
            );
        }

        Self {
            services: Mutex::new(services),
            child_processes: Mutex::new(HashMap::new()),
            log_tail_cancel: Mutex::new(None),
        }
    }
}
