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
pub enum ServiceType {
    Vm,
    Database,
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceType::Vm => write!(f, "Virtual Machine"),
            ServiceType::Database => write!(f, "Managed Database"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseEngine {
    PostgreSql,
    MySql,
    SqlServer,
}

impl fmt::Display for DatabaseEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseEngine::PostgreSql => write!(f, "PostgreSQL"),
            DatabaseEngine::MySql => write!(f, "MySQL"),
            DatabaseEngine::SqlServer => write!(f, "SQL Server"),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkloadType {
    General,
    Ai,
    Sap,
    Hpc,
    Database,
    Batch,
    Storage,
}

impl fmt::Display for WorkloadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkloadType::General => write!(f, "General purpose"),
            WorkloadType::Ai => write!(f, "AI / ML"),
            WorkloadType::Sap => write!(f, "SAP"),
            WorkloadType::Hpc => write!(f, "HPC"),
            WorkloadType::Database => write!(f, "Database"),
            WorkloadType::Batch => write!(f, "Batch / CI-CD"),
            WorkloadType::Storage => write!(f, "Storage-intensive"),
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
    pub workload: WorkloadType,
    pub service: ServiceType,
    pub db_engine: Option<DatabaseEngine>,
    pub gpu_type: Option<String>,
    pub gpu_count: Option<u8>,
    pub spot: bool,
    pub explain: bool,
    pub min_network_gbps: Option<f32>,
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
    pub gpu: Option<GpuInfo>,
    pub network_gbps: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GpuInfo {
    pub model: &'static str,
    pub count: u8,
    pub vram_gb: u16,
}

impl InstanceType {
    /// Effective network bandwidth in Gbps. If the catalog value is 0 (unknown),
    /// estimate from vCPU count using the industry ~0.5 Gbps per vCPU heuristic,
    /// clamped to [0.5, 100].
    pub fn effective_network_gbps(&self) -> f32 {
        if self.network_gbps > 0.0 {
            self.network_gbps
        } else {
            (self.vcpus * 0.5).clamp(0.5, 100.0)
        }
    }
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
    #[allow(dead_code)]
    pub delta_vcpu: f32,
    #[allow(dead_code)]
    pub delta_ram: f32,
    pub breakdown: ScoreBreakdown,
}

/// Per-component scoring breakdown (for --explain mode)
#[derive(Debug, Clone, Copy)]
pub struct ScoreBreakdown {
    pub cpu_component: f32,
    pub ram_component: f32,
    pub cost_component: f32,
    pub network_component: f32,
    pub cpu_weight: f32,
    pub ram_weight: f32,
    pub cost_weight: f32,
    pub network_weight: f32,
}

/// Spot/Preemptible discount rates by provider (approximate averages vs on-demand)
pub struct SpotRates {
    pub discount: f32,
}

pub fn spot_rates(cloud: CloudProvider) -> SpotRates {
    match cloud {
        // GCP Spot VMs: ~60-91% off, use ~70% average
        CloudProvider::Gcp => SpotRates { discount: 0.70 },
        // AWS Spot: ~70-90% off, use ~70% average
        CloudProvider::Aws => SpotRates { discount: 0.70 },
        // Azure Spot: ~60-90% off, use ~65% average
        CloudProvider::Azure => SpotRates { discount: 0.65 },
    }
}

/// Committed use discount rates by provider
pub struct CommittedRates {
    pub one_year_discount: f32,  // e.g., 0.37 = 37% off
    pub three_year_discount: f32,
}

pub fn committed_rates(cloud: CloudProvider) -> CommittedRates {
    match cloud {
        CloudProvider::Gcp => CommittedRates {
            one_year_discount: 0.37,
            three_year_discount: 0.55,
        },
        CloudProvider::Aws => CommittedRates {
            one_year_discount: 0.36,
            three_year_discount: 0.56,
        },
        CloudProvider::Azure => CommittedRates {
            one_year_discount: 0.34,
            three_year_discount: 0.51,
        },
    }
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
