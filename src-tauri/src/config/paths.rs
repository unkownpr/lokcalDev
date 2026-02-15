use std::path::PathBuf;

pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Library")
            .join("Application Support")
            .join("LokcalDev")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
            .join("LokcalDev")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("lokcaldev")
    }
}

pub fn get_config_dir() -> PathBuf {
    get_data_dir().join("config")
}

pub fn get_logs_dir() -> PathBuf {
    get_data_dir().join("logs")
}

pub fn get_ssl_dir() -> PathBuf {
    get_data_dir().join("ssl")
}

pub fn get_binaries_dir() -> PathBuf {
    get_data_dir().join("binaries")
}

#[allow(dead_code)]
pub fn get_php_dir() -> PathBuf {
    get_binaries_dir().join("php")
}

#[allow(dead_code)]
pub fn get_nginx_dir() -> PathBuf {
    get_binaries_dir().join("nginx")
}

#[allow(dead_code)]
pub fn get_mariadb_dir() -> PathBuf {
    get_binaries_dir().join("mariadb")
}

pub fn get_mkcert_dir() -> PathBuf {
    get_binaries_dir().join("mkcert")
}

pub fn get_sites_dir() -> PathBuf {
    get_data_dir().join("sites")
}

pub fn get_nginx_config_dir() -> PathBuf {
    get_config_dir().join("nginx")
}

pub fn get_phpmyadmin_dir() -> PathBuf {
    get_data_dir().join("phpmyadmin")
}
