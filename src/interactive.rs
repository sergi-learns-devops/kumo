use dialoguer::{Input, Select};

use crate::models::{CloudProvider, DatabaseEngine, Environment, ServiceType, UserRequest, WorkloadType};
use crate::providers::aws::AWS_REGIONS;
use crate::providers::azure::AZURE_REGIONS;
use crate::providers::gcp::GCP_REGIONS;

pub fn run_interactive() -> UserRequest {
    if !atty_check() {
        eprintln!("Error: interactive mode requires a terminal (TTY).");
        eprintln!("Use flags instead: kumo --cloud gcp --vcpu 4 --ram 16 --env prod --region us-central1");
        eprintln!("Run 'kumo --help' for all options.");
        std::process::exit(1);
    }

    println!();
    println!("  ☁  kumo — Cloud Instance Recommender");
    println!();

    // 1. Cloud provider
    let cloud_items = &[
        "GCP (Google Cloud Platform)",
        "AWS (Amazon Web Services)",
        "Azure (Microsoft Azure)",
    ];
    let cloud_idx = Select::new()
        .with_prompt("  Cloud provider")
        .items(cloud_items)
        .default(0)
        .interact()
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    let cloud = match cloud_idx {
        0 => CloudProvider::Gcp,
        1 => CloudProvider::Aws,
        _ => CloudProvider::Azure,
    };

    // 2. Service type
    let service_items = &[
        "Virtual Machine (Compute Engine / EC2 / Azure VM)",
        "Managed Database (Cloud SQL / RDS / Azure Database)",
    ];
    let service_idx = Select::new()
        .with_prompt("  Service type")
        .items(service_items)
        .default(0)
        .interact()
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    let service = match service_idx {
        0 => ServiceType::Vm,
        _ => ServiceType::Database,
    };

    // 2b. Database engine (only if DB)
    let db_engine = if service == ServiceType::Database {
        let engine_items = &["PostgreSQL", "MySQL", "SQL Server"];
        let engine_idx = Select::new()
            .with_prompt("  Database engine")
            .items(engine_items)
            .default(0)
            .interact()
            .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
        Some(match engine_idx {
            0 => DatabaseEngine::PostgreSql,
            1 => DatabaseEngine::MySql,
            _ => DatabaseEngine::SqlServer,
        })
    } else {
        None
    };

    // 3. Workload type (only for VMs)
    let workload = if service == ServiceType::Vm {
        let workload_items = &[
            "General purpose (web, APIs, microservices)",
            "AI / Machine Learning (GPU-accelerated)",
            "SAP (certified, high-memory)",
            "HPC (high-performance computing)",
            "Databases (memory-optimized, high I/O)",
            "Batch / CI-CD (cost-optimized, ephemeral)",
            "Storage-intensive (high throughput, local NVMe)",
        ];
        let workload_idx = Select::new()
            .with_prompt("  Workload type")
            .items(workload_items)
            .default(0)
            .interact()
            .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

        match workload_idx {
            0 => WorkloadType::General,
            1 => WorkloadType::Ai,
            2 => WorkloadType::Sap,
            3 => WorkloadType::Hpc,
            4 => WorkloadType::Database,
            5 => WorkloadType::Batch,
            _ => WorkloadType::Storage,
        }
    } else {
        WorkloadType::General
    };

    // 3b. GPU selection (only for AI workload)
    let (gpu_type, gpu_count) = if workload == WorkloadType::Ai {
        println!();
        let available = match cloud {
            CloudProvider::Gcp => "L4 (24GB), A100 (40GB)",
            CloudProvider::Aws => "T4 (16GB), A10G (24GB), A100 (40GB)",
            CloudProvider::Azure => "T4 (16GB), A100 (80GB)",
        };
        println!("  \x1b[2m  Available GPUs for {}: {}\x1b[0m", cloud, available);

        let gpu_items: Vec<&str> = match cloud {
            CloudProvider::Gcp => vec!["any — show all GPU instances", "L4 (24 GB VRAM)", "A100 (40 GB VRAM)"],
            CloudProvider::Aws => vec!["any — show all GPU instances", "T4 (16 GB VRAM)", "A10G (24 GB VRAM)", "A100 (40 GB VRAM)"],
            CloudProvider::Azure => vec!["any — show all GPU instances", "T4 (16 GB VRAM)", "A100 (80 GB VRAM)"],
        };

        let gpu_idx = Select::new()
            .with_prompt("  GPU model")
            .items(&gpu_items)
            .default(0)
            .interact()
            .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

        let selected_gpu = if gpu_idx == 0 {
            None
        } else {
            Some(gpu_items[gpu_idx].split(' ').next().unwrap_or("").to_string())
        };

        let count: u8 = Input::new()
            .with_prompt("  Minimum GPU count")
            .default(1u8)
            .interact_text()
            .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

        (selected_gpu, Some(count))
    } else {
        (None, None)
    };

    // 4. Environment
    let env_items = &["Production 🔴", "Pre-Production 🟡", "Test 🔵", "Dev 🟢"];
    let env_idx = Select::new()
        .with_prompt("  Target environment")
        .items(env_items)
        .default(0)
        .interact()
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    let env = match env_idx {
        0 => Environment::Production,
        1 => Environment::PreProduction,
        2 => Environment::Test,
        _ => Environment::Dev,
    };

    // Tier hint
    let tier_hint = match env {
        Environment::Production | Environment::PreProduction | Environment::Test => "Standard-tier instances",
        Environment::Dev => "Economy-tier instances",
    };
    println!("  \x1b[2m  → Will recommend: {}\x1b[0m", tier_hint);

    // 5. Region
    let valid_regions: &[&str] = match cloud {
        CloudProvider::Gcp => &GCP_REGIONS,
        CloudProvider::Aws => &AWS_REGIONS,
        CloudProvider::Azure => &AZURE_REGIONS,
    };

    let default_region = detect_default_region(cloud);
    let region_prompt = match &default_region {
        Some(r) => format!("  Region [default: {}]", r),
        None => format!("  Region (e.g., {})", valid_regions[0]),
    };

    let region: String = Input::new()
        .with_prompt(&region_prompt)
        .default(default_region.unwrap_or_default())
        .validate_with(move |input: &String| -> Result<(), String> {
            if input.is_empty() { return Err("Region cannot be empty".to_string()); }
            let input_lower = input.to_lowercase();
            if valid_regions.contains(&input_lower.as_str()) {
                Ok(())
            } else {
                let mut msg = format!("Invalid region '{}'.", input);
                if let Some(suggestion) = crate::recommend::suggest_region(&input_lower, valid_regions) {
                    msg.push_str(&format!(" Did you mean '{}'?", suggestion));
                }
                Err(msg)
            }
        })
        .interact_text()
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    // 6. vCPUs
    let vcpus: f32 = Input::new()
        .with_prompt("  vCPUs (e.g., 0.5, 1, 2, 4, 8, 16)")
        .default(4.0_f32)
        .validate_with(|input: &f32| -> Result<(), &str> {
            if *input <= 0.0 { Err("vCPUs must be greater than 0") }
            else if *input > 512.0 { Err("vCPUs cannot exceed 512") }
            else { Ok(()) }
        })
        .interact_text()
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    // 7. RAM
    let ram_gb: f32 = Input::new()
        .with_prompt("  RAM in GB (e.g., 2, 4, 8, 16, 32)")
        .default(16.0_f32)
        .validate_with(|input: &f32| -> Result<(), &str> {
            if *input <= 0.0 { Err("RAM must be greater than 0") }
            else if *input > 12288.0 { Err("RAM cannot exceed 12288 GB") }
            else { Ok(()) }
        })
        .interact_text()
        .unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });

    UserRequest {
        cloud,
        vcpus,
        ram_gb,
        env,
        region,
        family: None,
        workload,
        service,
        db_engine,
        gpu_type,
        gpu_count,
    }
}

fn detect_default_region(cloud: CloudProvider) -> Option<String> {
    match cloud {
        CloudProvider::Gcp => std::env::var("CLOUDSDK_COMPUTE_REGION")
            .or_else(|_| std::env::var("GOOGLE_CLOUD_REGION")).ok(),
        CloudProvider::Aws => std::env::var("AWS_DEFAULT_REGION")
            .or_else(|_| std::env::var("AWS_REGION")).ok(),
        CloudProvider::Azure => std::env::var("AZURE_DEFAULTS_LOCATION").ok(),
    }
}

fn atty_check() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}
