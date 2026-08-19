use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloudProvider {
    Gcp,
    Aws,
    Azure,
}

impl fmt::Display for CloudProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloudProvider::Gcp => write!(f, "GCP"),
            CloudProvider::Aws => write!(f, "AWS"),
            CloudProvider::Azure => write!(f, "Azure"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Environment {
    Production,
    PreProduction,
    Test,
    Dev,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Production => write!(f, "Production"),
            Environment::PreProduction => write!(f, "Pre-Production"),
            Environment::Test => write!(f, "Test"),
            Environment::Dev => write!(f, "Dev"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRequest {
    pub cloud: CloudProvider,
    pub vcpus: f32,
    pub ram_gb: f32,
    pub env: Environment,
    pub region: String,
    pub family: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceTier {
    Standard,
    Economy,
}

#[derive(Debug, Clone)]
pub struct InstanceType {
    pub name: &'static str,
    pub family: &'static str,
    pub vcpus: f32,
    pub ram_gb: f32,
    pub tier: InstanceTier,
    pub cost_hour_usd: f32,
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub exact_match: Option<&'static InstanceType>,
    pub closest_matches: Vec<ScoredInstance>,
    pub custom_option: Option<CustomMachine>,
    pub notes: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ScoredInstance {
    pub instance: &'static InstanceType,
    pub score: f32,
    pub delta_vcpu: f32,
    pub delta_ram: f32,
}

#[derive(Debug, Clone)]
pub struct CustomMachine {
    pub name: String,
    pub vcpus: f32,
    pub ram_gb: f32,
    pub estimated_cost_hour: f32,
}

/// Catalog pricing metadata
#[allow(dead_code)]
pub struct CatalogMeta {
    pub last_updated: &'static str,
    pub source: &'static str,
}

pub const CATALOG_META: CatalogMeta = CatalogMeta {
    last_updated: "live (cached 24h)",
    source: "Official cloud provider pricing APIs",
};
