use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "kumo", version, about = "☁ Cloud instance type recommender (GCP, AWS & Azure)")]
pub struct CliArgs {
    /// Cloud provider: gcp, aws, azure, or all (compare all providers)
    #[arg(long = "cloud", short = 'c')]
    pub cloud: Option<String>,

    /// Number of vCPUs required (supports decimals, e.g., 0.5)
    #[arg(long = "vcpu", short = 'v')]
    pub vcpu: Option<f32>,

    /// RAM in GB required
    #[arg(long = "ram", short = 'r')]
    pub ram: Option<f32>,

    /// Environment: production, pre-production, test, dev
    #[arg(long = "env", short = 'e')]
    pub env: Option<String>,

    /// Cloud region (e.g., us-central1, eu-west-1, eastus)
    #[arg(long = "region")]
    pub region: Option<String>,

    /// Filter by instance family (e.g., N2, M5, Dsv5)
    #[arg(long = "family", short = 'f')]
    pub family: Option<String>,

    /// Workload type: general, ai, sap, hpc, database, batch, storage
    #[arg(long = "workload", short = 'w')]
    pub workload: Option<String>,

    /// Service type: vm or db (default: vm)
    #[arg(long = "service")]
    pub service: Option<String>,

    /// Database engine (for --service db): postgres, mysql, sqlserver
    #[arg(long = "engine")]
    pub engine: Option<String>,

    /// List valid regions for a provider (usage: --list-regions gcp)
    #[arg(long = "list-regions")]
    pub list_regions: Option<String>,

    /// Download updated pricing catalog (requires network)
    #[arg(long = "update-catalog")]
    pub update_catalog: bool,

    /// Generate shell completions (bash, zsh, fish)
    #[arg(long = "completions")]
    pub completions: Option<String>,
}

impl CliArgs {
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.cloud.is_some()
            && self.vcpu.is_some()
            && self.ram.is_some()
            && self.env.is_some()
            && self.region.is_some()
    }
}
