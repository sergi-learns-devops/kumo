mod cli;
mod config;
mod db_catalog;
mod interactive;
mod models;
mod output;
mod pricing;
mod providers;
mod recommend;
mod workloads;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use cli::CliArgs;
use config::KumoConfig;
use models::{CloudProvider, DatabaseEngine, Environment, ServiceType, UserRequest, WorkloadType};
use providers::aws::AWS_REGIONS;
use providers::azure::AZURE_REGIONS;
use providers::gcp::GCP_REGIONS;
use recommend::suggest_region;

fn parse_cloud(s: &str) -> Option<CloudProvider> {
    match s.to_lowercase().as_str() {
        "gcp" | "google" => Some(CloudProvider::Gcp),
        "aws" | "amazon" => Some(CloudProvider::Aws),
        "azure" | "az" | "microsoft" => Some(CloudProvider::Azure),
        _ => None,
    }
}

fn parse_env(s: &str) -> Option<Environment> {
    match s.to_lowercase().as_str() {
        "production" | "prod" => Some(Environment::Production),
        "pre-production" | "preprod" | "pre" | "staging" => Some(Environment::PreProduction),
        "test" | "testing" => Some(Environment::Test),
        "dev" | "development" => Some(Environment::Dev),
        _ => None,
    }
}

fn parse_workload(s: &str) -> Option<WorkloadType> {
    match s.to_lowercase().as_str() {
        "general" | "gp" | "standard" => Some(WorkloadType::General),
        "ai" | "ml" | "gpu" => Some(WorkloadType::Ai),
        "sap" | "hana" => Some(WorkloadType::Sap),
        "hpc" | "compute" => Some(WorkloadType::Hpc),
        "database" | "db" | "memory" => Some(WorkloadType::Database),
        "batch" | "ci" | "cicd" | "ci-cd" => Some(WorkloadType::Batch),
        "storage" | "io" | "disk" => Some(WorkloadType::Storage),
        _ => None,
    }
}

fn parse_service(s: &str) -> Option<ServiceType> {
    match s.to_lowercase().as_str() {
        "vm" | "compute" | "instance" => Some(ServiceType::Vm),
        "db" | "database" | "sql" | "rds" | "cloudsql" => Some(ServiceType::Database),
        _ => None,
    }
}

fn parse_engine(s: &str) -> Option<DatabaseEngine> {
    match s.to_lowercase().as_str() {
        "postgres" | "postgresql" | "pg" => Some(DatabaseEngine::PostgreSql),
        "mysql" | "maria" | "mariadb" => Some(DatabaseEngine::MySql),
        "sqlserver" | "sql-server" | "mssql" => Some(DatabaseEngine::SqlServer),
        _ => None,
    }
}

fn get_valid_regions(cloud: CloudProvider) -> &'static [&'static str] {
    match cloud {
        CloudProvider::Gcp => &GCP_REGIONS,
        CloudProvider::Aws => &AWS_REGIONS,
        CloudProvider::Azure => &AZURE_REGIONS,
    }
}

fn validate_region(region: &str, cloud: CloudProvider) -> Result<String, String> {
    let region_lower = region.to_lowercase();
    let valid_regions = get_valid_regions(cloud);

    if valid_regions.contains(&region_lower.as_str()) {
        Ok(region_lower)
    } else {
        let provider_name = match cloud {
            CloudProvider::Gcp => "GCP",
            CloudProvider::Aws => "AWS",
            CloudProvider::Azure => "Azure",
        };
        let mut msg = format!("Invalid region '{}' for {}.", region, provider_name);
        if let Some(suggestion) = suggest_region(region, valid_regions) {
            msg.push_str(&format!("\n  Did you mean '{}'?", suggestion));
        }
        msg.push_str(&format!(
            "\n  Run 'kumo --list-regions {}' to see all valid regions.",
            provider_name.to_lowercase()
        ));
        Err(msg)
    }
}

fn handle_list_regions(provider_str: &str) {
    let cloud = parse_cloud(provider_str).unwrap_or_else(|| {
        eprintln!("Error: invalid provider '{}'. Use 'gcp', 'aws', or 'azure'.", provider_str);
        std::process::exit(1);
    });
    let regions = get_valid_regions(cloud);
    let provider_name = match cloud {
        CloudProvider::Gcp => "GCP",
        CloudProvider::Aws => "AWS",
        CloudProvider::Azure => "Azure",
    };
    println!("\n☁  Valid regions for {} ({} regions):\n", provider_name, regions.len());
    for region in regions {
        println!("  • {}", region);
    }
    println!();
}

fn handle_update_catalog() {
    eprintln!("☁  kumo --update-catalog");
    eprintln!();
    eprintln!("  Downloading fresh pricing data...");
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cache_path = std::path::PathBuf::from(&home).join(".kumo").join("catalog.json");
    let _ = std::fs::remove_file(&cache_path);
    pricing::init_live_pricing(models::CloudProvider::Gcp);
    eprintln!("  ✓ Pricing data updated and cached at ~/.kumo/catalog.json");
    eprintln!("  Cache is valid for 24 hours.");
    std::process::exit(0);
}

fn handle_completions(shell_name: &str) {
    let mut cmd = CliArgs::command();
    match shell_name.to_lowercase().as_str() {
        "bash" => generate(shells::Bash, &mut cmd, "kumo", &mut std::io::stdout()),
        "zsh" => generate(shells::Zsh, &mut cmd, "kumo", &mut std::io::stdout()),
        "fish" => generate(shells::Fish, &mut cmd, "kumo", &mut std::io::stdout()),
        _ => {
            eprintln!("Error: unsupported shell '{}'. Use: bash, zsh, or fish.", shell_name);
            std::process::exit(1);
        }
    }
}

fn main() {
    let start = std::time::Instant::now();
    let args = CliArgs::parse();

    // Handle --completions
    if let Some(ref shell) = args.completions {
        handle_completions(shell);
        return;
    }

    // Handle --list-regions
    if let Some(ref provider) = args.list_regions {
        handle_list_regions(provider);
        return;
    }

    // Handle --update-catalog
    if args.update_catalog {
        handle_update_catalog();
        return;
    }

    // Load config defaults
    let cfg = KumoConfig::load();

    // Check for multi-cloud mode
    let is_multicloud = args.cloud.as_deref() == Some("all");

    // Determine if user is attempting flags mode (passed at least one data flag)
    let any_data_flag = args.cloud.is_some()
        || args.vcpu.is_some()
        || args.ram.is_some()
        || args.env.is_some()
        || args.region.is_some()
        || args.workload.is_some()
        || args.service.is_some()
        || args.family.is_some();

    // If user passed any flag, validate all required params are present (args + config)
    if any_data_flag && !is_multicloud {
        let has_cloud = args.cloud.is_some() || cfg.defaults.cloud.is_some();
        let has_vcpu = args.vcpu.is_some();
        let has_ram = args.ram.is_some();
        let has_env = args.env.is_some() || cfg.defaults.env.is_some();
        let has_region = args.region.is_some() || cfg.defaults.region.is_some();

        let mut missing = Vec::new();
        if !has_cloud { missing.push("--cloud (-c)"); }
        if !has_vcpu { missing.push("--vcpu (-v)"); }
        if !has_ram { missing.push("--ram (-r)"); }
        if !has_env { missing.push("--env (-e)"); }
        if !has_region { missing.push("--region"); }

        if !missing.is_empty() {
            eprintln!("Error: missing required parameters: {}", missing.join(", "));
            eprintln!("  Provide all required flags or run 'kumo' without flags for interactive mode.");
            eprintln!("  You can also set defaults in ~/.kumo/config.toml");
            std::process::exit(1);
        }
    }

    let request = if is_multicloud {
        // Multi-cloud mode: need vcpu, ram, env (region not required)
        let vcpu = args.vcpu.or_else(|| cfg.defaults.cloud.as_deref().and(None)).unwrap_or_else(|| {
            eprintln!("Error: --vcpu is required for multi-cloud compare.");
            std::process::exit(1);
        });
        let ram = args.ram.unwrap_or_else(|| {
            eprintln!("Error: --ram is required for multi-cloud compare.");
            std::process::exit(1);
        });
        let env_str = args.env.as_deref()
            .or(cfg.defaults.env.as_deref())
            .unwrap_or_else(|| {
                eprintln!("Error: --env is required for multi-cloud compare.");
                std::process::exit(1);
            });
        let env = parse_env(env_str).unwrap_or_else(|| {
            eprintln!("Error: invalid environment '{}'.", env_str);
            std::process::exit(1);
        });
        let workload_str = args.workload.as_deref().or(cfg.defaults.workload.as_deref()).unwrap_or("general");
        let workload = parse_workload(workload_str).unwrap_or(WorkloadType::General);

        // Build a base request (region will be set per-provider)
        UserRequest {
            cloud: CloudProvider::Gcp, // placeholder
            vcpus: vcpu,
            ram_gb: ram,
            env,
            region: String::new(),
            family: args.family.clone(),
            workload,
            service: ServiceType::Vm,
            db_engine: None,
            gpu_type: args.gpu_type.clone(),
            gpu_count: args.gpu_count,
        }
    } else if any_data_flag {
        // All args provided — flags mode
        let cloud_str = args.cloud.as_deref().or(cfg.defaults.cloud.as_deref()).unwrap_or("");
        let cloud = parse_cloud(cloud_str).unwrap_or_else(|| {
            eprintln!("Error: invalid cloud provider. Use 'gcp', 'aws', 'azure', or 'all'.");
            std::process::exit(1);
        });

        let vcpu = args.vcpu.unwrap();
        if vcpu <= 0.0 || vcpu.is_nan() || vcpu.is_infinite() {
            eprintln!("Error: vCPUs must be a valid number greater than 0.");
            std::process::exit(1);
        }
        if vcpu > 512.0 {
            eprintln!("Error: vCPUs cannot exceed 512.");
            std::process::exit(1);
        }

        let ram = args.ram.unwrap();
        if ram <= 0.0 || ram.is_nan() || ram.is_infinite() {
            eprintln!("Error: RAM must be a valid number greater than 0.");
            std::process::exit(1);
        }
        if ram > 12288.0 {
            eprintln!("Error: RAM cannot exceed 12288 GB.");
            std::process::exit(1);
        }

        let env_str = args.env.as_deref().or(cfg.defaults.env.as_deref()).unwrap_or("");
        let env = parse_env(env_str).unwrap_or_else(|| {
            eprintln!("Error: invalid environment. Use 'production', 'pre-production', 'test', or 'dev'.");
            std::process::exit(1);
        });

        let region_str = args.region.as_deref().or(cfg.defaults.region.as_deref()).unwrap_or("");
        let region = validate_region(region_str, cloud).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

        let workload_str = args.workload.as_deref().or(cfg.defaults.workload.as_deref()).unwrap_or("general");
        let workload = parse_workload(workload_str).unwrap_or_else(|| {
            eprintln!("Error: invalid workload. Use: general, ai, sap, hpc, database, batch, storage.");
            std::process::exit(1);
        });

        let service_str = args.service.as_deref().or(cfg.defaults.service.as_deref()).unwrap_or("vm");
        let service = parse_service(service_str).unwrap_or(ServiceType::Vm);

        let db_engine = if service == ServiceType::Database {
            let engine_str = args.engine.as_deref().or(cfg.defaults.engine.as_deref());
            if let Some(e) = engine_str {
                Some(parse_engine(e).unwrap_or_else(|| {
                    eprintln!("Error: invalid engine '{}'. Use: postgres, mysql, or sqlserver.", e);
                    std::process::exit(1);
                }))
            } else {
                eprintln!("Error: --engine is required when --service db.");
                std::process::exit(1);
            }
        } else {
            None
        };

        UserRequest {
            cloud,
            vcpus: vcpu,
            ram_gb: ram,
            env,
            region,
            family: args.family,
            workload,
            service,
            db_engine,
            gpu_type: args.gpu_type,
            gpu_count: args.gpu_count,
        }
    } else {
        // Missing args — interactive mode
        interactive::run_interactive()
    };

    if is_multicloud {
        // Multi-cloud compare: run recommendation for each provider
        let providers = [CloudProvider::Gcp, CloudProvider::Aws, CloudProvider::Azure];
        let default_regions = ["us-central1", "us-east-1", "eastus"];

        let mut results = Vec::new();
        for (i, &cloud) in providers.iter().enumerate() {
            pricing::init_live_pricing(cloud);
            let mut req = request.clone();
            req.cloud = cloud;
            req.region = default_regions[i].to_string();
            let rec = recommend::recommend(&req);
            results.push((cloud, rec));
        }

        output::display_multicloud(&results, &request, start);
    } else {
        // Single provider mode
        pricing::init_live_pricing(request.cloud);
        let recommendation = recommend::recommend(&request);
        output::display(&recommendation, &request, start);
    }
}
