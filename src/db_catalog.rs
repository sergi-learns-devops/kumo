use crate::models::{CloudProvider, DatabaseEngine, InstanceTier, InstanceType};

/// Get DB instance class catalog for a given provider + engine
pub fn get_db_catalog(
    cloud: CloudProvider,
    engine: DatabaseEngine,
) -> &'static [InstanceType] {
    match (cloud, engine) {
        (CloudProvider::Gcp, _) => &CLOUD_SQL,
        (CloudProvider::Aws, DatabaseEngine::SqlServer) => &RDS_SQL_SERVER,
        (CloudProvider::Aws, _) => &RDS,
        (CloudProvider::Azure, _) => &AZURE_DB,
    }
}

/// Get DB-specific notes
pub fn db_notes(cloud: CloudProvider, engine: DatabaseEngine) -> Vec<&'static str> {
    let mut notes = Vec::new();

    match cloud {
        CloudProvider::Gcp => {
            notes.push("Cloud SQL pricing includes HA replica cost when enabled.");
            notes.push("Consider read replicas for read-heavy workloads.");
        }
        CloudProvider::Aws => {
            notes.push("RDS Multi-AZ deployments double the instance cost.");
            notes.push("Consider Aurora for PostgreSQL/MySQL for better price/performance.");
        }
        CloudProvider::Azure => {
            notes.push("Azure DB Flexible Server is recommended over Single Server.");
            notes.push("Zone-redundant HA adds ~2x cost to the compute tier.");
        }
    }

    match engine {
        DatabaseEngine::SqlServer => {
            notes.push("SQL Server license is included in the instance cost (License Included model).");
        }
        DatabaseEngine::PostgreSql => {
            notes.push("PostgreSQL extensions availability may vary by instance tier.");
        }
        DatabaseEngine::MySql => {
            notes.push("Consider upgrading to MySQL 8.0+ for improved performance.");
        }
    }

    notes
}

// ─── Cloud SQL (GCP) ────────────────────────────────────────────────────────
// Naming: db-custom-{vcpus}-{ram_mb} or db-{family}-{size}
// All engines (PostgreSQL, MySQL, SQL Server) use the same instance classes

static CLOUD_SQL: [InstanceType; 14] = [
    // Lightweight / Dev (Economy)
    InstanceType { name: "db-f1-micro", family: "F1", vcpus: 0.6, ram_gb: 0.6, tier: InstanceTier::Economy, cost_hour_usd: 0.0150, gpu: None },
    InstanceType { name: "db-g1-small", family: "G1", vcpus: 1.0, ram_gb: 1.7, tier: InstanceTier::Economy, cost_hour_usd: 0.0500, gpu: None },
    // Standard (matches N1 sizing)
    InstanceType { name: "db-n1-standard-1", family: "N1-std", vcpus: 1.0, ram_gb: 3.75, tier: InstanceTier::Standard, cost_hour_usd: 0.0965, gpu: None },
    InstanceType { name: "db-n1-standard-2", family: "N1-std", vcpus: 2.0, ram_gb: 7.5, tier: InstanceTier::Standard, cost_hour_usd: 0.1930, gpu: None },
    InstanceType { name: "db-n1-standard-4", family: "N1-std", vcpus: 4.0, ram_gb: 15.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3860, gpu: None },
    InstanceType { name: "db-n1-standard-8", family: "N1-std", vcpus: 8.0, ram_gb: 30.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7720, gpu: None },
    InstanceType { name: "db-n1-standard-16", family: "N1-std", vcpus: 16.0, ram_gb: 60.0, tier: InstanceTier::Standard, cost_hour_usd: 1.5440, gpu: None },
    InstanceType { name: "db-n1-standard-32", family: "N1-std", vcpus: 32.0, ram_gb: 120.0, tier: InstanceTier::Standard, cost_hour_usd: 3.0880, gpu: None },
    InstanceType { name: "db-n1-standard-64", family: "N1-std", vcpus: 64.0, ram_gb: 240.0, tier: InstanceTier::Standard, cost_hour_usd: 6.1760, gpu: None },
    InstanceType { name: "db-n1-standard-96", family: "N1-std", vcpus: 96.0, ram_gb: 360.0, tier: InstanceTier::Standard, cost_hour_usd: 9.2640, gpu: None },
    // High-memory
    InstanceType { name: "db-n1-highmem-2", family: "N1-highmem", vcpus: 2.0, ram_gb: 13.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2360, gpu: None },
    InstanceType { name: "db-n1-highmem-4", family: "N1-highmem", vcpus: 4.0, ram_gb: 26.0, tier: InstanceTier::Standard, cost_hour_usd: 0.4720, gpu: None },
    InstanceType { name: "db-n1-highmem-8", family: "N1-highmem", vcpus: 8.0, ram_gb: 52.0, tier: InstanceTier::Standard, cost_hour_usd: 0.9440, gpu: None },
    InstanceType { name: "db-n1-highmem-16", family: "N1-highmem", vcpus: 16.0, ram_gb: 104.0, tier: InstanceTier::Standard, cost_hour_usd: 1.8880, gpu: None },
];

// ─── RDS (AWS) — PostgreSQL / MySQL ─────────────────────────────────────────

static RDS: [InstanceType; 16] = [
    // Burstable (Economy)
    InstanceType { name: "db.t3.micro", family: "T3", vcpus: 2.0, ram_gb: 1.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0170, gpu: None },
    InstanceType { name: "db.t3.small", family: "T3", vcpus: 2.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0340, gpu: None },
    InstanceType { name: "db.t3.medium", family: "T3", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0680, gpu: None },
    InstanceType { name: "db.t3.large", family: "T3", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1360, gpu: None },
    InstanceType { name: "db.t3.xlarge", family: "T3", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.2720, gpu: None },
    InstanceType { name: "db.t3.2xlarge", family: "T3", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.5440, gpu: None },
    // General Purpose (Standard)
    InstanceType { name: "db.m6i.large", family: "M6i", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1710, gpu: None },
    InstanceType { name: "db.m6i.xlarge", family: "M6i", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3420, gpu: None },
    InstanceType { name: "db.m6i.2xlarge", family: "M6i", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6840, gpu: None },
    InstanceType { name: "db.m6i.4xlarge", family: "M6i", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3680, gpu: None },
    InstanceType { name: "db.m6i.8xlarge", family: "M6i", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 2.7360, gpu: None },
    // Memory Optimized (Standard)
    InstanceType { name: "db.r6i.large", family: "R6i", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2500, gpu: None },
    InstanceType { name: "db.r6i.xlarge", family: "R6i", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5000, gpu: None },
    InstanceType { name: "db.r6i.2xlarge", family: "R6i", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0000, gpu: None },
    InstanceType { name: "db.r6i.4xlarge", family: "R6i", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0000, gpu: None },
    InstanceType { name: "db.r6i.8xlarge", family: "R6i", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 4.0000, gpu: None },
];

// ─── RDS SQL Server (AWS) — higher cost due to license ──────────────────────

static RDS_SQL_SERVER: [InstanceType; 8] = [
    InstanceType { name: "db.t3.small", family: "T3", vcpus: 2.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0590, gpu: None },
    InstanceType { name: "db.t3.medium", family: "T3", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1180, gpu: None },
    InstanceType { name: "db.t3.xlarge", family: "T3", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.4700, gpu: None },
    InstanceType { name: "db.m6i.large", family: "M6i", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.4840, gpu: None },
    InstanceType { name: "db.m6i.xlarge", family: "M6i", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.9680, gpu: None },
    InstanceType { name: "db.m6i.2xlarge", family: "M6i", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 1.9360, gpu: None },
    InstanceType { name: "db.r6i.large", family: "R6i", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5700, gpu: None },
    InstanceType { name: "db.r6i.xlarge", family: "R6i", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 1.1400, gpu: None },
];

// ─── Azure Database (Flexible Server) ───────────────────────────────────────
// Tiers: Burstable (B), General Purpose (GP), Memory Optimized (MO)

static AZURE_DB: [InstanceType; 14] = [
    // Burstable (Economy)
    InstanceType { name: "B_Standard_B1s", family: "Burstable", vcpus: 1.0, ram_gb: 1.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0124, gpu: None },
    InstanceType { name: "B_Standard_B1ms", family: "Burstable", vcpus: 1.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0248, gpu: None },
    InstanceType { name: "B_Standard_B2s", family: "Burstable", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0496, gpu: None },
    InstanceType { name: "B_Standard_B2ms", family: "Burstable", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0992, gpu: None },
    InstanceType { name: "B_Standard_B4ms", family: "Burstable", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1984, gpu: None },
    // General Purpose (Standard)
    InstanceType { name: "GP_Standard_D2ds_v4", family: "GP", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1660, gpu: None },
    InstanceType { name: "GP_Standard_D4ds_v4", family: "GP", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3320, gpu: None },
    InstanceType { name: "GP_Standard_D8ds_v4", family: "GP", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6640, gpu: None },
    InstanceType { name: "GP_Standard_D16ds_v4", family: "GP", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3280, gpu: None },
    InstanceType { name: "GP_Standard_D32ds_v4", family: "GP", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 2.6560, gpu: None },
    // Memory Optimized (Standard)
    InstanceType { name: "MO_Standard_E2ds_v4", family: "MO", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2220, gpu: None },
    InstanceType { name: "MO_Standard_E4ds_v4", family: "MO", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.4440, gpu: None },
    InstanceType { name: "MO_Standard_E8ds_v4", family: "MO", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.8880, gpu: None },
    InstanceType { name: "MO_Standard_E16ds_v4", family: "MO", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.7760, gpu: None },
];
