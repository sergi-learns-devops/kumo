use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Default, Debug)]
pub struct KumoConfig {
    #[serde(default)]
    pub defaults: ConfigDefaults,
}

#[derive(Deserialize, Default, Debug)]
pub struct ConfigDefaults {
    pub cloud: Option<String>,
    pub region: Option<String>,
    pub env: Option<String>,
    pub workload: Option<String>,
    pub service: Option<String>,
    pub engine: Option<String>,
}

impl KumoConfig {
    /// Load config from ~/.kumo/config.toml. Returns default if file doesn't exist.
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            return Self::default();
        }

        // Security: refuse symlinks
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if meta.file_type().is_symlink() {
                return Self::default();
            }
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        // Limit file size (avoid reading huge files)
        if content.len() > 10240 {
            return Self::default();
        }

        toml::from_str(&content).unwrap_or_default()
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".kumo").join("config.toml")
}

/// Create a default config file if it doesn't exist
#[allow(dead_code)]
pub fn init_config() {
    let path = config_path();
    if path.exists() {
        return;
    }

    let dir = path.parent().unwrap();
    let _ = fs::create_dir_all(dir);

    let default_content = r#"# kumo configuration file
# Uncomment and set your defaults:

[defaults]
# cloud = "gcp"           # gcp, aws, azure
# region = "us-central1"  # your default region
# env = "dev"             # production, pre-production, test, dev
# workload = "general"    # general, ai, sap, hpc, database, batch, storage
# service = "vm"          # vm, db
# engine = "postgres"     # postgres, mysql, sqlserver (for service = db)
"#;

    let _ = fs::write(&path, default_content);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
}
