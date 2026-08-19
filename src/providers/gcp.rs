use crate::models::{InstanceTier, InstanceType};
use crate::providers::ProviderCatalog;

pub struct GcpProvider;

impl ProviderCatalog for GcpProvider {
    fn catalog() -> &'static [InstanceType] {
        &GCP_INSTANCES
    }

    fn supports_custom() -> bool {
        true
    }

    fn valid_regions() -> &'static [&'static str] {
        &GCP_REGIONS
    }
}

pub static GCP_REGIONS: [&str; 40] = [
    "africa-south1",
    "asia-east1",
    "asia-east2",
    "asia-northeast1",
    "asia-northeast2",
    "asia-northeast3",
    "asia-south1",
    "asia-south2",
    "asia-southeast1",
    "asia-southeast2",
    "australia-southeast1",
    "australia-southeast2",
    "europe-central2",
    "europe-north1",
    "europe-southwest1",
    "europe-west1",
    "europe-west2",
    "europe-west3",
    "europe-west4",
    "europe-west6",
    "europe-west8",
    "europe-west9",
    "europe-west10",
    "europe-west12",
    "me-central1",
    "me-central2",
    "me-west1",
    "northamerica-northeast1",
    "northamerica-northeast2",
    "southamerica-east1",
    "southamerica-west1",
    "us-central1",
    "us-east1",
    "us-east4",
    "us-east5",
    "us-south1",
    "us-west1",
    "us-west2",
    "us-west3",
    "us-west4",
];

static GCP_INSTANCES: [InstanceType; 48] = [
    // E2 family (Economy tier) — us-central1 pricing
    // e2-micro and e2-small have shared/fractional vCPUs
    InstanceType { name: "e2-micro", family: "E2", vcpus: 0.25, ram_gb: 1.0, tier: InstanceTier::Economy, cost_hour_usd: 0.00838 },
    InstanceType { name: "e2-small", family: "E2", vcpus: 0.5, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.01675 },
    InstanceType { name: "e2-medium", family: "E2", vcpus: 1.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.03351 },
    InstanceType { name: "e2-standard-2", family: "E2", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.06701 },
    InstanceType { name: "e2-standard-4", family: "E2", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.13402 },
    InstanceType { name: "e2-standard-8", family: "E2", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.26805 },
    InstanceType { name: "e2-standard-16", family: "E2", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Economy, cost_hour_usd: 0.53609 },
    InstanceType { name: "e2-standard-32", family: "E2", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Economy, cost_hour_usd: 1.07218 },
    InstanceType { name: "e2-highmem-2", family: "E2", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.09040 },
    InstanceType { name: "e2-highmem-4", family: "E2", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.18080 },
    InstanceType { name: "e2-highmem-8", family: "E2", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Economy, cost_hour_usd: 0.36160 },
    InstanceType { name: "e2-highmem-16", family: "E2", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Economy, cost_hour_usd: 0.72320 },
    InstanceType { name: "e2-highcpu-2", family: "E2", vcpus: 2.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.04960 },
    InstanceType { name: "e2-highcpu-4", family: "E2", vcpus: 4.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.09920 },
    InstanceType { name: "e2-highcpu-8", family: "E2", vcpus: 8.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.19839 },
    InstanceType { name: "e2-highcpu-16", family: "E2", vcpus: 16.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.39678 },
    InstanceType { name: "e2-highcpu-32", family: "E2", vcpus: 32.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.79357 },
    // N2 family (Standard tier)
    InstanceType { name: "n2-standard-2", family: "N2", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.09710 },
    InstanceType { name: "n2-standard-4", family: "N2", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.19420 },
    InstanceType { name: "n2-standard-8", family: "N2", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.38840 },
    InstanceType { name: "n2-standard-16", family: "N2", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.77680 },
    InstanceType { name: "n2-standard-32", family: "N2", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.55360 },
    InstanceType { name: "n2-standard-48", family: "N2", vcpus: 48.0, ram_gb: 192.0, tier: InstanceTier::Standard, cost_hour_usd: 2.33040 },
    InstanceType { name: "n2-standard-64", family: "N2", vcpus: 64.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 3.10720 },
    InstanceType { name: "n2-highmem-2", family: "N2", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.13100 },
    InstanceType { name: "n2-highmem-4", family: "N2", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.26200 },
    InstanceType { name: "n2-highmem-8", family: "N2", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.52400 },
    InstanceType { name: "n2-highmem-16", family: "N2", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.04800 },
    InstanceType { name: "n2-highcpu-2", family: "N2", vcpus: 2.0, ram_gb: 2.0, tier: InstanceTier::Standard, cost_hour_usd: 0.07170 },
    InstanceType { name: "n2-highcpu-4", family: "N2", vcpus: 4.0, ram_gb: 4.0, tier: InstanceTier::Standard, cost_hour_usd: 0.14340 },
    InstanceType { name: "n2-highcpu-8", family: "N2", vcpus: 8.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.28680 },
    InstanceType { name: "n2-highcpu-16", family: "N2", vcpus: 16.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.57360 },
    InstanceType { name: "n2-highcpu-32", family: "N2", vcpus: 32.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 1.14720 },
    // N2D family (Standard tier)
    InstanceType { name: "n2d-standard-2", family: "N2D", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.08450 },
    InstanceType { name: "n2d-standard-4", family: "N2D", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.16900 },
    InstanceType { name: "n2d-standard-8", family: "N2D", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.33800 },
    InstanceType { name: "n2d-standard-16", family: "N2D", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.67600 },
    InstanceType { name: "n2d-standard-32", family: "N2D", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.35200 },
    // C2 family (Standard tier)
    InstanceType { name: "c2-standard-4", family: "C2", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.20990 },
    InstanceType { name: "c2-standard-8", family: "C2", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.41980 },
    InstanceType { name: "c2-standard-16", family: "C2", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.83960 },
    InstanceType { name: "c2-standard-30", family: "C2", vcpus: 30.0, ram_gb: 120.0, tier: InstanceTier::Standard, cost_hour_usd: 1.57420 },
    InstanceType { name: "c2-standard-60", family: "C2", vcpus: 60.0, ram_gb: 240.0, tier: InstanceTier::Standard, cost_hour_usd: 3.14840 },
    // C2D family (Standard tier)
    InstanceType { name: "c2d-standard-4", family: "C2D", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.18480 },
    InstanceType { name: "c2d-standard-8", family: "C2D", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.36960 },
    InstanceType { name: "c2d-standard-16", family: "C2D", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.73920 },
    InstanceType { name: "c2d-standard-32", family: "C2D", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.47840 },
    InstanceType { name: "c2d-standard-56", family: "C2D", vcpus: 56.0, ram_gb: 224.0, tier: InstanceTier::Standard, cost_hour_usd: 2.58720 },
];
