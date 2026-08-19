mod cli;
mod db_catalog;
mod interactive;
mod models;
mod output;
mod pricing;
mod providers;
mod recommend;
mod workloads;

use clap::Parser;
use cli::CliArgs;
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

        // Did-you-mean suggestion
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
        eprintln!(
            "Error: invalid provider '{}'. Use 'gcp', 'aws', or 'azure'.",
            provider_str
        );
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

    // Force refresh by removing the cache
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cache_path = std::path::PathBuf::from(&home).join(".kumo").join("catalog.json");
    let _ = std::fs::remove_file(&cache_path);

    // Trigger download
    pricing::init_live_pricing(models::CloudProvider::Gcp);

    eprintln!("  ✓ Pricing data updated and cached at ~/.kumo/catalog.json");
    eprintln!("  Cache is valid for 24 hours.");
    std::process::exit(0);
}

fn main() {
    let start = std::time::Instant::now();
    let args = CliArgs::parse();

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

    let request = if args.is_complete() {
        // All args provided — flags mode
        let cloud = parse_cloud(args.cloud.as_deref().unwrap_or("")).unwrap_or_else(|| {
            eprintln!("Error: invalid cloud provider. Use 'gcp', 'aws', or 'azure'.");
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

        let env = parse_env(args.env.as_deref().unwrap_or("")).unwrap_or_else(|| {
            eprintln!(
                "Error: invalid environment. Use 'production', 'pre-production', 'test', or 'dev'."
            );
            std::process::exit(1);
        });

        let region =
            validate_region(args.region.as_deref().unwrap_or(""), cloud).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });

        let workload = if let Some(ref w) = args.workload {
            parse_workload(w).unwrap_or_else(|| {
                eprintln!("Error: invalid workload '{}'. Use: general, ai, sap, hpc, database, batch, storage.", w);
                std::process::exit(1);
            })
        } else {
            WorkloadType::General
        };

        let service = if let Some(ref s) = args.service {
            parse_service(s).unwrap_or_else(|| {
                eprintln!("Error: invalid service '{}'. Use: vm or db.", s);
                std::process::exit(1);
            })
        } else {
            ServiceType::Vm
        };

        let db_engine = if service == ServiceType::Database {
            if let Some(ref e) = args.engine {
                Some(parse_engine(e).unwrap_or_else(|| {
                    eprintln!("Error: invalid engine '{}'. Use: postgres, mysql, or sqlserver.", e);
                    std::process::exit(1);
                }))
            } else {
                eprintln!("Error: --engine is required when --service db. Use: postgres, mysql, or sqlserver.");
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
        }
    } else {
        // Missing args — interactive mode
        interactive::run_interactive()
    };

    // Initialize live pricing (checks cache, downloads if needed)
    pricing::init_live_pricing(request.cloud);

    let recommendation = recommend::recommend(&request);
    output::display(&recommendation, &request, start);
}
