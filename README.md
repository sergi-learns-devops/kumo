# ☁ kumo (雲)

**Cloud instance type recommender for GCP, AWS & Azure.**

kumo helps you find the optimal cloud instance type based on your vCPU and RAM requirements, considering your target environment and budget constraints.

## Installation

### One-liner (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/sergi-learns-devops/kumo/main/install.sh | bash
```

Installs to `/usr/local/bin` when writable, otherwise falls back to `~/.local/bin`
(no `sudo` required). Set `KUMO_INSTALL_DIR` to choose a custom location. Installing
to a user-owned directory means `kumo --update` works without `sudo`.

### Homebrew (macOS/Linux)

```bash
brew tap sergi-learns-devops/kumo https://github.com/sergi-learns-devops/kumo
brew install kumo
```

### From source

```bash
git clone https://github.com/sergi-learns-devops/kumo.git
cd kumo
make install
```

### Manual download

Download the latest binary from [Releases](https://github.com/sergi-learns-devops/kumo/releases):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `kumo-macos-aarch64.tar.gz` |
| macOS (Intel) | `kumo-macos-x86_64.tar.gz` |
| Linux (x86_64) | `kumo-linux-x86_64.tar.gz` |
| Linux (ARM64) | `kumo-linux-aarch64.tar.gz` |

```bash
tar -xzf kumo-*.tar.gz
sudo mv kumo /usr/local/bin/
```

### Uninstall

```bash
make uninstall
# or manually:
rm -f /usr/local/bin/kumo
rm -rf ~/.kumo
```

## Updating

### Self-update (recommended)

```bash
kumo --update
```

Checks GitHub for the latest release and updates the binary in place.

### One-liner (same as install)

```bash
curl -fsSL https://raw.githubusercontent.com/sergi-learns-devops/kumo/main/install.sh | bash
```

### Homebrew

```bash
brew upgrade kumo
```

### From source

```bash
cd kumo
git pull
make install
```

### Verify the update

```bash
kumo --version
```

## Usage

### Interactive mode (default)

```bash
kumo
```

Launches an interactive wizard that guides you through selecting provider, environment, workload type, region, and resources.

### Flags mode

```bash
kumo --cloud gcp --vcpu 8 --ram 32 --env production --region us-central1
kumo --cloud aws --vcpu 4 --ram 16 --env dev --region eu-west-1
kumo --cloud azure --vcpu 16 --ram 64 --env prod --region eastus
kumo -c gcp -v 0.5 -r 2 -e dev --region us-central1
kumo -c aws -v 4 -r 16 -e prod --region us-east-1 --family M6i
kumo -c aws -v 8 -r 32 -e prod --region us-east-1 -w ai
```

### Options

| Flag | Short | Description | Values |
|------|-------|-------------|--------|
| `--cloud` | `-c` | Cloud provider | `gcp`, `aws`, `azure`, `all` |
| `--vcpu` | `-v` | Number of vCPUs (supports decimals) | 0.25-512 |
| `--ram` | `-r` | RAM in GB | 0.5-12288 |
| `--env` | `-e` | Target environment | `production`, `pre-production`, `test`, `dev` |
| `--region` | | Cloud region (comma-separated for multi-region compare) | `us-central1`, `eu-west-1,eu-west-2`, etc. |
| `--family` | `-f` | Filter by instance family | `N2`, `M5`, `Dsv5`, etc. |
| `--workload` | `-w` | Workload type | `general`, `ai`, `sap`, `hpc`, `database`, `batch`, `storage` |
| `--service` | | Service type | `vm` (default), `db` |
| `--engine` | | Database engine (requires `--service db`) | `postgres`, `mysql`, `sqlserver` |
| `--gpu-type` | | GPU model (for AI workloads) | `T4`, `A10G`, `L4`, `A100`, `H100` |
| `--gpu-count` | | Minimum number of GPUs | 1-8 |
| `--spot` | | Show Spot/Preemptible pricing alongside on-demand | |
| `--explain` | | Show the scoring breakdown per recommendation | |
| `--min-network` | | Minimum network bandwidth in Gbps (adds network to scoring) | e.g. `10` |
| `--list-regions` | | List valid regions for a provider | `gcp`, `aws`, `azure` |
| `--update-catalog` | | Check for pricing updates | |
| `--update` | | Update kumo to the latest version | |
| `--completions` | | Generate shell completions | `bash`, `zsh`, `fish` |

### Workload types

kumo recommends specialized instance families based on your workload:

| Workload | Flag value | Aliases | Instance families |
|----------|-----------|---------|-------------------|
| **General purpose** | `general` | `gp`, `standard` | N2, M5, Dsv5 (balanced) |
| **AI / Machine Learning** | `ai` | `ml`, `gpu` | A2, G2 (GCP), G5, P4d (AWS), NC (Azure) |
| **SAP** | `sap` | `hana` | M2, M1 (GCP), X2idn, U (AWS), Msv2 (Azure) |
| **HPC** | `hpc` | `compute` | C2 (GCP), Hpc6a, C6i (AWS), HBv3 (Azure) |
| **Databases** | `database` | `db`, `memory` | N2-highmem (GCP), R6i (AWS), Esv5 (Azure) |
| **Batch / CI-CD** | `batch` | `ci`, `cicd` | E2, C2D (GCP), C6i (AWS), Fsv2 (Azure) |
| **Storage-intensive** | `storage` | `io`, `disk` | N2D (GCP), I3, D3 (AWS), Lsv2 (Azure) |

```bash
# AI workload — recommends GPU instances
kumo -c aws -v 8 -r 32 -e prod --region us-east-1 -w ai

# SAP workload — recommends certified high-memory instances
kumo -c gcp -v 96 -r 1400 -e prod --region europe-west1 -w sap

# HPC workload — recommends compute-optimized instances
kumo -c azure -v 64 -r 128 -e prod --region eastus -w hpc

# Database workload — recommends memory-optimized instances
kumo -c aws -v 8 -r 64 -e prod --region eu-west-1 -w database
```

### GPU filter (AI workloads)

Filter by GPU model and minimum count for AI/ML workloads:

```bash
# All GPU instances
kumo -c aws -v 8 -r 32 -e prod --region us-east-1 -w ai

# Filter by T4 GPU
kumo -c aws -v 4 -r 16 -e prod --region us-east-1 -w ai --gpu-type T4

# Filter by A100 with at least 2 GPUs
kumo -c gcp -v 24 -r 170 -e prod --region us-central1 -w ai --gpu-type A100 --gpu-count 2
```

Available GPUs by provider:

| Provider | GPUs |
|----------|------|
| GCP | L4 (24 GB), A100 (40 GB) |
| AWS | T4 (16 GB), A10G (24 GB), A100 (40 GB) |
| Azure | T4 (16 GB), A100 (80 GB) |

### Managed Database instances

Recommend instance classes for Cloud SQL, RDS, and Azure Database:

```bash
# RDS PostgreSQL
kumo -c aws -v 4 -r 16 -e prod --region us-east-1 --service db --engine postgres

# Cloud SQL MySQL
kumo -c gcp -v 2 -r 8 -e dev --region us-central1 --service db --engine mysql

# Azure Database SQL Server
kumo -c azure -v 8 -r 32 -e prod --region eastus --service db --engine sqlserver
```

### Multi-cloud compare

Compare the best instance across all providers at once:

```bash
kumo --cloud all -v 4 -r 16 -e prod
kumo --cloud all -v 8 -r 32 -e prod -w ai
```

### Multi-region compare

Pass a comma-separated list of regions to compare pricing across them (the cheapest is marked):

```bash
kumo -c aws -v 4 -r 16 -e prod --region us-east-1,eu-west-1,ap-south-1
kumo -c gcp -v 8 -r 32 -e prod --region us-central1,europe-west1
```

### Spot / Preemptible pricing

Add `--spot` to see interruptible pricing alongside on-demand (ideal for batch/dev):

```bash
kumo -c aws -v 8 -r 32 -e dev --region us-east-1 --spot
```

Approximate discounts: GCP/AWS ~70%, Azure ~65%.

### Explain the scoring

Use `--explain` to see the per-component breakdown (CPU, RAM, cost, network) of each recommendation's score:

```bash
kumo -c gcp -v 4 -r 16 -e prod --region us-central1 --explain
```

### Network bandwidth

Filter by minimum network bandwidth in Gbps. When set, network becomes a scoring factor (weights shift to CPU 30% / RAM 30% / cost 25% / network 15%):

```bash
kumo -c aws -v 16 -r 64 -e prod --region us-east-1 --min-network 10
```

### Config file

Save your defaults in `~/.kumo/config.toml` to avoid repeating flags:

```toml
[defaults]
cloud = "gcp"
region = "europe-west1"
env = "dev"
# workload = "general"
# service = "vm"
```

Then just run:

```bash
kumo -v 4 -r 16
```

### Shell completions

```bash
# Zsh
kumo --completions zsh >> ~/.zshrc

# Bash
kumo --completions bash >> ~/.bashrc

# Fish
kumo --completions fish > ~/.config/fish/completions/kumo.fish
```

### List available regions

```bash
kumo --list-regions gcp
kumo --list-regions aws
kumo --list-regions azure
```

### Filter by instance family

```bash
kumo -c aws -v 8 -r 32 -e prod --region us-east-1 --family M6i
kumo -c gcp -v 4 -r 16 -e prod --region us-central1 --family N2
kumo -c azure -v 8 -r 32 -e prod --region eastus --family Dsv5
```

### Fractional vCPUs

Some instance types have fractional/shared vCPUs (e.g., e2-micro = 0.25 vCPU, t3.nano = 0.5 vCPU):

```bash
kumo -c gcp -v 0.5 -r 2 -e dev --region us-central1
kumo -c aws -v 0.5 -r 0.5 -e dev --region us-east-1
```

### Environment aliases

| Environment | Accepted values |
|-------------|----------------|
| Production | `production`, `prod` |
| Pre-Production | `pre-production`, `preprod`, `pre`, `staging` |
| Test | `test`, `testing` |
| Dev | `dev`, `development` |

### Disable colors

```bash
NO_COLOR=1 kumo --cloud aws --vcpu 4 --ram 16 --env prod --region us-east-1
```

## How it works

1. Selects instance catalog based on cloud provider and workload type
2. Filters by tier (Standard for prod/pre-prod/test, Economy for dev)
3. Optionally filters by instance family and/or GPU type
4. Scores instances by proximity to requirements and cost efficiency
5. Returns top 3 recommendations + custom machine option (GCP only)

## Scoring algorithm

```
score = 1.0 - (0.35 × |Δcpu|/req_cpu + 0.35 × |Δram|/req_ram + 0.30 × normalized_cost)
```

- 35% weight — CPU proximity
- 35% weight — RAM proximity
- 30% weight — Cost efficiency

## Supported instance families

### GCP
E2 (economy), N2, N2D, C2, C2D (standard)

### AWS
T3, T3a (economy), M5, M6i, C5, C6i, R5, R6i (standard)

### Azure
B, Av2 (economy), Dsv5, Dsv4, Esv5, Fsv2, Lsv2, NCasT4v3 (standard)

## Roadmap

### v0.3 (next)

| # | Feature | Status |
|---|---------|--------|
| 1 | Committed Use Pricing (CUD/Reserved/Savings Plans) — show savings vs on-demand | ✅ Done |
| 2 | Multi-cloud compare (`--cloud all`) — unified ranking across GCP, AWS & Azure | ✅ Done |
| 3 | Config file (`~/.kumo/config.toml`) — persist defaults (cloud, region, env) | ✅ Done |
| 4 | Table side-by-side — aligned columns for easier visual comparison | ✅ Done |
| 5 | Shell completions (`--completions bash\|zsh\|fish`) | ✅ Done |

### v1.0 (stable)

| # | Feature | Status |
|---|---------|--------|
| 6 | Output structured (`--output json\|csv`) | Planned |
| 7 | Spot/Preemptible pricing (`--spot`) | ✅ Done |
| 8 | `--explain` mode — scoring breakdown per recommendation | ✅ Done |
| 9 | GPU filter (`--gpu-type T4\|A100\|H100`) for AI workloads | ✅ Done |
| 10 | Terraform/Pulumi snippet (`--emit terraform`) | Planned |
| 11 | Budget constraint (`--max-cost`) | Planned |

### v2.0 (future)

| # | Feature | Status |
|---|---------|--------|
| 12 | Kubernetes node sizing (`--k8s`) | Planned |
| 13 | Multi-region compare (`--region us-east-1,eu-west-1`) | ✅ Done |
| 14 | Network bandwidth scoring (`--min-network`) | ✅ Done |
| 15 | Compact mode (auto-detect TTY for pipelines) | Planned |
| 16 | Plugin system (WASM) for custom providers | Planned |

## Pricing disclaimer

Prices are embedded estimates based on official pricing pages (last updated: 2024-11-01). Actual costs may vary by region, applied discounts, and current pricing. Use `--update-catalog` to check for updates.

## Dependencies

- `clap` — CLI argument parsing
- `dialoguer` — Interactive terminal prompts

## License

MIT
