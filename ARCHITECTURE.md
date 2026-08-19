# Architecture — kumo (雲)

## Overview

kumo is a single-binary CLI tool written in Rust that recommends cloud instance types
based on user requirements. It supports both GCP and AWS with real pricing data embedded
in the binary. Zero network calls, works 100% offline.

## Module Structure

```
src/
├── main.rs          → Entry point, mode selection (flags vs interactive)
├── cli.rs           → CLI argument definitions using clap derive
├── interactive.rs   → Interactive wizard using dialoguer
├── models.rs        → Core data types (providers, instances, recommendations)
├── recommend.rs     → Recommendation engine with scoring algorithm
├── output.rs        → Terminal output with ANSI colors
└── providers/
    ├── mod.rs       → ProviderCatalog trait definition
    ├── gcp.rs       → GCP instance catalog (45 instances)
    └── aws.rs       → AWS instance catalog (48 instances)
```

## Data Flow

```
User Input (CLI flags or interactive prompts)
        │
        ▼
    UserRequest { cloud, vcpus, ram_gb, env }
        │
        ▼
    recommend::recommend(&request)
        │
        ├─ Select catalog (GCP/AWS static arrays)
        ├─ Filter by tier (Standard/Economy based on env)
        ├─ Find exact match (vcpus + ram)
        ├─ Score all instances (weighted proximity)
        ├─ Take top 3 by score
        ├─ Generate custom machine (GCP only)
        └─ Build contextual tips
        │
        ▼
    Recommendation { exact_match, closest_matches, custom_option, notes }
        │
        ▼
    output::display() → formatted terminal output
```

## Scoring Algorithm

```
score = 1.0 - (0.35 × |Δcpu|/req_cpu + 0.35 × |Δram|/req_ram + 0.30 × normalized_cost)
```

Where:
- Δcpu = instance_vcpus - requested_vcpus
- Δram = instance_ram - requested_ram
- normalized_cost = instance_cost / max_cost_in_filtered_set

## Tier Selection by Environment

| Environment    | Tier     | Families                    |
|---------------|----------|-----------------------------|
| Production    | Standard | N2, N2D, C2, M5, M6i, C5   |
| Pre-Production| Standard | Same as prod                |
| Test          | Economy  | E2, T3, T3a                 |
| Dev           | Economy  | Same as test                |

## Custom Machines (GCP only)

GCP supports custom machine types with arbitrary vCPU/RAM:
- Name format: `n2-custom-{vcpus}-{ram_mb}`
- RAM rounded to nearest 256 MB (GCP requirement)
- Pricing formula: `vcpus × $0.03398/hr + ram_gb × $0.00455/hr`

## Design Decisions

1. **Static catalogs** — No network calls, instant results, works offline
2. **Zero-copy references** — `&'static` for instance data, minimal heap allocation
3. **Two dependencies** — clap (CLI) + dialoguer (interactive), nothing else
4. **Direct ANSI codes** — No color library, respects NO_COLOR env var
5. **Release profile** — LTO + opt-level z + strip + panic=abort → small binary

## Extensibility

To add a new provider (e.g., Azure):
1. Create `src/providers/azure.rs` with static catalog
2. Implement `ProviderCatalog` trait
3. Add `Azure` variant to `CloudProvider` enum
4. Register in `recommend.rs` match arm
5. Add to interactive prompt options

## Compilation

```bash
cargo build --release
# Binary: target/release/kumo
# Expected size: 1.5-2.5 MB (macOS arm64)
```
