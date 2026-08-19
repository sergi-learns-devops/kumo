# ☁ kumo (雲)

**Cloud instance type recommender for GCP, AWS & Azure.**

kumo helps you find the optimal cloud instance type based on your vCPU and RAM requirements, considering your target environment and budget constraints.

## Installation

### One-liner (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/sergiserra/kumo/main/install.sh | bash
```

### Homebrew (macOS/Linux)

```bash
brew tap sergiserra/kumo https://github.com/sergiserra/kumo
brew install kumo
```

### From source

```bash
git clone https://github.com/sergiserra/kumo.git
cd kumo
make install
```

### Manual download

Download the latest binary from [Releases](https://github.com/sergiserra/kumo/releases):

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

## Usage

### Interactive mode (default)

```bash
kumo
```

Launches an interactive wizard that guides you through selecting provider, resources, and environment.

### Flags mode

```bash
kumo --cloud gcp --vcpu 8 --ram 32 --env production --region us-central1
kumo --cloud aws --vcpu 4 --ram 16 --env dev --region eu-west-1
kumo --cloud azure --vcpu 16 --ram 64 --env prod --region eastus
kumo -c gcp -v 0.5 -r 2 -e dev --region us-central1
kumo -c aws -v 4 -r 16 -e prod --region us-east-1 --family M6i
```

### Options

| Flag | Short | Description | Values |
|------|-------|-------------|--------|
| `--cloud` | `-c` | Cloud provider | `gcp`, `aws`, `azure` |
| `--vcpu` | `-v` | Number of vCPUs (supports decimals) | 0.25-512 |
| `--ram` | `-r` | RAM in GB | 0.5-12288 |
| `--env` | `-e` | Target environment | `production`, `pre-production`, `test`, `dev` |
| `--region` | | Cloud region | `us-central1`, `eu-west-1`, `eastus`, etc. |
| `--family` | `-f` | Filter by instance family | `N2`, `M5`, `Dsv5`, etc. |
| `--list-regions` | | List valid regions for a provider | `gcp`, `aws`, `azure` |
| `--update-catalog` | | Check for pricing updates | |

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

1. Selects instance catalog based on cloud provider
2. Filters by tier (Standard for prod/pre-prod, Economy for test/dev)
3. Optionally filters by instance family
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

## Pricing disclaimer

Prices are embedded estimates based on official pricing pages (last updated: 2024-11-01). Actual costs may vary by region, applied discounts, and current pricing. Use `--update-catalog` to check for updates.

## Dependencies

- `clap` — CLI argument parsing
- `dialoguer` — Interactive terminal prompts

## License

MIT
