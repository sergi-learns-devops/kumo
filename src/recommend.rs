use crate::db_catalog;
use crate::models::*;
use crate::pricing;
use crate::providers::aws::AwsProvider;
use crate::providers::azure::AzureProvider;
use crate::providers::gcp::GcpProvider;
use crate::providers::ProviderCatalog;
use crate::workloads;

pub fn recommend(request: &UserRequest) -> Recommendation {
    // 1. Select catalog based on service type
    let catalog: &'static [InstanceType] = if request.service == ServiceType::Database {
        // Use DB instance class catalog
        let engine = request.db_engine.unwrap_or(DatabaseEngine::PostgreSql);
        db_catalog::get_db_catalog(request.cloud, engine)
    } else if request.workload != WorkloadType::General {
        // Use workload catalog (dynamic from live, fallback to static)
        match workloads::get_workload_catalog(request.workload, request.cloud) {
            Some(wl_catalog) => wl_catalog,
            None => get_default_catalog(request),
        }
    } else {
        get_default_catalog(request)
    };

    // 2. Filter by tier and workload patterns
    let filtered: Vec<&'static InstanceType> = if request.service == ServiceType::Database {
        // DB: filter by tier
        let target_tier = match request.env {
            Environment::Production | Environment::PreProduction | Environment::Test => InstanceTier::Standard,
            Environment::Dev => InstanceTier::Economy,
        };
        let tier_filtered: Vec<&'static InstanceType> = catalog.iter().filter(|i| i.tier == target_tier).collect();
        if tier_filtered.is_empty() { catalog.iter().collect() } else { tier_filtered }
    } else if request.workload != WorkloadType::General {
        // Specialized workloads: filter by workload patterns (for live catalog)
        // then by tier
        let workload_filtered = workloads::filter_by_workload(catalog, request.workload, request.cloud);
        if workload_filtered.is_empty() {
            // No matches with pattern filtering — use all (static catalog case)
            catalog.iter().collect()
        } else {
            workload_filtered
        }
    } else {
        // General: filter by tier based on environment
        let target_tier = match request.env {
            Environment::Production | Environment::PreProduction | Environment::Test => InstanceTier::Standard,
            Environment::Dev => InstanceTier::Economy,
        };
        catalog.iter().filter(|i| i.tier == target_tier).collect()
    };

    // 3. Filter by family if specified
    let filtered: Vec<&'static InstanceType> = if let Some(ref family) = request.family {
        let family_lower = family.to_lowercase();
        filtered.into_iter().filter(|i| i.family.to_lowercase() == family_lower).collect()
    } else {
        filtered
    };

    // 3b. Filter by GPU type and count (for AI workloads)
    let filtered: Vec<&'static InstanceType> = if request.gpu_type.is_some() || request.gpu_count.is_some() {
        filtered.into_iter().filter(|i| {
            if let Some(ref gpu) = i.gpu {
                let type_ok = match &request.gpu_type {
                    Some(t) => gpu.model.to_lowercase() == t.to_lowercase(),
                    None => true,
                };
                let count_ok = match request.gpu_count {
                    Some(c) => gpu.count >= c,
                    None => true,
                };
                type_ok && count_ok
            } else {
                false // No GPU on this instance, filter it out
            }
        }).collect()
    } else {
        filtered
    };

    // 3c. Filter by minimum network bandwidth if specified
    let filtered: Vec<&'static InstanceType> = if let Some(min_net) = request.min_network_gbps {
        filtered.into_iter().filter(|i| i.effective_network_gbps() >= min_net).collect()
    } else {
        filtered
    };

    if filtered.is_empty() {
        return Recommendation {
            exact_match: None,
            closest_matches: vec![],
            custom_option: build_custom(request),
            notes: vec!["No instances found matching your criteria."],
        };
    }

    // 4. Look for exact match
    let exact_match = filtered
        .iter()
        .find(|i| {
            (i.vcpus - request.vcpus).abs() < 0.01 && (i.ram_gb - request.ram_gb).abs() < 0.01
        })
        .copied();

    // 5. Calculate scores for all instances
    let max_cost = filtered
        .iter()
        .map(|i| i.cost_hour_usd)
        .fold(0.0_f32, f32::max);

    let max_net = filtered
        .iter()
        .map(|i| i.effective_network_gbps())
        .fold(0.0_f32, f32::max);

    let req_cpu = if request.vcpus > 0.0 { request.vcpus } else { 1.0 };
    let req_ram = if request.ram_gb > 0.0 { request.ram_gb } else { 1.0 };

    // Network-aware weighting: if the user cares about network (min_network set),
    // give it weight; otherwise keep the classic CPU/RAM/cost split.
    let network_matters = request.min_network_gbps.is_some();
    let (w_cpu, w_ram, w_cost, w_net) = if network_matters {
        (0.30_f32, 0.30, 0.25, 0.15)
    } else {
        (0.35_f32, 0.35, 0.30, 0.0)
    };

    let mut scored: Vec<ScoredInstance> = filtered
        .iter()
        .map(|instance| {
            let delta_vcpu = instance.vcpus - request.vcpus;
            let delta_ram = instance.ram_gb - request.ram_gb;

            let cpu_diff = delta_vcpu.abs() / req_cpu;
            let ram_diff = delta_ram.abs() / req_ram;
            let normalized_cost = if max_cost > 0.0 {
                instance.cost_hour_usd / max_cost
            } else {
                0.0
            };
            // Network: higher bandwidth = better, so reward proximity to the max
            let net = instance.effective_network_gbps();
            let net_deficit = if max_net > 0.0 { 1.0 - (net / max_net) } else { 0.0 };

            let cpu_component = w_cpu * cpu_diff;
            let ram_component = w_ram * ram_diff;
            let cost_component = w_cost * normalized_cost;
            let network_component = w_net * net_deficit;

            let score = 1.0 - (cpu_component + ram_component + cost_component + network_component);

            ScoredInstance {
                instance,
                score,
                delta_vcpu,
                delta_ram,
                breakdown: ScoreBreakdown {
                    cpu_component,
                    ram_component,
                    cost_component,
                    network_component,
                    cpu_weight: w_cpu,
                    ram_weight: w_ram,
                    cost_weight: w_cost,
                    network_weight: w_net,
                },
            }
        })
        .collect();

    // 6. Sort by score descending, take top 3
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let closest_matches: Vec<ScoredInstance> = scored.into_iter().take(3).collect();

    // 7. Custom machine option for GCP
    let custom_option = build_custom(request);

    // 8. Notes based on environment
    let notes = build_notes(request);

    Recommendation {
        exact_match,
        closest_matches,
        custom_option,
        notes,
    }
}

fn build_custom(request: &UserRequest) -> Option<CustomMachine> {
    // No custom machines for DB service or SAP workload
    if request.service == ServiceType::Database || request.workload == WorkloadType::Sap {
        return None;
    }

    match request.cloud {
        CloudProvider::Gcp => {
            let vcpus_rounded = if request.vcpus < 1.0 {
                1.0_f32
            } else {
                (request.vcpus.ceil() as u32) as f32
            };
            let ram_mb = (request.ram_gb * 1024.0) as u32;
            // Round RAM to nearest 256 MB (GCP requirement)
            let ram_mb_rounded = ((ram_mb + 255) / 256) * 256;
            let ram_gb_rounded = ram_mb_rounded as f32 / 1024.0;
            let name = format!("n2-custom-{}-{}", vcpus_rounded as u32, ram_mb_rounded);
            let estimated_cost_hour = vcpus_rounded * 0.03398 + ram_gb_rounded * 0.00455;

            Some(CustomMachine {
                name,
                vcpus: vcpus_rounded,
                ram_gb: ram_gb_rounded,
                estimated_cost_hour,
            })
        }
        CloudProvider::Aws | CloudProvider::Azure => None,
    }
}

fn build_notes(request: &UserRequest) -> Vec<&'static str> {
    let mut notes = Vec::new();

    // DB-specific notes
    if request.service == ServiceType::Database {
        let engine = request.db_engine.unwrap_or(DatabaseEngine::PostgreSql);
        notes.extend(db_catalog::db_notes(request.cloud, engine));
        return notes;
    }

    // Workload-specific notes
    notes.extend(workloads::workload_notes(request.workload));

    // Environment-specific notes (only if no workload notes or General)
    if request.workload == WorkloadType::General {
        match request.env {
            Environment::Production => {
                notes.push("Consider Committed Use Discounts (CUDs) for 1-3 year savings up to 57%.");
            }
            Environment::PreProduction => {
                notes.push("Use Spot/Preemptible VMs for pre-production to save up to 80%.");
                notes.push("Consider scheduling instance shutdowns outside business hours.");
            }
            Environment::Test => {
                notes.push("Using production-grade instances for realistic testing.");
                notes.push("Use Spot/Preemptible VMs to reduce cost up to 80%.");
                notes.push("Consider auto-shutdown — tests don't need 24/7 uptime.");
            }
            Environment::Dev => {
                notes.push("Using economy-tier instances for minimum cost.");
                notes.push("Use Spot/Preemptible for additional savings.");
                notes.push("Burstable instances (E2/T3/B) are ideal for dev workloads.");
                notes.push("Schedule auto-shutdown outside working hours for ~65% savings.");
            }
        }
    }

    if request.vcpus > 64.0 {
        notes.push("High vCPU count: ensure your workload actually uses all cores.");
    }

    if request.ram_gb > 256.0 {
        notes.push("High memory request: consider memory-optimized instance families.");
    }

    notes
}

fn get_default_catalog(request: &UserRequest) -> &'static [InstanceType] {
    match pricing::get_live_catalog(request.cloud) {
        Some(live) if !live.is_empty() => live,
        _ => match request.cloud {
            CloudProvider::Gcp => GcpProvider::catalog(),
            CloudProvider::Aws => AwsProvider::catalog(),
            CloudProvider::Azure => AzureProvider::catalog(),
        },
    }
}

/// Calculate Levenshtein distance between two strings (for did-you-mean)
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len { matrix[i][0] = i; }
    for j in 0..=b_len { matrix[0][j] = j; }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

/// Suggest the closest valid region (did-you-mean)
pub fn suggest_region(input: &str, valid_regions: &[&str]) -> Option<String> {
    let input_lower = input.to_lowercase();
    let mut best_match: Option<(&str, usize)> = None;

    for region in valid_regions {
        let dist = levenshtein_distance(&input_lower, region);
        if let Some((_, best_dist)) = best_match {
            if dist < best_dist {
                best_match = Some((region, dist));
            }
        } else {
            best_match = Some((region, dist));
        }
    }

    // Only suggest if distance is reasonable (max 3 edits for short names, 4 for longer)
    if let Some((region, dist)) = best_match {
        let threshold = if region.len() > 10 { 4 } else { 3 };
        if dist <= threshold && dist > 0 {
            return Some(region.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(cloud: CloudProvider, vcpus: f32, ram_gb: f32, env: Environment) -> UserRequest {
        UserRequest {
            cloud,
            vcpus,
            ram_gb,
            env,
            region: match cloud {
                CloudProvider::Gcp => "us-central1".to_string(),
                CloudProvider::Aws => "us-east-1".to_string(),
                CloudProvider::Azure => "eastus".to_string(),
            },
            family: None,
            workload: WorkloadType::General,
            service: ServiceType::Vm,
            db_engine: None,
            gpu_type: None,
            gpu_count: None,
            spot: false,
            explain: false,
            min_network_gbps: None,
        }
    }

    #[test]
    fn test_gcp_exact_match_production() {
        let req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Production);
        let rec = recommend(&req);
        assert!(rec.exact_match.is_some());
        let exact = rec.exact_match.unwrap();
        assert_eq!(exact.name, "n2-standard-4");
    }

    #[test]
    fn test_aws_exact_match_dev() {
        let req = make_request(CloudProvider::Aws, 4.0, 16.0, Environment::Dev);
        let rec = recommend(&req);
        assert!(rec.exact_match.is_some());
        let exact = rec.exact_match.unwrap();
        assert_eq!(exact.name, "t3.xlarge");
    }

    #[test]
    fn test_azure_exact_match_production() {
        let req = make_request(CloudProvider::Azure, 4.0, 16.0, Environment::Production);
        let rec = recommend(&req);
        assert!(rec.exact_match.is_some());
        let exact = rec.exact_match.unwrap();
        assert_eq!(exact.name, "Standard_D4s_v5");
    }

    #[test]
    fn test_azure_dev_tier() {
        let req = make_request(CloudProvider::Azure, 2.0, 4.0, Environment::Dev);
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert_eq!(scored.instance.tier, InstanceTier::Economy);
        }
    }

    #[test]
    fn test_no_exact_match_returns_alternatives() {
        let req = make_request(CloudProvider::Gcp, 6.0, 24.0, Environment::Production);
        let rec = recommend(&req);
        assert!(rec.exact_match.is_none());
        assert!(!rec.closest_matches.is_empty());
        assert!(rec.closest_matches.len() <= 3);
    }

    #[test]
    fn test_closest_matches_sorted_by_score_desc() {
        let req = make_request(CloudProvider::Aws, 8.0, 32.0, Environment::Production);
        let rec = recommend(&req);
        for window in rec.closest_matches.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }

    #[test]
    fn test_custom_machine_gcp_only() {
        let gcp_req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Production);
        assert!(recommend(&gcp_req).custom_option.is_some());

        let aws_req = make_request(CloudProvider::Aws, 4.0, 16.0, Environment::Production);
        assert!(recommend(&aws_req).custom_option.is_none());

        let azure_req = make_request(CloudProvider::Azure, 4.0, 16.0, Environment::Production);
        assert!(recommend(&azure_req).custom_option.is_none());
    }

    #[test]
    fn test_family_filter() {
        let mut req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Production);
        req.family = Some("C2".to_string());
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert_eq!(scored.instance.family, "C2");
        }
    }

    #[test]
    fn test_family_filter_case_insensitive() {
        let mut req = make_request(CloudProvider::Aws, 4.0, 16.0, Environment::Production);
        req.family = Some("m6i".to_string());
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert_eq!(scored.instance.family, "M6i");
        }
    }

    #[test]
    fn test_family_filter_no_results() {
        let mut req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Production);
        req.family = Some("NONEXISTENT".to_string());
        let rec = recommend(&req);
        assert!(rec.closest_matches.is_empty());
    }

    #[test]
    fn test_fractional_vcpu_request() {
        let req = make_request(CloudProvider::Gcp, 0.5, 2.0, Environment::Dev);
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
        // Should match e2-small (0.5 vCPU, 2.0 GB)
        assert!(rec.exact_match.is_some());
        assert_eq!(rec.exact_match.unwrap().name, "e2-small");
    }

    #[test]
    fn test_fractional_vcpu_aws() {
        let req = make_request(CloudProvider::Aws, 0.5, 0.5, Environment::Dev);
        let rec = recommend(&req);
        assert!(rec.exact_match.is_some());
        assert_eq!(rec.exact_match.unwrap().name, "t3.nano");
    }

    #[test]
    fn test_production_uses_standard_tier() {
        let req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Production);
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert_eq!(scored.instance.tier, InstanceTier::Standard);
        }
    }

    #[test]
    fn test_dev_uses_economy_tier() {
        let req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Dev);
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert_eq!(scored.instance.tier, InstanceTier::Economy);
        }
    }

    #[test]
    fn test_notes_populated_for_all_environments() {
        for env in [Environment::Production, Environment::PreProduction, Environment::Test, Environment::Dev] {
            let req = make_request(CloudProvider::Gcp, 4.0, 16.0, env);
            let rec = recommend(&req);
            assert!(!rec.notes.is_empty(), "Notes should not be empty for {:?}", env);
        }
    }

    #[test]
    fn test_high_vcpu_adds_note() {
        let req = make_request(CloudProvider::Gcp, 128.0, 256.0, Environment::Production);
        let rec = recommend(&req);
        assert!(rec.notes.iter().any(|n| n.contains("High vCPU")));
    }

    #[test]
    fn test_high_ram_adds_note() {
        let req = make_request(CloudProvider::Aws, 32.0, 512.0, Environment::Production);
        let rec = recommend(&req);
        assert!(rec.notes.iter().any(|n| n.contains("High memory")));
    }

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("us-central1", "us-cental1"), 1);
    }

    #[test]
    fn test_suggest_region_typo() {
        use crate::providers::gcp::GCP_REGIONS;
        let suggestion = suggest_region("us-cental1", &GCP_REGIONS);
        assert_eq!(suggestion, Some("us-central1".to_string()));
    }

    #[test]
    fn test_suggest_region_no_match() {
        use crate::providers::gcp::GCP_REGIONS;
        let suggestion = suggest_region("xxxxxxx", &GCP_REGIONS);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_suggest_region_aws_typo() {
        use crate::providers::aws::AWS_REGIONS;
        let suggestion = suggest_region("eu-wset-1", &AWS_REGIONS);
        assert_eq!(suggestion, Some("eu-west-1".to_string()));
    }

    #[test]
    fn test_custom_machine_fractional_vcpu() {
        let req = make_request(CloudProvider::Gcp, 0.5, 2.0, Environment::Dev);
        let rec = recommend(&req);
        let custom = rec.custom_option.unwrap();
        // Fractional vCPU should be rounded up to 1 for custom machine
        assert_eq!(custom.vcpus, 1.0);
    }

    #[test]
    fn test_workload_ai_returns_gpu_instances() {
        let mut req = make_request(CloudProvider::Aws, 8.0, 32.0, Environment::Production);
        req.workload = WorkloadType::Ai;
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
        // Should be from GPU families
        let families: Vec<&str> = rec.closest_matches.iter().map(|s| s.instance.family).collect();
        assert!(families.iter().any(|f| *f == "G5" || *f == "G4dn" || *f == "P4d"));
    }

    #[test]
    fn test_workload_sap_returns_high_memory() {
        let mut req = make_request(CloudProvider::Gcp, 96.0, 1400.0, Environment::Production);
        req.workload = WorkloadType::Sap;
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
        // SAP instances should have very high RAM
        assert!(rec.closest_matches[0].instance.ram_gb >= 900.0);
    }

    #[test]
    fn test_workload_hpc_gcp() {
        let mut req = make_request(CloudProvider::Gcp, 30.0, 120.0, Environment::Production);
        req.workload = WorkloadType::Hpc;
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
        assert!(rec.closest_matches[0].instance.family == "C2");
    }

    #[test]
    fn test_workload_database_memory_optimized() {
        let mut req = make_request(CloudProvider::Aws, 8.0, 64.0, Environment::Production);
        req.workload = WorkloadType::Database;
        let rec = recommend(&req);
        assert!(rec.exact_match.is_some());
        assert_eq!(rec.exact_match.unwrap().family, "R6i");
    }

    #[test]
    fn test_workload_general_still_uses_tiers() {
        let req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Dev);
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert_eq!(scored.instance.tier, InstanceTier::Economy);
        }
    }

    #[test]
    fn test_workload_notes_populated() {
        let mut req = make_request(CloudProvider::Aws, 8.0, 32.0, Environment::Production);
        req.workload = WorkloadType::Ai;
        let rec = recommend(&req);
        assert!(rec.notes.iter().any(|n| n.contains("GPU")));
    }

    #[test]
    fn test_db_service_rds_postgres() {
        let mut req = make_request(CloudProvider::Aws, 4.0, 16.0, Environment::Production);
        req.service = ServiceType::Database;
        req.db_engine = Some(DatabaseEngine::PostgreSql);
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
        // Should recommend db.* instances
        assert!(rec.closest_matches[0].instance.name.starts_with("db."));
    }

    #[test]
    fn test_db_service_cloud_sql() {
        let mut req = make_request(CloudProvider::Gcp, 4.0, 15.0, Environment::Production);
        req.service = ServiceType::Database;
        req.db_engine = Some(DatabaseEngine::MySql);
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
        assert!(rec.closest_matches[0].instance.name.starts_with("db-"));
    }

    #[test]
    fn test_db_service_azure() {
        let mut req = make_request(CloudProvider::Azure, 4.0, 16.0, Environment::Production);
        req.service = ServiceType::Database;
        req.db_engine = Some(DatabaseEngine::PostgreSql);
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
    }

    #[test]
    fn test_db_service_no_custom_machine() {
        let mut req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Production);
        req.service = ServiceType::Database;
        req.db_engine = Some(DatabaseEngine::PostgreSql);
        let rec = recommend(&req);
        assert!(rec.custom_option.is_none());
    }

    #[test]
    fn test_db_service_dev_uses_economy() {
        let mut req = make_request(CloudProvider::Aws, 2.0, 4.0, Environment::Dev);
        req.service = ServiceType::Database;
        req.db_engine = Some(DatabaseEngine::MySql);
        let rec = recommend(&req);
        assert!(!rec.closest_matches.is_empty());
        // Should prefer burstable/economy tier
        assert_eq!(rec.closest_matches[0].instance.tier, InstanceTier::Economy);
    }

    #[test]
    fn test_db_notes_populated() {
        let mut req = make_request(CloudProvider::Aws, 4.0, 16.0, Environment::Production);
        req.service = ServiceType::Database;
        req.db_engine = Some(DatabaseEngine::SqlServer);
        let rec = recommend(&req);
        assert!(rec.notes.iter().any(|n| n.contains("SQL Server") || n.contains("RDS")));
    }

    #[test]
    fn test_min_network_filter() {
        // Request minimum 20 Gbps — should exclude tiny instances
        let mut req = make_request(CloudProvider::Aws, 32.0, 128.0, Environment::Production);
        req.min_network_gbps = Some(20.0);
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert!(
                scored.instance.effective_network_gbps() >= 20.0,
                "{} has {} Gbps, below 20",
                scored.instance.name,
                scored.instance.effective_network_gbps()
            );
        }
    }

    #[test]
    fn test_network_weight_applied_when_requested() {
        let mut req = make_request(CloudProvider::Aws, 8.0, 32.0, Environment::Production);
        req.min_network_gbps = Some(1.0);
        let rec = recommend(&req);
        // When network matters, the breakdown should have a non-zero network weight
        assert!(rec.closest_matches[0].breakdown.network_weight > 0.0);
    }

    #[test]
    fn test_network_weight_zero_by_default() {
        let req = make_request(CloudProvider::Aws, 8.0, 32.0, Environment::Production);
        let rec = recommend(&req);
        assert_eq!(rec.closest_matches[0].breakdown.network_weight, 0.0);
        // Classic weights should sum with cost to 1.0 total
        let b = &rec.closest_matches[0].breakdown;
        assert!((b.cpu_weight + b.ram_weight + b.cost_weight - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_effective_network_estimation() {
        // An instance with 0 catalog bandwidth estimates from vCPUs
        let req = make_request(CloudProvider::Aws, 8.0, 32.0, Environment::Production);
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            assert!(scored.instance.effective_network_gbps() > 0.0);
        }
    }

    #[test]
    fn test_breakdown_sums_to_score() {
        let req = make_request(CloudProvider::Gcp, 4.0, 16.0, Environment::Production);
        let rec = recommend(&req);
        for scored in &rec.closest_matches {
            let b = &scored.breakdown;
            let total_penalty = b.cpu_component + b.ram_component + b.cost_component + b.network_component;
            let expected_score = 1.0 - total_penalty;
            assert!((scored.score - expected_score).abs() < 0.001);
        }
    }
}
