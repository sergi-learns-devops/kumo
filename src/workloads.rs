use crate::models::{CloudProvider, InstanceTier, InstanceType, WorkloadType};

/// Get specialized instance families for a given workload + provider combination.
/// Returns None for General (uses the standard catalog with tier filtering).
pub fn get_workload_catalog(
    workload: WorkloadType,
    cloud: CloudProvider,
) -> Option<&'static [InstanceType]> {
    match (workload, cloud) {
        (WorkloadType::General, _) => None, // Use standard catalog
        (WorkloadType::Ai, CloudProvider::Gcp) => Some(&GCP_AI),
        (WorkloadType::Ai, CloudProvider::Aws) => Some(&AWS_AI),
        (WorkloadType::Ai, CloudProvider::Azure) => Some(&AZURE_AI),
        (WorkloadType::Sap, CloudProvider::Gcp) => Some(&GCP_SAP),
        (WorkloadType::Sap, CloudProvider::Aws) => Some(&AWS_SAP),
        (WorkloadType::Sap, CloudProvider::Azure) => Some(&AZURE_SAP),
        (WorkloadType::Hpc, CloudProvider::Gcp) => Some(&GCP_HPC),
        (WorkloadType::Hpc, CloudProvider::Aws) => Some(&AWS_HPC),
        (WorkloadType::Hpc, CloudProvider::Azure) => Some(&AZURE_HPC),
        (WorkloadType::Database, CloudProvider::Gcp) => Some(&GCP_DB),
        (WorkloadType::Database, CloudProvider::Aws) => Some(&AWS_DB),
        (WorkloadType::Database, CloudProvider::Azure) => Some(&AZURE_DB),
        (WorkloadType::Batch, CloudProvider::Gcp) => Some(&GCP_BATCH),
        (WorkloadType::Batch, CloudProvider::Aws) => Some(&AWS_BATCH),
        (WorkloadType::Batch, CloudProvider::Azure) => Some(&AZURE_BATCH),
        (WorkloadType::Storage, CloudProvider::Gcp) => Some(&GCP_STORAGE),
        (WorkloadType::Storage, CloudProvider::Aws) => Some(&AWS_STORAGE),
        (WorkloadType::Storage, CloudProvider::Azure) => Some(&AZURE_STORAGE),
    }
}

/// Get workload-specific tips/notes
pub fn workload_notes(workload: WorkloadType) -> Vec<&'static str> {
    match workload {
        WorkloadType::General => vec![],
        WorkloadType::Ai => vec![
            "GPU instances have separate GPU memory — check model VRAM requirements.",
            "Consider Spot/Preemptible for training jobs to save up to 70%.",
            "For inference, smaller GPU instances (T4) are often more cost-efficient.",
        ],
        WorkloadType::Sap => vec![
            "Ensure the instance type is SAP-certified for your workload.",
            "SAP HANA requires minimum 64 GB RAM per node.",
            "Check SAP Note 1928533 (Azure), 2456432 (GCP), 1656099 (AWS) for supported types.",
        ],
        WorkloadType::Hpc => vec![
            "HPC instances provide high-frequency CPUs and fast interconnects.",
            "Consider placement groups/compact placement for low-latency node communication.",
            "Spot/Preemptible can reduce HPC costs significantly for fault-tolerant workloads.",
        ],
        WorkloadType::Database => vec![
            "Memory-optimized instances improve database buffer cache hit ratios.",
            "Consider local NVMe storage for high-IOPS database workloads.",
            "Persistent disks with provisioned IOPS are recommended for production DBs.",
        ],
        WorkloadType::Batch => vec![
            "Batch workloads are ideal for Spot/Preemptible instances (up to 90% savings).",
            "Consider auto-scaling groups to handle variable workloads.",
            "Compute-optimized families offer best price/performance for CPU-bound jobs.",
        ],
        WorkloadType::Storage => vec![
            "Storage-optimized instances include local NVMe SSDs for high throughput.",
            "Local storage is ephemeral — ensure data is replicated or backed up.",
            "Consider D3/I3 (AWS), L-series (Azure), or N2D+local SSD (GCP) for HDFS/data lakes.",
        ],
    }
}

// ─── GCP AI/ML (GPU instances) ──────────────────────────────────────────────

static GCP_AI: [InstanceType; 8] = [
    InstanceType { name: "a2-highgpu-1g", family: "A2", vcpus: 12.0, ram_gb: 85.0, tier: InstanceTier::Standard, cost_hour_usd: 3.6735 },
    InstanceType { name: "a2-highgpu-2g", family: "A2", vcpus: 24.0, ram_gb: 170.0, tier: InstanceTier::Standard, cost_hour_usd: 7.3470 },
    InstanceType { name: "a2-highgpu-4g", family: "A2", vcpus: 48.0, ram_gb: 340.0, tier: InstanceTier::Standard, cost_hour_usd: 14.6940 },
    InstanceType { name: "a2-highgpu-8g", family: "A2", vcpus: 96.0, ram_gb: 680.0, tier: InstanceTier::Standard, cost_hour_usd: 29.3878 },
    InstanceType { name: "g2-standard-4", family: "G2", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7211 },
    InstanceType { name: "g2-standard-8", family: "G2", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0227 },
    InstanceType { name: "g2-standard-12", family: "G2", vcpus: 12.0, ram_gb: 48.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3243 },
    InstanceType { name: "g2-standard-24", family: "G2", vcpus: 24.0, ram_gb: 96.0, tier: InstanceTier::Standard, cost_hour_usd: 2.2291 },
];

// ─── AWS AI/ML (GPU instances) ──────────────────────────────────────────────

static AWS_AI: [InstanceType; 8] = [
    InstanceType { name: "g5.xlarge", family: "G5", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0060 },
    InstanceType { name: "g5.2xlarge", family: "G5", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 1.2120 },
    InstanceType { name: "g5.4xlarge", family: "G5", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.6240 },
    InstanceType { name: "g5.8xlarge", family: "G5", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 2.4480 },
    InstanceType { name: "p4d.24xlarge", family: "P4d", vcpus: 96.0, ram_gb: 1152.0, tier: InstanceTier::Standard, cost_hour_usd: 32.7726 },
    InstanceType { name: "g4dn.xlarge", family: "G4dn", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5260 },
    InstanceType { name: "g4dn.2xlarge", family: "G4dn", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7520 },
    InstanceType { name: "g4dn.4xlarge", family: "G4dn", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.2040 },
];

// ─── Azure AI/ML (GPU instances) ────────────────────────────────────────────

static AZURE_AI: [InstanceType; 7] = [
    InstanceType { name: "Standard_NC4as_T4_v3", family: "NCasT4v3", vcpus: 4.0, ram_gb: 28.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5260 },
    InstanceType { name: "Standard_NC8as_T4_v3", family: "NCasT4v3", vcpus: 8.0, ram_gb: 56.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7520 },
    InstanceType { name: "Standard_NC16as_T4_v3", family: "NCasT4v3", vcpus: 16.0, ram_gb: 110.0, tier: InstanceTier::Standard, cost_hour_usd: 1.2040 },
    InstanceType { name: "Standard_NC64as_T4_v3", family: "NCasT4v3", vcpus: 64.0, ram_gb: 440.0, tier: InstanceTier::Standard, cost_hour_usd: 4.3520 },
    InstanceType { name: "Standard_NC24ads_A100_v4", family: "NCA100v4", vcpus: 24.0, ram_gb: 220.0, tier: InstanceTier::Standard, cost_hour_usd: 3.6730 },
    InstanceType { name: "Standard_NC48ads_A100_v4", family: "NCA100v4", vcpus: 48.0, ram_gb: 440.0, tier: InstanceTier::Standard, cost_hour_usd: 7.3460 },
    InstanceType { name: "Standard_NC96ads_A100_v4", family: "NCA100v4", vcpus: 96.0, ram_gb: 880.0, tier: InstanceTier::Standard, cost_hour_usd: 14.6920 },
];

// ─── GCP SAP ────────────────────────────────────────────────────────────────

static GCP_SAP: [InstanceType; 6] = [
    InstanceType { name: "m2-ultramem-208", family: "M2", vcpus: 208.0, ram_gb: 5888.0, tier: InstanceTier::Standard, cost_hour_usd: 42.1860 },
    InstanceType { name: "m2-ultramem-416", family: "M2", vcpus: 416.0, ram_gb: 11776.0, tier: InstanceTier::Standard, cost_hour_usd: 84.3720 },
    InstanceType { name: "m2-megamem-96", family: "M2", vcpus: 96.0, ram_gb: 1433.6, tier: InstanceTier::Standard, cost_hour_usd: 10.9940 },
    InstanceType { name: "m1-ultramem-40", family: "M1", vcpus: 40.0, ram_gb: 961.0, tier: InstanceTier::Standard, cost_hour_usd: 6.3039 },
    InstanceType { name: "m1-ultramem-80", family: "M1", vcpus: 80.0, ram_gb: 1922.0, tier: InstanceTier::Standard, cost_hour_usd: 12.6078 },
    InstanceType { name: "m1-ultramem-160", family: "M1", vcpus: 160.0, ram_gb: 3844.0, tier: InstanceTier::Standard, cost_hour_usd: 25.2156 },
];

// ─── AWS SAP ────────────────────────────────────────────────────────────────

static AWS_SAP: [InstanceType; 6] = [
    InstanceType { name: "x2idn.16xlarge", family: "X2idn", vcpus: 64.0, ram_gb: 1024.0, tier: InstanceTier::Standard, cost_hour_usd: 6.6690 },
    InstanceType { name: "x2idn.24xlarge", family: "X2idn", vcpus: 96.0, ram_gb: 1536.0, tier: InstanceTier::Standard, cost_hour_usd: 10.0035 },
    InstanceType { name: "x2idn.32xlarge", family: "X2idn", vcpus: 128.0, ram_gb: 2048.0, tier: InstanceTier::Standard, cost_hour_usd: 13.3380 },
    InstanceType { name: "u-6tb1.56xlarge", family: "U", vcpus: 224.0, ram_gb: 6144.0, tier: InstanceTier::Standard, cost_hour_usd: 36.5820 },
    InstanceType { name: "u-9tb1.112xlarge", family: "U", vcpus: 448.0, ram_gb: 9216.0, tier: InstanceTier::Standard, cost_hour_usd: 54.8730 },
    InstanceType { name: "u-12tb1.112xlarge", family: "U", vcpus: 448.0, ram_gb: 12288.0, tier: InstanceTier::Standard, cost_hour_usd: 73.1640 },
];

// ─── Azure SAP ──────────────────────────────────────────────────────────────

static AZURE_SAP: [InstanceType; 5] = [
    InstanceType { name: "Standard_M32ts", family: "Msv2", vcpus: 32.0, ram_gb: 192.0, tier: InstanceTier::Standard, cost_hour_usd: 3.0730 },
    InstanceType { name: "Standard_M64s", family: "Msv2", vcpus: 64.0, ram_gb: 1024.0, tier: InstanceTier::Standard, cost_hour_usd: 6.6690 },
    InstanceType { name: "Standard_M128s", family: "Msv2", vcpus: 128.0, ram_gb: 2048.0, tier: InstanceTier::Standard, cost_hour_usd: 13.3380 },
    InstanceType { name: "Standard_M208s_v2", family: "Msv2", vcpus: 208.0, ram_gb: 2850.0, tier: InstanceTier::Standard, cost_hour_usd: 22.0370 },
    InstanceType { name: "Standard_M416s_v2", family: "Msv2", vcpus: 416.0, ram_gb: 5700.0, tier: InstanceTier::Standard, cost_hour_usd: 44.0740 },
];

// ─── GCP HPC ────────────────────────────────────────────────────────────────

static GCP_HPC: [InstanceType; 5] = [
    InstanceType { name: "c2-standard-4", family: "C2", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2099 },
    InstanceType { name: "c2-standard-8", family: "C2", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.4198 },
    InstanceType { name: "c2-standard-16", family: "C2", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.8396 },
    InstanceType { name: "c2-standard-30", family: "C2", vcpus: 30.0, ram_gb: 120.0, tier: InstanceTier::Standard, cost_hour_usd: 1.5742 },
    InstanceType { name: "c2-standard-60", family: "C2", vcpus: 60.0, ram_gb: 240.0, tier: InstanceTier::Standard, cost_hour_usd: 3.1484 },
];

// ─── AWS HPC ────────────────────────────────────────────────────────────────

static AWS_HPC: [InstanceType; 5] = [
    InstanceType { name: "hpc6a.48xlarge", family: "Hpc6a", vcpus: 96.0, ram_gb: 384.0, tier: InstanceTier::Standard, cost_hour_usd: 2.8800 },
    InstanceType { name: "c6i.8xlarge", family: "C6i", vcpus: 32.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3600 },
    InstanceType { name: "c6i.16xlarge", family: "C6i", vcpus: 64.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 2.7200 },
    InstanceType { name: "c6i.24xlarge", family: "C6i", vcpus: 96.0, ram_gb: 192.0, tier: InstanceTier::Standard, cost_hour_usd: 4.0800 },
    InstanceType { name: "c5n.18xlarge", family: "C5n", vcpus: 72.0, ram_gb: 192.0, tier: InstanceTier::Standard, cost_hour_usd: 3.8880 },
];

// ─── Azure HPC ──────────────────────────────────────────────────────────────

static AZURE_HPC: [InstanceType; 4] = [
    InstanceType { name: "Standard_HB120rs_v3", family: "HBv3", vcpus: 120.0, ram_gb: 448.0, tier: InstanceTier::Standard, cost_hour_usd: 3.6000 },
    InstanceType { name: "Standard_HC44rs", family: "HC", vcpus: 44.0, ram_gb: 352.0, tier: InstanceTier::Standard, cost_hour_usd: 3.1680 },
    InstanceType { name: "Standard_HB120rs_v2", family: "HBv2", vcpus: 120.0, ram_gb: 480.0, tier: InstanceTier::Standard, cost_hour_usd: 3.6000 },
    InstanceType { name: "Standard_F72s_v2", family: "Fsv2", vcpus: 72.0, ram_gb: 144.0, tier: InstanceTier::Standard, cost_hour_usd: 3.0450 },
];

// ─── GCP Database ───────────────────────────────────────────────────────────

static GCP_DB: [InstanceType; 5] = [
    InstanceType { name: "n2-highmem-4", family: "N2", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2620 },
    InstanceType { name: "n2-highmem-8", family: "N2", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5240 },
    InstanceType { name: "n2-highmem-16", family: "N2", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0480 },
    InstanceType { name: "n2-highmem-32", family: "N2", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0960 },
    InstanceType { name: "n2-highmem-64", family: "N2", vcpus: 64.0, ram_gb: 512.0, tier: InstanceTier::Standard, cost_hour_usd: 4.1920 },
];

// ─── AWS Database ───────────────────────────────────────────────────────────

static AWS_DB: [InstanceType; 6] = [
    InstanceType { name: "r6i.large", family: "R6i", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1260 },
    InstanceType { name: "r6i.xlarge", family: "R6i", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2520 },
    InstanceType { name: "r6i.2xlarge", family: "R6i", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5040 },
    InstanceType { name: "r6i.4xlarge", family: "R6i", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0080 },
    InstanceType { name: "r6i.8xlarge", family: "R6i", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0160 },
    InstanceType { name: "r6i.16xlarge", family: "R6i", vcpus: 64.0, ram_gb: 512.0, tier: InstanceTier::Standard, cost_hour_usd: 4.0320 },
];

// ─── Azure Database ─────────────────────────────────────────────────────────

static AZURE_DB: [InstanceType; 5] = [
    InstanceType { name: "Standard_E4s_v5", family: "Esv5", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2520 },
    InstanceType { name: "Standard_E8s_v5", family: "Esv5", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5040 },
    InstanceType { name: "Standard_E16s_v5", family: "Esv5", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0080 },
    InstanceType { name: "Standard_E32s_v5", family: "Esv5", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0160 },
    InstanceType { name: "Standard_E64s_v5", family: "Esv5", vcpus: 64.0, ram_gb: 512.0, tier: InstanceTier::Standard, cost_hour_usd: 4.0320 },
];

// ─── GCP Batch/CI-CD ────────────────────────────────────────────────────────

static GCP_BATCH: [InstanceType; 5] = [
    InstanceType { name: "e2-standard-2", family: "E2", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0670 },
    InstanceType { name: "e2-standard-4", family: "E2", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1340 },
    InstanceType { name: "e2-standard-8", family: "E2", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.2681 },
    InstanceType { name: "c2d-standard-4", family: "C2D", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1848 },
    InstanceType { name: "c2d-standard-8", family: "C2D", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3696 },
];

// ─── AWS Batch/CI-CD ────────────────────────────────────────────────────────

static AWS_BATCH: [InstanceType; 5] = [
    InstanceType { name: "c6i.large", family: "C6i", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0850 },
    InstanceType { name: "c6i.xlarge", family: "C6i", vcpus: 4.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1700 },
    InstanceType { name: "c6i.2xlarge", family: "C6i", vcpus: 8.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3400 },
    InstanceType { name: "c6i.4xlarge", family: "C6i", vcpus: 16.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6800 },
    InstanceType { name: "c6i.8xlarge", family: "C6i", vcpus: 32.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3600 },
];

// ─── Azure Batch/CI-CD ──────────────────────────────────────────────────────

static AZURE_BATCH: [InstanceType; 5] = [
    InstanceType { name: "Standard_F2s_v2", family: "Fsv2", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0846 },
    InstanceType { name: "Standard_F4s_v2", family: "Fsv2", vcpus: 4.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1690 },
    InstanceType { name: "Standard_F8s_v2", family: "Fsv2", vcpus: 8.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3380 },
    InstanceType { name: "Standard_F16s_v2", family: "Fsv2", vcpus: 16.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6760 },
    InstanceType { name: "Standard_F32s_v2", family: "Fsv2", vcpus: 32.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3530 },
];

// ─── GCP Storage-intensive ──────────────────────────────────────────────────

static GCP_STORAGE: [InstanceType; 4] = [
    InstanceType { name: "n2d-standard-4", family: "N2D", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1690 },
    InstanceType { name: "n2d-standard-8", family: "N2D", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3380 },
    InstanceType { name: "n2d-standard-16", family: "N2D", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6760 },
    InstanceType { name: "n2d-standard-32", family: "N2D", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3520 },
];

// ─── AWS Storage-intensive ──────────────────────────────────────────────────

static AWS_STORAGE: [InstanceType; 5] = [
    InstanceType { name: "i3.large", family: "I3", vcpus: 2.0, ram_gb: 15.25, tier: InstanceTier::Standard, cost_hour_usd: 0.1560 },
    InstanceType { name: "i3.xlarge", family: "I3", vcpus: 4.0, ram_gb: 30.5, tier: InstanceTier::Standard, cost_hour_usd: 0.3120 },
    InstanceType { name: "i3.2xlarge", family: "I3", vcpus: 8.0, ram_gb: 61.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6240 },
    InstanceType { name: "i3.4xlarge", family: "I3", vcpus: 16.0, ram_gb: 122.0, tier: InstanceTier::Standard, cost_hour_usd: 1.2480 },
    InstanceType { name: "d3.xlarge", family: "D3", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.4990 },
];

// ─── Azure Storage-intensive ────────────────────────────────────────────────

static AZURE_STORAGE: [InstanceType; 4] = [
    InstanceType { name: "Standard_L8s_v2", family: "Lsv2", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6240 },
    InstanceType { name: "Standard_L16s_v2", family: "Lsv2", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.2480 },
    InstanceType { name: "Standard_L32s_v2", family: "Lsv2", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.4960 },
    InstanceType { name: "Standard_L48s_v2", family: "Lsv2", vcpus: 48.0, ram_gb: 384.0, tier: InstanceTier::Standard, cost_hour_usd: 3.7440 },
];
