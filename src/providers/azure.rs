use crate::models::{InstanceTier, InstanceType};
use crate::providers::ProviderCatalog;

pub struct AzureProvider;

impl ProviderCatalog for AzureProvider {
    fn catalog() -> &'static [InstanceType] {
        &AZURE_INSTANCES
    }

    fn supports_custom() -> bool {
        false
    }

    fn valid_regions() -> &'static [&'static str] {
        &AZURE_REGIONS
    }
}

pub static AZURE_REGIONS: [&str; 30] = [
    "australiaeast",
    "australiasoutheast",
    "brazilsouth",
    "canadacentral",
    "canadaeast",
    "centralindia",
    "centralus",
    "eastasia",
    "eastus",
    "eastus2",
    "francecentral",
    "germanywestcentral",
    "japaneast",
    "japanwest",
    "koreacentral",
    "northcentralus",
    "northeurope",
    "norwayeast",
    "southafricanorth",
    "southcentralus",
    "southeastasia",
    "swedencentral",
    "switzerlandnorth",
    "uaenorth",
    "uksouth",
    "ukwest",
    "westcentralus",
    "westeurope",
    "westus",
    "westus2",
];

static AZURE_INSTANCES: [InstanceType; 44] = [
    // B-series (Economy/Burstable tier) — East US pricing
    InstanceType { name: "Standard_B1s", family: "B", vcpus: 1.0, ram_gb: 1.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0104 },
    InstanceType { name: "Standard_B1ms", family: "B", vcpus: 1.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0207 },
    InstanceType { name: "Standard_B2s", family: "B", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0416 },
    InstanceType { name: "Standard_B2ms", family: "B", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0832 },
    InstanceType { name: "Standard_B4ms", family: "B", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1660 },
    InstanceType { name: "Standard_B8ms", family: "B", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Economy, cost_hour_usd: 0.3320 },
    InstanceType { name: "Standard_B16ms", family: "B", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Economy, cost_hour_usd: 0.6650 },
    InstanceType { name: "Standard_B20ms", family: "B", vcpus: 20.0, ram_gb: 80.0, tier: InstanceTier::Economy, cost_hour_usd: 0.8320 },
    // A-series v2 (Economy tier)
    InstanceType { name: "Standard_A1_v2", family: "Av2", vcpus: 1.0, ram_gb: 2.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0430 },
    InstanceType { name: "Standard_A2_v2", family: "Av2", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Economy, cost_hour_usd: 0.0910 },
    InstanceType { name: "Standard_A4_v2", family: "Av2", vcpus: 4.0, ram_gb: 8.0, tier: InstanceTier::Economy, cost_hour_usd: 0.1910 },
    InstanceType { name: "Standard_A8_v2", family: "Av2", vcpus: 8.0, ram_gb: 16.0, tier: InstanceTier::Economy, cost_hour_usd: 0.4000 },
    // D-series v5 (Standard/General purpose)
    InstanceType { name: "Standard_D2s_v5", family: "Dsv5", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0960 },
    InstanceType { name: "Standard_D4s_v5", family: "Dsv5", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1920 },
    InstanceType { name: "Standard_D8s_v5", family: "Dsv5", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3840 },
    InstanceType { name: "Standard_D16s_v5", family: "Dsv5", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7680 },
    InstanceType { name: "Standard_D32s_v5", family: "Dsv5", vcpus: 32.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.5360 },
    InstanceType { name: "Standard_D48s_v5", family: "Dsv5", vcpus: 48.0, ram_gb: 192.0, tier: InstanceTier::Standard, cost_hour_usd: 2.3040 },
    InstanceType { name: "Standard_D64s_v5", family: "Dsv5", vcpus: 64.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 3.0720 },
    // D-series v4 (Standard)
    InstanceType { name: "Standard_D2s_v4", family: "Dsv4", vcpus: 2.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0960 },
    InstanceType { name: "Standard_D4s_v4", family: "Dsv4", vcpus: 4.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1920 },
    InstanceType { name: "Standard_D8s_v4", family: "Dsv4", vcpus: 8.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3840 },
    InstanceType { name: "Standard_D16s_v4", family: "Dsv4", vcpus: 16.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7680 },
    // E-series v5 (Memory optimized — Standard tier)
    InstanceType { name: "Standard_E2s_v5", family: "Esv5", vcpus: 2.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1260 },
    InstanceType { name: "Standard_E4s_v5", family: "Esv5", vcpus: 4.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.2520 },
    InstanceType { name: "Standard_E8s_v5", family: "Esv5", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5040 },
    InstanceType { name: "Standard_E16s_v5", family: "Esv5", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.0080 },
    InstanceType { name: "Standard_E32s_v5", family: "Esv5", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0160 },
    InstanceType { name: "Standard_E48s_v5", family: "Esv5", vcpus: 48.0, ram_gb: 384.0, tier: InstanceTier::Standard, cost_hour_usd: 3.0240 },
    InstanceType { name: "Standard_E64s_v5", family: "Esv5", vcpus: 64.0, ram_gb: 512.0, tier: InstanceTier::Standard, cost_hour_usd: 4.0320 },
    // F-series v2 (Compute optimized — Standard tier)
    InstanceType { name: "Standard_F2s_v2", family: "Fsv2", vcpus: 2.0, ram_gb: 4.0, tier: InstanceTier::Standard, cost_hour_usd: 0.0846 },
    InstanceType { name: "Standard_F4s_v2", family: "Fsv2", vcpus: 4.0, ram_gb: 8.0, tier: InstanceTier::Standard, cost_hour_usd: 0.1690 },
    InstanceType { name: "Standard_F8s_v2", family: "Fsv2", vcpus: 8.0, ram_gb: 16.0, tier: InstanceTier::Standard, cost_hour_usd: 0.3380 },
    InstanceType { name: "Standard_F16s_v2", family: "Fsv2", vcpus: 16.0, ram_gb: 32.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6760 },
    InstanceType { name: "Standard_F32s_v2", family: "Fsv2", vcpus: 32.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 1.3530 },
    InstanceType { name: "Standard_F48s_v2", family: "Fsv2", vcpus: 48.0, ram_gb: 96.0, tier: InstanceTier::Standard, cost_hour_usd: 2.0240 },
    InstanceType { name: "Standard_F64s_v2", family: "Fsv2", vcpus: 64.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 2.7040 },
    // L-series v2 (Storage optimized — Standard tier)
    InstanceType { name: "Standard_L8s_v2", family: "Lsv2", vcpus: 8.0, ram_gb: 64.0, tier: InstanceTier::Standard, cost_hour_usd: 0.6240 },
    InstanceType { name: "Standard_L16s_v2", family: "Lsv2", vcpus: 16.0, ram_gb: 128.0, tier: InstanceTier::Standard, cost_hour_usd: 1.2480 },
    InstanceType { name: "Standard_L32s_v2", family: "Lsv2", vcpus: 32.0, ram_gb: 256.0, tier: InstanceTier::Standard, cost_hour_usd: 2.4960 },
    // NC-series (GPU — Standard tier)
    InstanceType { name: "Standard_NC4as_T4_v3", family: "NCasT4v3", vcpus: 4.0, ram_gb: 28.0, tier: InstanceTier::Standard, cost_hour_usd: 0.5260 },
    InstanceType { name: "Standard_NC8as_T4_v3", family: "NCasT4v3", vcpus: 8.0, ram_gb: 56.0, tier: InstanceTier::Standard, cost_hour_usd: 0.7520 },
    InstanceType { name: "Standard_NC16as_T4_v3", family: "NCasT4v3", vcpus: 16.0, ram_gb: 110.0, tier: InstanceTier::Standard, cost_hour_usd: 1.2040 },
    InstanceType { name: "Standard_NC64as_T4_v3", family: "NCasT4v3", vcpus: 64.0, ram_gb: 440.0, tier: InstanceTier::Standard, cost_hour_usd: 4.3520 },
];
