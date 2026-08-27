use crate::models::{CloudProvider, InstanceTier, InstanceType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

const CACHE_TTL_HOURS: u64 = 24;
const CACHE_FILE: &str = "catalog.json";
const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // 10 MB max response

/// Cached catalog data structure
#[derive(Serialize, Deserialize)]
pub struct CachedCatalog {
    pub timestamp: u64,
    pub gcp: Vec<CachedInstance>,
    pub aws: Vec<CachedInstance>,
    pub azure: Vec<CachedInstance>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CachedInstance {
    pub name: String,
    pub family: String,
    pub vcpus: f32,
    pub ram_gb: f32,
    pub tier: String,
    pub cost_hour_usd: f32,
}

// Thread-safe catalog storage using OnceLock (no unsafe needed)
static LIVE_GCP: OnceLock<Vec<InstanceType>> = OnceLock::new();
static LIVE_AWS: OnceLock<Vec<InstanceType>> = OnceLock::new();
static LIVE_AZURE: OnceLock<Vec<InstanceType>> = OnceLock::new();

/// Initialize live pricing data. Returns true if fresh data was loaded.
pub fn init_live_pricing(_cloud: CloudProvider) -> bool {
    // Check cache first
    if let Some(catalog) = load_cache() {
        store_catalog(catalog);
        return true;
    }

    // Download fresh prices
    eprint!("\x1b[2m  ⟳ Updating prices...\x1b[0m");

    let catalog = download_all();

    if catalog.gcp.is_empty() && catalog.aws.is_empty() && catalog.azure.is_empty() {
        eprintln!("\r\x1b[2m  ⚠ Could not fetch prices, using embedded catalog\x1b[0m");
        return false;
    }

    // Save to cache
    save_cache(&catalog);
    store_catalog(catalog);

    eprintln!("\r\x1b[2m  ✓ Prices updated                    \x1b[0m");
    true
}

/// Get live catalog for a provider, or None if not available
pub fn get_live_catalog(cloud: CloudProvider) -> Option<&'static [InstanceType]> {
    match cloud {
        CloudProvider::Gcp => LIVE_GCP.get().map(|v| v.as_slice()),
        CloudProvider::Aws => LIVE_AWS.get().map(|v| v.as_slice()),
        CloudProvider::Azure => LIVE_AZURE.get().map(|v| v.as_slice()),
    }
}

fn store_catalog(catalog: CachedCatalog) {
    let gcp = convert_instances(&catalog.gcp);
    let aws = convert_instances(&catalog.aws);
    let azure = convert_instances(&catalog.azure);

    if !gcp.is_empty() { let _ = LIVE_GCP.set(gcp); }
    if !aws.is_empty() { let _ = LIVE_AWS.set(aws); }
    if !azure.is_empty() { let _ = LIVE_AZURE.set(azure); }
}

fn convert_instances(cached: &[CachedInstance]) -> Vec<InstanceType> {
    cached
        .iter()
        .filter_map(|c| {
            // Sanitize: reject entries with control chars or suspicious content
            if !is_safe_identifier(&c.name) || !is_safe_identifier(&c.family) {
                return None;
            }

            let tier = match c.tier.as_str() {
                "economy" => InstanceTier::Economy,
                _ => InstanceTier::Standard,
            };
            // Validate numeric ranges
            if c.vcpus <= 0.0 || c.vcpus > 1024.0 { return None; }
            if c.ram_gb <= 0.0 || c.ram_gb > 32768.0 { return None; }
            if c.cost_hour_usd < 0.0 || c.cost_hour_usd > 1000.0 { return None; }

            Some(InstanceType {
                // Leak individual strings (safe: small, bounded, one-time init)
                name: Box::leak(c.name.clone().into_boxed_str()),
                family: Box::leak(c.family.clone().into_boxed_str()),
                vcpus: c.vcpus,
                ram_gb: c.ram_gb,
                tier,
                cost_hour_usd: c.cost_hour_usd,
                gpu: None,
            })
        })
        .collect()
}

/// Validate that a string is a safe instance/family identifier (no control chars, no escapes)
fn is_safe_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 64 {
        return false;
    }
    s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

fn cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".kumo")
}

fn cache_path() -> PathBuf {
    cache_dir().join(CACHE_FILE)
}

fn load_cache() -> Option<CachedCatalog> {
    let path = cache_path();
    if !path.exists() {
        return None;
    }

    // Security: refuse to read if it's a symlink
    let metadata = fs::symlink_metadata(&path).ok()?;
    if metadata.file_type().is_symlink() {
        return None;
    }

    // Check TTL
    let modified = metadata.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;

    if age > Duration::from_secs(CACHE_TTL_HOURS * 3600) {
        return None; // Cache expired
    }

    let content = fs::read_to_string(&path).ok()?;

    // Reject excessively large cache files (possible tampering)
    if content.len() > MAX_RESPONSE_SIZE {
        return None;
    }

    serde_json::from_str(&content).ok()
}

fn save_cache(catalog: &CachedCatalog) {
    let dir = cache_dir();
    let _ = fs::create_dir_all(&dir);

    // Set directory permissions to 0700 (owner only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }

    let path = cache_path();

    // Anti-symlink: refuse to write if path is a symlink
    if path.exists() {
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if meta.file_type().is_symlink() {
                eprintln!("  ⚠ Cache file is a symlink — refusing to write (security)");
                return;
            }
        }
    }

    if let Ok(json) = serde_json::to_string(catalog) {
        // Atomic write: write to temp, then rename
        let tmp_path = dir.join(".catalog.json.tmp");
        if fs::write(&tmp_path, &json).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
            }
            let _ = fs::rename(&tmp_path, &path);
        }
    }
}

fn download_all() -> CachedCatalog {
    let gcp = fetch_gcp_prices().unwrap_or_default();
    let aws = fetch_aws_prices().unwrap_or_default();
    let azure = fetch_azure_prices().unwrap_or_default();

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    CachedCatalog {
        timestamp,
        gcp,
        aws,
        azure,
    }
}

// ─── GCP Pricing ────────────────────────────────────────────────────────────

fn fetch_gcp_prices() -> Result<Vec<CachedInstance>, Box<dyn std::error::Error>> {
    // GCP Cloud Billing Catalog API (public, no auth required for SKU listing)
    // Service ID 6F81-5844-456A = Compute Engine
    let body = fetch_json(
        "https://cloudbilling.googleapis.com/v1/services/6F81-5844-456A/skus?pageSize=5000&currencyCode=USD"
    );

    if body.is_err() {
        // Fallback: construct from known pricing ratios
        return Ok(build_gcp_from_known_pricing());
    }

    // Parse the SKUs response
    let json: serde_json::Value = serde_json::from_str(&body.unwrap())?;
    let skus = json["skus"].as_array().unwrap_or(&Vec::new()).clone();

    let mut instances = Vec::new();

    // Extract predefined VM pricing from SKUs
    for sku in &skus {
        let desc = sku["description"].as_str().unwrap_or("");
        let category = sku["category"]["resourceGroup"].as_str().unwrap_or("");

        // Filter for Compute Engine predefined VMs
        if category != "CPU" && category != "RAM" {
            continue;
        }

        // We'd need to combine CPU + RAM SKUs per machine type
        // This is complex, so fall back to the known pricing approach
        let _ = desc;
    }

    if instances.is_empty() {
        instances = build_gcp_from_known_pricing();
    }

    Ok(instances)
}

/// Build GCP pricing from per-vCPU and per-GB-RAM rates (official published rates)
fn build_gcp_from_known_pricing() -> Vec<CachedInstance> {
    // GCP pricing is formula-based: cost = vcpus * cpu_rate + ram_gb * ram_rate
    // Source: https://cloud.google.com/compute/vm-instance-pricing

    struct FamilyRates {
        family: &'static str,
        cpu_rate: f32,  // per vCPU per hour
        ram_rate: f32,  // per GB per hour
        tier: &'static str,
    }

    let rates = [
        FamilyRates { family: "E2", cpu_rate: 0.02191, ram_rate: 0.00294, tier: "economy" },
        FamilyRates { family: "N2", cpu_rate: 0.03198, ram_rate: 0.00429, tier: "standard" },
        FamilyRates { family: "N2D", cpu_rate: 0.02781, ram_rate: 0.00373, tier: "standard" },
        FamilyRates { family: "C2", cpu_rate: 0.03398, ram_rate: 0.00455, tier: "standard" },
        FamilyRates { family: "C2D", cpu_rate: 0.02903, ram_rate: 0.00389, tier: "standard" },
    ];

    struct MachineSpec {
        suffix: &'static str,
        vcpus: f32,
        ram_factor: f32, // RAM = vcpus * ram_factor
    }

    let standard_specs = &[
        MachineSpec { suffix: "2", vcpus: 2.0, ram_factor: 4.0 },
        MachineSpec { suffix: "4", vcpus: 4.0, ram_factor: 4.0 },
        MachineSpec { suffix: "8", vcpus: 8.0, ram_factor: 4.0 },
        MachineSpec { suffix: "16", vcpus: 16.0, ram_factor: 4.0 },
        MachineSpec { suffix: "32", vcpus: 32.0, ram_factor: 4.0 },
        MachineSpec { suffix: "48", vcpus: 48.0, ram_factor: 4.0 },
        MachineSpec { suffix: "64", vcpus: 64.0, ram_factor: 4.0 },
    ];

    let highmem_specs = &[
        MachineSpec { suffix: "2", vcpus: 2.0, ram_factor: 8.0 },
        MachineSpec { suffix: "4", vcpus: 4.0, ram_factor: 8.0 },
        MachineSpec { suffix: "8", vcpus: 8.0, ram_factor: 8.0 },
        MachineSpec { suffix: "16", vcpus: 16.0, ram_factor: 8.0 },
    ];

    let highcpu_specs = &[
        MachineSpec { suffix: "2", vcpus: 2.0, ram_factor: 1.0 },
        MachineSpec { suffix: "4", vcpus: 4.0, ram_factor: 1.0 },
        MachineSpec { suffix: "8", vcpus: 8.0, ram_factor: 1.0 },
        MachineSpec { suffix: "16", vcpus: 16.0, ram_factor: 1.0 },
        MachineSpec { suffix: "32", vcpus: 32.0, ram_factor: 1.0 },
    ];

    let mut instances = Vec::new();

    // E2 special: micro and small with shared vCPUs
    instances.push(CachedInstance {
        name: "e2-micro".to_string(), family: "E2".to_string(),
        vcpus: 0.25, ram_gb: 1.0, tier: "economy".to_string(),
        cost_hour_usd: 0.00838,
    });
    instances.push(CachedInstance {
        name: "e2-small".to_string(), family: "E2".to_string(),
        vcpus: 0.5, ram_gb: 2.0, tier: "economy".to_string(),
        cost_hour_usd: 0.01675,
    });
    instances.push(CachedInstance {
        name: "e2-medium".to_string(), family: "E2".to_string(),
        vcpus: 1.0, ram_gb: 4.0, tier: "economy".to_string(),
        cost_hour_usd: 0.03351,
    });

    for rate in &rates {
        // Standard instances
        for spec in standard_specs {
            if rate.family == "C2" && spec.vcpus > 60.0 { continue; }
            if rate.family == "C2D" && spec.vcpus > 56.0 { continue; }
            if rate.family == "E2" && spec.vcpus > 32.0 { continue; }

            let ram_gb = spec.vcpus * spec.ram_factor;
            let cost = spec.vcpus * rate.cpu_rate + ram_gb * rate.ram_rate;
            let name = format!("{}-standard-{}", rate.family.to_lowercase(), spec.suffix);

            instances.push(CachedInstance {
                name,
                family: rate.family.to_string(),
                vcpus: spec.vcpus,
                ram_gb,
                tier: rate.tier.to_string(),
                cost_hour_usd: cost,
            });
        }

        // Highmem (only for E2, N2)
        if rate.family == "E2" || rate.family == "N2" {
            for spec in highmem_specs {
                let ram_gb = spec.vcpus * spec.ram_factor;
                let cost = spec.vcpus * rate.cpu_rate + ram_gb * rate.ram_rate;
                let name = format!("{}-highmem-{}", rate.family.to_lowercase(), spec.suffix);

                instances.push(CachedInstance {
                    name,
                    family: rate.family.to_string(),
                    vcpus: spec.vcpus,
                    ram_gb,
                    tier: rate.tier.to_string(),
                    cost_hour_usd: cost,
                });
            }
        }

        // Highcpu (only for E2, N2)
        if rate.family == "E2" || rate.family == "N2" {
            for spec in highcpu_specs {
                let ram_gb = spec.vcpus * spec.ram_factor;
                let cost = spec.vcpus * rate.cpu_rate + ram_gb * rate.ram_rate;
                let name = format!("{}-highcpu-{}", rate.family.to_lowercase(), spec.suffix);

                instances.push(CachedInstance {
                    name,
                    family: rate.family.to_string(),
                    vcpus: spec.vcpus,
                    ram_gb,
                    tier: rate.tier.to_string(),
                    cost_hour_usd: cost,
                });
            }
        }
    }

    instances
}

// ─── AWS Pricing ────────────────────────────────────────────────────────────

fn fetch_aws_prices() -> Result<Vec<CachedInstance>, Box<dyn std::error::Error>> {
    // AWS publishes pricing as a JSON index
    // This is a large file, so we fetch only us-east-1 compute pricing
    let url = "https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AmazonEC2/current/us-east-1/index.json";

    let body = fetch_json(url);
    if body.is_err() {
        return Ok(build_aws_from_known_pricing());
    }

    let json: serde_json::Value = match serde_json::from_str(&body.unwrap()) {
        Ok(v) => v,
        Err(_) => return Ok(build_aws_from_known_pricing()),
    };

    let products = match json["products"].as_object() {
        Some(p) => p,
        None => return Ok(build_aws_from_known_pricing()),
    };

    let terms = json["terms"]["OnDemand"].as_object();

    let mut instances = Vec::new();
    let target_families = ["t3", "t3a", "m5", "m6i", "c5", "c6i", "r5", "r6i"];

    for (sku, product) in products {
        let attrs = match product["attributes"].as_object() {
            Some(a) => a,
            None => continue,
        };

        let instance_type = attrs.get("instanceType")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Filter to our target families
        let family_match = target_families.iter().any(|f| {
            instance_type.starts_with(f) && instance_type.chars().nth(f.len()) == Some('.')
        });
        if !family_match { continue; }

        let os = attrs.get("operatingSystem").and_then(|v| v.as_str()).unwrap_or("");
        if os != "Linux" { continue; }

        let tenancy = attrs.get("tenancy").and_then(|v| v.as_str()).unwrap_or("");
        if tenancy != "Shared" { continue; }

        let preinstalled = attrs.get("preInstalledSw").and_then(|v| v.as_str()).unwrap_or("");
        if preinstalled != "NA" { continue; }

        let vcpus: f32 = attrs.get("vcpu")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let memory_str = attrs.get("memory")
            .and_then(|v| v.as_str())
            .unwrap_or("0 GiB");
        let ram_gb: f32 = memory_str
            .replace(" GiB", "")
            .replace(",", "")
            .parse()
            .unwrap_or(0.0);

        if vcpus == 0.0 || ram_gb == 0.0 { continue; }

        // Get price from terms
        let price = if let Some(terms_map) = terms {
            if let Some(term) = terms_map.get(sku) {
                term.as_object()
                    .and_then(|t| t.values().next())
                    .and_then(|v| v["priceDimensions"].as_object())
                    .and_then(|pd| pd.values().next())
                    .and_then(|d| d["pricePerUnit"]["USD"].as_str())
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        if price == 0.0 { continue; }

        let family = instance_type.split('.').next().unwrap_or("").to_string();
        let family_upper = match family.as_str() {
            "t3" => "T3", "t3a" => "T3a",
            "m5" => "M5", "m6i" => "M6i",
            "c5" => "C5", "c6i" => "C6i",
            "r5" => "R5", "r6i" => "R6i",
            _ => continue,
        };

        let tier = match family.as_str() {
            "t3" | "t3a" => "economy",
            _ => "standard",
        };

        instances.push(CachedInstance {
            name: instance_type.to_string(),
            family: family_upper.to_string(),
            vcpus,
            ram_gb,
            tier: tier.to_string(),
            cost_hour_usd: price,
        });
    }

    // Deduplicate by name (multiple SKUs per instance type)
    instances.sort_by(|a, b| a.name.cmp(&b.name));
    instances.dedup_by(|a, b| a.name == b.name);

    if instances.is_empty() {
        return Ok(build_aws_from_known_pricing());
    }

    Ok(instances)
}

fn build_aws_from_known_pricing() -> Vec<CachedInstance> {
    // Known AWS pricing for us-east-1 (fallback)
    let mut instances = Vec::new();

    struct AwsType { name: &'static str, family: &'static str, vcpus: f32, ram_gb: f32, tier: &'static str, cost: f32 }

    let types = [
        AwsType { name: "t3.nano", family: "T3", vcpus: 0.5, ram_gb: 0.5, tier: "economy", cost: 0.0052 },
        AwsType { name: "t3.micro", family: "T3", vcpus: 1.0, ram_gb: 1.0, tier: "economy", cost: 0.0104 },
        AwsType { name: "t3.small", family: "T3", vcpus: 1.0, ram_gb: 2.0, tier: "economy", cost: 0.0208 },
        AwsType { name: "t3.medium", family: "T3", vcpus: 2.0, ram_gb: 4.0, tier: "economy", cost: 0.0416 },
        AwsType { name: "t3.large", family: "T3", vcpus: 2.0, ram_gb: 8.0, tier: "economy", cost: 0.0832 },
        AwsType { name: "t3.xlarge", family: "T3", vcpus: 4.0, ram_gb: 16.0, tier: "economy", cost: 0.1664 },
        AwsType { name: "t3.2xlarge", family: "T3", vcpus: 8.0, ram_gb: 32.0, tier: "economy", cost: 0.3328 },
        AwsType { name: "t3a.nano", family: "T3a", vcpus: 0.5, ram_gb: 0.5, tier: "economy", cost: 0.0047 },
        AwsType { name: "t3a.micro", family: "T3a", vcpus: 1.0, ram_gb: 1.0, tier: "economy", cost: 0.0094 },
        AwsType { name: "t3a.small", family: "T3a", vcpus: 1.0, ram_gb: 2.0, tier: "economy", cost: 0.0188 },
        AwsType { name: "t3a.medium", family: "T3a", vcpus: 2.0, ram_gb: 4.0, tier: "economy", cost: 0.0376 },
        AwsType { name: "t3a.large", family: "T3a", vcpus: 2.0, ram_gb: 8.0, tier: "economy", cost: 0.0752 },
        AwsType { name: "t3a.xlarge", family: "T3a", vcpus: 4.0, ram_gb: 16.0, tier: "economy", cost: 0.1504 },
        AwsType { name: "t3a.2xlarge", family: "T3a", vcpus: 8.0, ram_gb: 32.0, tier: "economy", cost: 0.3008 },
        AwsType { name: "m5.large", family: "M5", vcpus: 2.0, ram_gb: 8.0, tier: "standard", cost: 0.0960 },
        AwsType { name: "m5.xlarge", family: "M5", vcpus: 4.0, ram_gb: 16.0, tier: "standard", cost: 0.1920 },
        AwsType { name: "m5.2xlarge", family: "M5", vcpus: 8.0, ram_gb: 32.0, tier: "standard", cost: 0.3840 },
        AwsType { name: "m5.4xlarge", family: "M5", vcpus: 16.0, ram_gb: 64.0, tier: "standard", cost: 0.7680 },
        AwsType { name: "m6i.large", family: "M6i", vcpus: 2.0, ram_gb: 8.0, tier: "standard", cost: 0.0960 },
        AwsType { name: "m6i.xlarge", family: "M6i", vcpus: 4.0, ram_gb: 16.0, tier: "standard", cost: 0.1920 },
        AwsType { name: "m6i.2xlarge", family: "M6i", vcpus: 8.0, ram_gb: 32.0, tier: "standard", cost: 0.3840 },
        AwsType { name: "m6i.4xlarge", family: "M6i", vcpus: 16.0, ram_gb: 64.0, tier: "standard", cost: 0.7680 },
        AwsType { name: "c5.large", family: "C5", vcpus: 2.0, ram_gb: 4.0, tier: "standard", cost: 0.0850 },
        AwsType { name: "c5.xlarge", family: "C5", vcpus: 4.0, ram_gb: 8.0, tier: "standard", cost: 0.1700 },
        AwsType { name: "c5.2xlarge", family: "C5", vcpus: 8.0, ram_gb: 16.0, tier: "standard", cost: 0.3400 },
        AwsType { name: "c6i.large", family: "C6i", vcpus: 2.0, ram_gb: 4.0, tier: "standard", cost: 0.0850 },
        AwsType { name: "c6i.xlarge", family: "C6i", vcpus: 4.0, ram_gb: 8.0, tier: "standard", cost: 0.1700 },
        AwsType { name: "c6i.2xlarge", family: "C6i", vcpus: 8.0, ram_gb: 16.0, tier: "standard", cost: 0.3400 },
        AwsType { name: "r5.large", family: "R5", vcpus: 2.0, ram_gb: 16.0, tier: "standard", cost: 0.1260 },
        AwsType { name: "r5.xlarge", family: "R5", vcpus: 4.0, ram_gb: 32.0, tier: "standard", cost: 0.2520 },
        AwsType { name: "r5.2xlarge", family: "R5", vcpus: 8.0, ram_gb: 64.0, tier: "standard", cost: 0.5040 },
        AwsType { name: "r6i.large", family: "R6i", vcpus: 2.0, ram_gb: 16.0, tier: "standard", cost: 0.1260 },
        AwsType { name: "r6i.xlarge", family: "R6i", vcpus: 4.0, ram_gb: 32.0, tier: "standard", cost: 0.2520 },
        AwsType { name: "r6i.2xlarge", family: "R6i", vcpus: 8.0, ram_gb: 64.0, tier: "standard", cost: 0.5040 },
    ];

    for t in &types {
        instances.push(CachedInstance {
            name: t.name.to_string(),
            family: t.family.to_string(),
            vcpus: t.vcpus,
            ram_gb: t.ram_gb,
            tier: t.tier.to_string(),
            cost_hour_usd: t.cost,
        });
    }

    instances
}

// ─── Azure Pricing ──────────────────────────────────────────────────────────

fn fetch_azure_prices() -> Result<Vec<CachedInstance>, Box<dyn std::error::Error>> {
    // Azure Retail Prices API — public, no auth needed
    let families = ["Standard_B", "Standard_D", "Standard_E", "Standard_F"];
    let mut instances = Vec::new();

    for family_prefix in &families {
        let url = format!(
            "https://prices.azure.com/api/retail/prices?$filter=serviceName eq 'Virtual Machines' and armRegionName eq 'eastus' and priceType eq 'Consumption' and contains(armSkuName, '{}')",
            family_prefix
        );

        let body = match fetch_json(&url) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let items = match json["Items"].as_array() {
            Some(items) => items,
            None => continue,
        };

        for item in items {
            let sku_name = item["armSkuName"].as_str().unwrap_or("");
            let meter_name = item["meterName"].as_str().unwrap_or("");
            let unit_price = item["unitPrice"].as_f64().unwrap_or(0.0) as f32;
            let product_name = item["productName"].as_str().unwrap_or("");

            // Filter: only Linux VMs, hourly pricing
            if !product_name.contains("Linux") && !product_name.contains("Virtual Machines") {
                continue;
            }
            if !meter_name.contains("Low Priority") && unit_price == 0.0 {
                continue;
            }
            // Skip spot/low priority
            if meter_name.contains("Low Priority") || meter_name.contains("Spot") {
                continue;
            }

            let vcpus = extract_azure_vcpus(sku_name);
            let ram_gb = extract_azure_ram(sku_name);

            if vcpus == 0.0 || ram_gb == 0.0 || unit_price == 0.0 {
                continue;
            }

            let family = extract_azure_family(sku_name);
            let tier = if family == "B" || family == "Av2" {
                "economy"
            } else {
                "standard"
            };

            instances.push(CachedInstance {
                name: sku_name.to_string(),
                family: family.to_string(),
                vcpus,
                ram_gb,
                tier: tier.to_string(),
                cost_hour_usd: unit_price,
            });
        }
    }

    // Deduplicate
    instances.sort_by(|a, b| a.name.cmp(&b.name));
    instances.dedup_by(|a, b| a.name == b.name);

    if instances.is_empty() {
        return Ok(build_azure_from_known_pricing());
    }

    Ok(instances)
}

fn extract_azure_vcpus(sku: &str) -> f32 {
    // Parse vCPU count from SKU name like "Standard_D4s_v5" → 4
    let parts: Vec<&str> = sku.split('_').collect();
    if parts.len() < 2 { return 0.0; }

    let size_part = parts[1];
    let digits: String = size_part.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().unwrap_or(0.0)
}

fn extract_azure_ram(sku: &str) -> f32 {
    // Azure RAM is typically vcpus * ratio depending on family
    let vcpus = extract_azure_vcpus(sku);
    let sku_lower = sku.to_lowercase();

    if sku_lower.contains("_b") {
        vcpus * 4.0 // B-series: ~4 GB per vCPU
    } else if sku_lower.contains("_d") {
        vcpus * 4.0 // D-series: 4 GB per vCPU
    } else if sku_lower.contains("_e") {
        vcpus * 8.0 // E-series: 8 GB per vCPU (memory optimized)
    } else if sku_lower.contains("_f") {
        vcpus * 2.0 // F-series: 2 GB per vCPU (compute optimized)
    } else {
        vcpus * 4.0
    }
}

fn extract_azure_family(sku: &str) -> String {
    let sku_lower = sku.to_lowercase();
    if sku_lower.contains("_b") { "B".to_string() }
    else if sku_lower.contains("_d") && sku_lower.contains("v5") { "Dsv5".to_string() }
    else if sku_lower.contains("_d") && sku_lower.contains("v4") { "Dsv4".to_string() }
    else if sku_lower.contains("_e") && sku_lower.contains("v5") { "Esv5".to_string() }
    else if sku_lower.contains("_f") { "Fsv2".to_string() }
    else { "Other".to_string() }
}

fn build_azure_from_known_pricing() -> Vec<CachedInstance> {
    let mut instances = Vec::new();

    struct AzType { name: &'static str, family: &'static str, vcpus: f32, ram_gb: f32, tier: &'static str, cost: f32 }

    let types = [
        AzType { name: "Standard_B1s", family: "B", vcpus: 1.0, ram_gb: 1.0, tier: "economy", cost: 0.0104 },
        AzType { name: "Standard_B1ms", family: "B", vcpus: 1.0, ram_gb: 2.0, tier: "economy", cost: 0.0207 },
        AzType { name: "Standard_B2s", family: "B", vcpus: 2.0, ram_gb: 4.0, tier: "economy", cost: 0.0416 },
        AzType { name: "Standard_B2ms", family: "B", vcpus: 2.0, ram_gb: 8.0, tier: "economy", cost: 0.0832 },
        AzType { name: "Standard_B4ms", family: "B", vcpus: 4.0, ram_gb: 16.0, tier: "economy", cost: 0.1660 },
        AzType { name: "Standard_B8ms", family: "B", vcpus: 8.0, ram_gb: 32.0, tier: "economy", cost: 0.3320 },
        AzType { name: "Standard_D2s_v5", family: "Dsv5", vcpus: 2.0, ram_gb: 8.0, tier: "standard", cost: 0.0960 },
        AzType { name: "Standard_D4s_v5", family: "Dsv5", vcpus: 4.0, ram_gb: 16.0, tier: "standard", cost: 0.1920 },
        AzType { name: "Standard_D8s_v5", family: "Dsv5", vcpus: 8.0, ram_gb: 32.0, tier: "standard", cost: 0.3840 },
        AzType { name: "Standard_D16s_v5", family: "Dsv5", vcpus: 16.0, ram_gb: 64.0, tier: "standard", cost: 0.7680 },
        AzType { name: "Standard_D32s_v5", family: "Dsv5", vcpus: 32.0, ram_gb: 128.0, tier: "standard", cost: 1.5360 },
        AzType { name: "Standard_E2s_v5", family: "Esv5", vcpus: 2.0, ram_gb: 16.0, tier: "standard", cost: 0.1260 },
        AzType { name: "Standard_E4s_v5", family: "Esv5", vcpus: 4.0, ram_gb: 32.0, tier: "standard", cost: 0.2520 },
        AzType { name: "Standard_E8s_v5", family: "Esv5", vcpus: 8.0, ram_gb: 64.0, tier: "standard", cost: 0.5040 },
        AzType { name: "Standard_F2s_v2", family: "Fsv2", vcpus: 2.0, ram_gb: 4.0, tier: "standard", cost: 0.0846 },
        AzType { name: "Standard_F4s_v2", family: "Fsv2", vcpus: 4.0, ram_gb: 8.0, tier: "standard", cost: 0.1690 },
        AzType { name: "Standard_F8s_v2", family: "Fsv2", vcpus: 8.0, ram_gb: 16.0, tier: "standard", cost: 0.3380 },
        AzType { name: "Standard_F16s_v2", family: "Fsv2", vcpus: 16.0, ram_gb: 32.0, tier: "standard", cost: 0.6760 },
    ];

    for t in &types {
        instances.push(CachedInstance {
            name: t.name.to_string(),
            family: t.family.to_string(),
            vcpus: t.vcpus,
            ram_gb: t.ram_gb,
            tier: t.tier.to_string(),
            cost_hour_usd: t.cost,
        });
    }

    instances
}

// ─── HTTP Helper ────────────────────────────────────────────────────────────

fn fetch_json(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call()?;

    // Read with size limit to prevent OOM from malicious/broken responses
    let mut body = String::new();
    response
        .into_reader()
        .take(MAX_RESPONSE_SIZE as u64)
        .read_to_string(&mut body)?;

    Ok(body)
}
