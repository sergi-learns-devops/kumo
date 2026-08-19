use crate::models::{InstanceTier, InstanceType};
use crate::providers::ProviderCatalog;

pub struct AwsProvider;

impl ProviderCatalog for AwsProvider {
    fn catalog() -> &'static [InstanceType] {
        &AWS_INSTANCES
    }

    fn supports_custom() -> bool {
        false
    }

    fn valid_regions() -> &'static [&'static str] {
        &AWS_REGIONS
    }
}

pub static AWS_REGIONS: [&str; 33] = [
    "af-south-1",
    "ap-east-1",
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-northeast-3",
    "ap-south-1",
    "ap-south-2",
    "ap-southeast-1",
    "ap-southeast-2",
    "ap-southeast-3",
    "ap-southeast-4",
    "ca-central-1",
    "ca-west-1",
    "cn-north-1",
    "cn-northwest-1",
    "eu-central-1",
    "eu-central-2",
    "eu-north-1",
    "eu-south-1",
    "eu-south-2",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "il-central-1",
    "me-central-1",
    "me-south-1",
    "sa-east-1",
    "us-east-1",
    "us-east-2",
    "us-gov-east-1",
    "us-gov-west-1",
    "us-west-1",
    "us-west-2",
];

static AWS_INSTANCES: [InstanceType; 50] = [
    // T3 family (Economy tier) — us-east-1 pricing
    // t3.nano and t3.micro have fractional/burstable vCPUs
    InstanceType { name: "t3.nano", family: "T3", vcpus: 0.5, ram_gb: 0.5, tier: InstanceTier::Economy, cost_hour_usd: 0.0052 },
    InstanceType { name: "t3.micro", family: "T3", vcpus: 1.0, ram_gb: 1.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0104 },
    InstanceType { name: "t3.small", family: "T3", vcpus: 1.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0208 },
    InstanceType { name: "t3.medium", family: "T3", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0416 },
    InstanceType { name: "t3.large", family: "T3", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0832 },
    InstanceType { name: "t3.xlarge", family: "T3", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1664 },
    InstanceType { name: "t3.2xlarge", family: "T3", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.3328 },
    // T3a family (Economy tier)
    InstanceType { name: "t3a.nano", family: "T3a", vcpus: 0.5, ram_gb: 0.5, tier: InstanceTier::Economy, cost_hour_usd: 0.0047 },
    InstanceType { name: "t3a.micro", family: "T3a", vcpus: 1.0, ram_gb: 1.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0094 },
    InstanceType { name: "t3a.small", family: "T3a", vcpus: 1.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0188 },
    InstanceType { name: "t3a.medium", family: "T3a", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0376 },
    InstanceType { name: "t3a.large", family: "T3a", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0752 },
    InstanceType { name: "t3a.xlarge", family: "T3a", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1504 },
    InstanceType { name: "t3a.2xlarge", family: "T3a", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.3008 },
    // M5 family (Standard tier)
    InstanceType { name: "m5.large", family: "M5", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0960 },
    InstanceType { name: "m5.xlarge", family: "M5", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1920 },
    InstanceType { name: "m5.2xlarge", family: "M5", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3840 },
    InstanceType { name: "m5.4xlarge", family: "M5", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7680 },
    InstanceType { name: "m5.8xlarge", family: "M5", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.5360 },
    InstanceType { name: "m5.12xlarge", family: "M5", vcpus: 48.0, ram_gb: 192.0, tier: InstanceTier::Standard, cost_hour_usd: 2.3040 },
    // M6i family (Standard tier)
    InstanceType { name: "m6i.large", family: "M6i", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0960 },
    InstanceType { name: "m6i.xlarge", family: "M6i", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1920 },
    InstanceType { name: "m6i.2xlarge", family: "M6i", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3840 },
    InstanceType { name: "m6i.4xlarge", family: "M6i", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7680 },
    InstanceType { name: "m6i.8xlarge", family: "M6i", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.5360 },
    InstanceType { name: "m6i.12xlarge", family: "M6i", vcpus: 48.0, ram_gb: 192.0, tier: InstanceTier::Standard, cost_hour_usd: 2.3040 },
    // C5 family (Standard tier)
    InstanceType { name: "c5.large", family: "C5", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0850 },
    InstanceType { name: "c5.xlarge", family: "C5", vcpus: 4.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1700 },
    InstanceType { name: "c5.2xlarge", family: "C5", vcpus: 8.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3400 },
    InstanceType { name: "c5.4xlarge", family: "C5", vcpus: 16.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6800 },
    InstanceType { name: "c5.9xlarge", family: "C5", vcpus: 36.0, ram_gb: 72.0, tier: InstanceTier::Standard, cost_hour_usd: 1.5300 },
    // C6i family (Standard tier)
    InstanceType { name: "c6i.large", family: "C6i", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0850 },
    InstanceType { name: "c6i.xlarge", family: "C6i", vcpus: 4.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1700 },
    InstanceType { name: "c6i.2xlarge", family: "C6i", vcpus: 8.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3400 },
    InstanceType { name: "c6i.4xlarge", family: "C6i", vcpus: 16.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6800 },
    InstanceType { name: "c6i.8xlarge", family: "C6i", vcpus: 32.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3600 },
    // R5 family (Standard tier)
    InstanceType { name: "r5.large", family: "R5", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1260 },
    InstanceType { name: "r5.xlarge", family: "R5", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2520 },
    InstanceType { name: "r5.2xlarge", family: "R5", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5040 },
    InstanceType { name: "r5.4xlarge", family: "R5", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0080 },
    InstanceType { name: "r5.8xlarge", family: "R5", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0160 },
    InstanceType { name: "r5.12xlarge", family: "R5", vcpus: 48.0, ram_gb: 384.0, tier: InstanceTier::Standard, cost_hour_usd: 3.0240 },
    // R6i family (Standard tier)
    InstanceType { name: "r6i.large", family: "R6i", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1260 },
    InstanceType { name: "r6i.xlarge", family: "R6i", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2520 },
    InstanceType { name: "r6i.2xlarge", family: "R6i", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5040 },
    InstanceType { name: "r6i.4xlarge", family: "R6i", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0080 },
    InstanceType { name: "r6i.8xlarge", family: "R6i", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0160 },
    InstanceType { name: "r6i.12xlarge", family: "R6i", vcpus: 48.0, ram_gb: 384.0, tier: InstanceTier::Standard, cost_hour_usd: 3.0240 },
    InstanceType { name: "r6i.16xlarge", family: "R6i", vcpus: 64.0, ram_gb: 512.0, tier: InstanceTier::Standard, cost_hour_usd: 4.0320 },
    InstanceType { name: "r6i.24xlarge", family: "R6i", vcpus: 96.0, ram_gb: 768.0, tier: InstanceTier::Standard, cost_hour_usd: 6.0480 },
];
