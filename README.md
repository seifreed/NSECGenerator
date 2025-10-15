# NSEC3 Hash Generator

[![CI](https://github.com/seifreed/NSECGenerator/workflows/CI/badge.svg)](https://github.com/seifreed/NSECGenerator/actions/workflows/ci.yml)
[![Release](https://github.com/seifreed/NSECGenerator/workflows/Build%20and%20Release/badge.svg)](https://github.com/seifreed/NSECGenerator/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

High-performance Rust-based NSEC3 hash pre-calculator for DNS zone walking with [DNSight](https://github.com/seifreed/DNSight).

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Integration with DNSight](#integration-with-dnsight)
- [Salt Management Strategies](#salt-management-strategies)
- [Performance](#performance)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [Resources](#resources)

---

## Overview

This tool pre-computes NSEC3 hashes for subdomain wordlists, creating cache files that tools like DNSight can use for instant NSEC3 hash cracking during zone walking. By pre-calculating hashes, you avoid re-computing them every time you run a zone walk, resulting in speedups of 100-200x.

### Why Pre-compute Hashes?

NSEC3 uses SHA1 with iterations to hash **fully-qualified domain names (FQDNs)**, not just subdomains:

```
Hash = SHA1(subdomain.domain.com + salt) with N iterations
```

**Example:**
```
www.example.com → xbsm3ndrdp3wkjeizjov22ihyipiu46c
www.google.com  → 3c4z62fsbc2ukozzdsymnq6wvgbe6pb2
```

Hashes are completely different even if the subdomain is the same. Therefore:
- **You cannot have a universal dictionary** of subdomains
- **You must generate hashes for each specific domain**
- **Pre-computing saves massive amounts of time** during active reconnaissance

---

## Features

- **Ultra-fast**: Parallel processing with Rayon (multi-threaded)
- **Smart caching**: Compatible with DNSight's cache format
- **Flexible**: Supports custom salt and iterations
- **Progress tracking**: Real-time progress bar
- **Optimized**: Release build with LTO for maximum speed
- **Batch processing**: Generate multiple configurations at once

---

## Quick Start

### One-Command Installation

```bash
cd NSECGenerator
cargo build --release
./target/release/nsec3-generator download-wordlists
```

This will:
1. Build the optimized binary
2. Download sample wordlists (1K, 10K, 100K)

### Manual Quick Start

```bash
# 1. Build the release binary
cargo build --release

# 2. Download wordlists
./target/release/nsec3-generator download-wordlists

# 3. Run an example
./target/release/nsec3-generator \
    --domain example.com \
    --wordlist wordlists/subdomains-10k.txt \
    --salt "" \
    --iterations 0

# 4. Deploy to DNSight
cp output/*.json ../data/nsec3_cache/
```

---

## Installation

### Option 1: Download Pre-compiled Binary (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/seifreed/NSECGenerator/releases):

**Supported platforms:**
- Linux x86_64
- Linux ARM64 (aarch64)
- Windows x86_64
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)

**Linux/macOS:**
```bash
# Download for your platform
wget https://github.com/seifreed/NSECGenerator/releases/latest/download/nsec3-generator-Linux-x86_64.tar.gz

# Extract
tar xzf nsec3-generator-*.tar.gz

# Make executable
chmod +x nsec3-generator

# Run
./nsec3-generator --help
```

**Windows:**
```powershell
# Download and extract the .zip file
# Then run:
.\nsec3-generator.exe --help
```

### Option 2: Build from Source

**Prerequisites:**
- Rust 1.70+ ([Install Rust](https://rustup.rs/))

```bash
# Check Rust installation
rustc --version
cargo --version
```

**Build:**
```bash
# Clone or navigate to NSECGenerator directory
cd NSECGenerator

# Build optimized release binary
cargo build --release

# Binary will be at: target/release/nsec3-generator
```

### Using Makefile

```bash
# Build optimized binary
make release

# Download sample wordlists
make wordlists

# Install to system (optional)
make install
```

---

## Usage

### Basic Usage

```bash
./target/release/nsec3-generator \
    --domain example.com \
    --wordlist wordlists/subdomains-10k.txt \
    --salt "" \
    --iterations 0
```

### Advanced Usage

```bash
# With custom salt and iterations
./target/release/nsec3-generator \
    --domain example.com \
    --wordlist wordlists/subdomains-100k.txt \
    --salt "AABBCCDD" \
    --iterations 10 \
    --output output \
    --threads 16
```

### CLI Commands

#### Generate Hashes (Default)

```bash
./target/release/nsec3-generator \
    --domain <DOMAIN> \
    --wordlist <WORDLIST> \
    [OPTIONS]
```

**Options:**
```
  -d, --domain <DOMAIN>          Domain to generate hashes for
  -w, --wordlist <WORDLIST>      Path to wordlist file (one subdomain per line)
  -s, --salt <SALT>              NSEC3 salt (hex string) [default: ""]
  -i, --iterations <ITERATIONS>  Number of NSEC3 iterations [default: 0]
  -o, --output <OUTPUT>          Output directory for JSON cache files [default: output]
  -t, --threads <THREADS>        Number of threads (default: CPU cores)
  -h, --help                     Print help
  -V, --version                  Print version
```

#### Download Wordlists

```bash
./target/release/nsec3-generator download-wordlists [OPTIONS]
```

**Options:**
```
  -o, --output <OUTPUT>  Output directory for wordlists [default: wordlists]
  -s, --size <SIZE>      Size: 1k, 10k, 100k, or all [default: all]
  -h, --help             Print help
```

**Examples:**
```bash
# Download all wordlists
./target/release/nsec3-generator download-wordlists

# Download only 10K wordlist
./target/release/nsec3-generator download-wordlists --size 10k

# Download to custom directory
./target/release/nsec3-generator download-wordlists --output /tmp/wordlists
```

#### Generate Common Configurations

```bash
./target/release/nsec3-generator generate-common [OPTIONS]
```

Generates hashes for 8 common NSEC3 configurations:
1. No salt, no iterations (30% of NSEC3 domains)
2. Google Cloud DNS (DEADBEEF, 5 iterations)
3. AWS Route53 (CAFEBABE, 10 iterations)
4. Cloudflare minimal (00, 0 iterations)
5. Light security (AABBCCDD, 3 iterations)
6. Medium security (12345678, 5 iterations)
7. High security (FEDCBA98, 10 iterations)
8. Very high security (FFFFFFFF, 15 iterations)

**Options:**
```
  -d, --domain <DOMAIN>      Domain to generate hashes for
  -w, --wordlist <WORDLIST>  Path to wordlist file
  -o, --output <OUTPUT>      Output directory [default: output]
  -t, --threads <THREADS>    Number of threads (default: CPU cores)
  -h, --help                 Print help
```

**Examples:**
```bash
# Generate common configs for target.com
./target/release/nsec3-generator generate-common \
    --domain target.com \
    --wordlist wordlists/subdomains-100k.txt

# With custom output directory
./target/release/nsec3-generator generate-common \
    --domain target.com \
    --wordlist wordlists/subdomains-10k.txt \
    --output /tmp/caches
```

### Wordlists

Create your wordlists in the `wordlists/` directory. Format:

```
www
mail
api
admin
dev
test
staging
```

#### Download Sample Wordlists

**Option 1: Built-in downloader (RECOMMENDED)**

```bash
# Download all wordlists (1K, 10K, 100K)
./target/release/nsec3-generator download-wordlists

# Download specific size
./target/release/nsec3-generator download-wordlists --size 10k

# Download to custom directory
./target/release/nsec3-generator download-wordlists --output /path/to/wordlists
```

**Option 2: Using Makefile**

```bash
make wordlists
```

**Option 3: Manual download**

```bash
curl -sL https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-5000.txt | head -n 1000 > wordlists/subdomains-1k.txt

curl -sL https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-20000.txt | head -n 10000 > wordlists/subdomains-10k.txt

curl -sL https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-110000.txt | head -n 100000 > wordlists/subdomains-100k.txt
```

**Downloaded wordlists:**
- `subdomains-1k.txt` - 1,000 subdomains (~6 KB)
- `subdomains-10k.txt` - 10,000 subdomains (~70 KB)
- `subdomains-100k.txt` - 100,000 subdomains (~960 KB)

---

## Integration with DNSight

### Workflow Overview

```
┌─────────────────┐
│  NSECGenerator  │
│  (Rust binary)  │
└────────┬────────┘
         │
         │ 1. Reads wordlist
         │ 2. Computes SHA1+Base32 hashes
         │ 3. Generates JSON cache
         │
         ▼
┌─────────────────────────┐
│  output/nsec3_*.json    │
│  (Pre-computed hashes)  │
└────────┬────────────────┘
         │
         │ Copy to cache dir
         │
         ▼
┌─────────────────────────┐
│ ../data/nsec3_cache/    │
│ (DNSight cache)         │
└────────┬────────────────┘
         │
         │ Auto-loaded by DNSight
         │
         ▼
┌─────────────────────────┐
│  dnsight zonewalk       │
│  (Instant cracking!)    │
└─────────────────────────┘
```

### Step-by-Step Integration

#### 1. Generate Hashes (NSECGenerator)

```bash
cd NSECGenerator

# Build
cargo build --release

# Generate hashes
./target/release/nsec3-generator \
    --domain example.com \
    --wordlist wordlists/subdomains-10k.txt \
    --salt "ABC123" \
    --iterations 5 \
    --output output
```

**Output:**
```
output/nsec3_a1b2c3d4.json  (2.5 MB, 10,000 hashes)
```

#### 2. Deploy to DNSight

```bash
# Copy cache files
cp output/*.json ../data/nsec3_cache/

# Verify
ls -lh ../data/nsec3_cache/
```

#### 3. Use in DNSight

```bash
cd ..
source venv/bin/activate

# DNSight auto-detects and uses cache
dnsight zonewalk example.com
```

**DNSight Output:**
```
✓ Loaded 10,000 hashes from cache
✓ Cache hit: a1b2c3d4... → www.example.com
✓ Cache hit: b2c3d4e5... → mail.example.com
```

### Cache File Format

Generated JSON files are compatible with DNSight:

```json
{
  "domain": "example.com",
  "salt": "ABC123",
  "iterations": 5,
  "wordlist_size": 10000,
  "hashes": {
    "a1b2c3d4e5f6g7h8...": "www.example.com",
    "b2c3d4e5f6g7h8i9...": "mail.example.com"
  }
}
```

### Cache Matching Logic

DNSight matches caches by:
1. **Domain**: Must match exactly
2. **Salt**: Hex string comparison
3. **Iterations**: Integer match

```python
# In DNSight (nsec_walker.py)
cache_key = hashlib.md5(f"{salt}_{iterations}".encode()).hexdigest()
cache_file = f"nsec3_{cache_key}.json"

# NSECGenerator uses the same algorithm
# Same (salt, iterations) = same cache file
```

### When to Re-generate

Re-generate caches when:
- ✅ **Wordlist changes** - New subdomains added
- ✅ **Domain changes** - Different target domain
- ✅ **NSEC3 params change** - Different salt or iterations
- ❌ **Don't re-generate** if params are the same (cache is reusable!)

---

## Salt Management Strategies

### Understanding Salt Requirements

NSEC3 calculates hashes of **FQDN (fully-qualified domain names)**, not just subdomains. This means each domain requires its own set of hashes.

### Strategy 1: Known Salt (RECOMMENDED - 100% Efficiency)

**When to use**: You know the target domain before the engagement.

**Advantages**:
- ✅ Cache hit rate: 100%
- ✅ Maximum efficiency
- ✅ All scans are instantaneous

**Steps**:

```bash
# 1. Discover the target domain's salt
dig +dnssec target.com NSEC3PARAM

# Response:
# target.com. 3600 IN NSEC3PARAM 1 0 10 AABBCCDD
#                                    ^^  ^^^^^^^^
#                                    |   └─ Salt (hex)
#                                    └─ Iterations

# 2. Generate with that exact salt
cd NSECGenerator
./target/release/nsec3-generator \
    -d target.com \
    -w wordlists/subdomains-100k.txt \
    --salt "AABBCCDD" \
    --iterations 10

# 3. Deploy to DNSight
cp output/*.json ../data/nsec3_cache/

# 4. Scan (instantaneous!)
cd ..
dnsight zonewalk target.com  # ⚡ 0.3 seconds
```

### Strategy 2: Unknown Salt (Generate Common Configurations)

**When to use**: You don't know the salt before scanning, or you have multiple unknown targets.

**Advantages**:
- ✅ No prior information needed
- ✅ ~40-50% probability of cache hit
- ✅ Covers most common configurations

**Disadvantages**:
- ⚠️ No guarantee of cache hit
- ⚠️ Generates multiple files (larger size)

**Common configurations generated**:
1. No salt, no iterations (30% of domains)
2. Google Cloud DNS: `DEADBEEF`, 5 iterations
3. AWS Route53: `CAFEBABE`, 10 iterations
4. Cloudflare minimal: `00`, 0 iterations
5. Light security: `AABBCCDD`, 3 iterations
6. Medium security: `12345678`, 5 iterations
7. High security: `FEDCBA98`, 10 iterations
8. Very high security: `FFFFFFFF`, 15 iterations

**Using built-in command** (RECOMMENDED):

```bash
cd NSECGenerator

# Generate common configurations for your domain
./target/release/nsec3-generator generate-common \
    --domain target.com \
    --wordlist wordlists/subdomains-100k.txt

# Deploy
cp output/*.json ../data/nsec3_cache/

# Scan
dnsight zonewalk target.com
```

**Using Makefile** (alternative):

```bash
cd NSECGenerator

# Generate common configurations
make generate-common-for DOMAIN=target.com \
    WORDLIST=wordlists/subdomains-100k.txt

# Deploy
make deploy

# Scan
dnsight zonewalk target.com
```

### Strategy 3: Discover During Scan (Ad-hoc)

**When to use**: You discover a domain unexpectedly during recon.

**Advantages**:
- ✅ No preparation needed
- ✅ DNSight discovers salt automatically
- ✅ Subsequent scans are instantaneous

**Disadvantages**:
- ⚠️ First time is slow (~2 minutes)

**Steps**:

```bash
# 1. First time: DNSight discovers salt automatically
dnsight zonewalk surprise-domain.com

# Output shows:
# 🔐 NSEC3 Configuration:
#    Salt: FEDCBA98
#    Iterations: 15
# [Computes on-the-fly, takes ~2 minutes]

# 2. Generate cache for future scans
cd NSECGenerator
./target/release/nsec3-generator \
    -d surprise-domain.com \
    -w wordlists/subdomains-100k.txt \
    --salt "FEDCBA98" \
    --iterations 15

# 3. Deploy
cp output/*.json ../data/nsec3_cache/

# 4. Next scans: instantaneous
cd ..
dnsight zonewalk surprise-domain.com  # ⚡ Instant!
```

### Strategy 4: Batch for Multiple Domains (Red Team)

**When to use**: You have a list of target domains with their NSEC3 parameters.

**Advantages**:
- ✅ Generate all at once
- ✅ Efficient for multiple targets

**Steps**:

```bash
cd NSECGenerator

# 1. Edit scripts/batch-generate.sh
# Add your domains and their parameters:
CONFIGS=(
    "target1.com:ABC123:5"
    "target2.com:DEF456:10"
    "target3.com::0"           # No salt
)

# 2. Execute batch generation
./scripts/batch-generate.sh wordlists/subdomains-100k.txt

# 3. Deploy all
cp output/*.json ../data/nsec3_cache/

# 4. Scan all (instantaneous!)
dnsight zonewalk target1.com  # ⚡
dnsight zonewalk target2.com  # ⚡
dnsight zonewalk target3.com  # ⚡
```

### Strategy Comparison

| Strategy | When to Use | Cache Hit Rate | First Scan Time | Preparation |
|----------|-------------|----------------|-----------------|-------------|
| **1. Known Salt** | Domain known pre-engagement | 100% | Instant (0.3s) | Low (1 command) |
| **2. Common Configs** | Unknown salt, multiple targets | 40-50% | Mixed | Low (1 script) |
| **3. Discover During Scan** | Unexpected/ad-hoc domain | 0% (1st), 100% (2nd+) | Slow (1st), fast (2nd+) | None |
| **4. Batch Multi-domain** | List of targets with params | 100% | Instant | Medium (edit script) |

### Recommendations by Engagement Type

#### Professional Pentesting
**Recommended strategy**: **#1 - Known Salt**

```bash
# Pre-engagement: Discover salt
dig +dnssec target.com NSEC3PARAM

# Generate with exact salt
./target/release/nsec3-generator -d target.com \
    -w wordlists/subdomains-100k.txt \
    --salt "<discovered_salt>" --iterations <N>

# During engagement: instantaneous scans
dnsight zonewalk target.com  # ⚡ 100% cache hit
```

**Result**: Maximum efficiency, 100% cache hit rate.

#### Red Team / Multiple Unknown Targets
**Recommended strategy**: **#2 - Common Configs**

```bash
# Pre-generate common configurations for all targets
for domain in target1.com target2.com target3.com; do
    ./target/release/nsec3-generator generate-common \
        --domain $domain \
        --wordlist wordlists/subdomains-100k.txt
done

# Deploy all
cp output/*.json ../data/nsec3_cache/

# During engagement: ~50% cache hit
dnsight zonewalk target1.com  # May be instant
dnsight zonewalk target2.com  # May be instant
```

**Result**: ~40-50% success rate on cache hits, good first phase.

#### Bug Bounty / Ad-hoc Recon
**Recommended strategy**: **#3 - Discover During Scan**

```bash
# Discover new domain during recon
dnsight zonewalk new-domain.com  # Slow (1st time)

# DNSight shows you the salt
# Generate cache for future scans
cd NSECGenerator
./target/release/nsec3-generator -d new-domain.com \
    -w wordlists/subdomains-100k.txt \
    --salt "<shown_salt>" --iterations <N>

# Next scans: instantaneous
dnsight zonewalk new-domain.com  # ⚡ Fast
```

**Result**: Slow first time, but learn and optimize progressively.

#### Long Engagement (Same Domain Repeatedly)
**Recommended strategy**: **#1 + Permanent Cache**

```bash
# Generate once at the beginning
./target/release/nsec3-generator -d client.com \
    -w wordlists/subdomains-100k.txt \
    --salt "<salt>" --iterations <N>

cp output/*.json ../data/nsec3_cache/

# During weeks/months: instantaneous scans
dnsight zonewalk client.com  # ⚡ Always fast
dnsight zonewalk client.com  # ⚡ Always fast
```

**Result**: One-time investment, continuous benefit.

### How to Discover a Domain's Salt

#### Method 1: dig + NSEC3PARAM (direct)

```bash
dig +dnssec target.com NSEC3PARAM

# Expected response:
# target.com. 3600 IN NSEC3PARAM 1 0 10 AABBCCDD
#                                    ^^  ^^^^^^^^
#                                    |   └─ Salt (hex)
#                                    └─ Iterations
```

#### Method 2: dig + A record (indirect)

```bash
dig +dnssec target.com A

# Look for NSEC3 lines:
# abc123.target.com. NSEC3 1 0 10 AABBCCDD ...
#                                 ^^  ^^^^^^^^
#                                 |   └─ Salt
#                                 └─ Iterations
```

#### Method 3: DNSight automatic

```bash
dnsight zonewalk target.com

# Output shows:
# 🔐 NSEC3 Configuration:
#    Salt: AABBCCDD
#    Iterations: 10
```

---

## Performance

### Benchmarks

Hardware: Apple M1 Max (10 cores)

| Wordlist Size | Iterations | Time    | Speed          |
|---------------|------------|---------|----------------|
| 1,000         | 0          | 0.02s   | 50,000 hash/s  |
| 10,000        | 0          | 0.15s   | 66,000 hash/s  |
| 100,000       | 0          | 1.50s   | 66,000 hash/s  |
| 1,000,000     | 0          | 15.00s  | 66,000 hash/s  |
| 10,000        | 10         | 0.80s   | 12,500 hash/s  |
| 100,000       | 10         | 8.00s   | 12,500 hash/s  |

**Note:** Performance scales linearly with iterations. More iterations = slower but more secure NSEC3.

### Performance Comparison

| Method                 | 10K Subdomains | 100K Subdomains |
|------------------------|----------------|-----------------|
| **Without Cache**      | ~10 seconds    | ~100 seconds    |
| **With Cache (Rust)**  | ~0.1 seconds   | ~0.5 seconds    |
| **Speedup**            | 100x faster    | 200x faster     |

### Example Output

```
🚀 NSEC3 Hash Generator for DNSight
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 Configuration:
   Domain: example.com
   Wordlist: wordlists/subdomains-10k.txt
   Salt: DEADBEEF
   Iterations: 10
   Threads: 16

📖 Loading wordlist... ✓ 10000 subdomains loaded (0.02s)
⚡ Computing NSEC3 hashes...
[00:00:08] ████████████████████ 10000/10000 (100%) Done!

✅ Hash computation complete!
   Hashes computed: 10000
   Time elapsed: 8.50s
   Speed: 1176 hashes/sec

💾 Saving cache file...
   Output: output/nsec3_a1b2c3d4.json
   Size: 2.50 MB

🎉 Success! Cache file ready for DNSight
```

---

## Development

### Build for Debugging

```bash
cargo build
./target/debug/nsec3-generator --help
```

### Run Without Building

```bash
cargo run -- --domain test.com --wordlist wordlists/custom.txt
```

### Testing

```bash
# Run unit tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for errors
cargo check
```

### Using Makefile

```bash
make help       # Show all available commands
make build      # Build debug binary
make release    # Build optimized release binary
make test       # Run tests
make clean      # Clean build artifacts
make fmt        # Format code
make clippy     # Lint code
make benchmark  # Run performance benchmarks
```

### NSEC3 Algorithm

NSEC3 uses the following algorithm (RFC 5155):

```
IH(salt, x, 0) = H(x || salt)
IH(salt, x, k) = H(IH(salt, x, k-1) || salt)

Where:
- H = SHA1 hash function
- x = fully qualified domain name (lowercase)
- salt = hex-encoded salt value
- k = number of iterations
- || = concatenation
- Result is Base32 encoded (RFC 4648, no padding)
```

**Example:**

```
Domain: www.example.com
Salt: AABBCCDD (hex)
Iterations: 10

Step 1: SHA1("www.example.com" + 0xAABBCCDD)
Step 2: SHA1(result1 + 0xAABBCCDD)
...
Step 11: SHA1(result10 + 0xAABBCCDD)
Final: Base32(result11) = "a1b2c3d4e5f6..."
```

---

## Troubleshooting

### Issue: "Salt must be hex string"

**Solution:** Use hex-encoded salt:

```bash
# Correct
--salt "AABBCCDD"
--salt "FF00FF00"

# Incorrect
--salt "hello"  # Not hex
```

### Issue: "Wordlist not found"

**Solution:** Check file path:

```bash
# List available wordlists
ls -lh wordlists/

# Use absolute path
--wordlist /absolute/path/to/wordlist.txt
```

### Issue: Slow performance

**Solutions:**
1. Build with `--release` flag
2. Increase threads: `--threads 32`
3. Use smaller wordlist for testing

### Issue: Rust not installed

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Issue: Build errors

```bash
cargo clean
cargo build --release
```

### Issue: Cache not loading in DNSight

**1. Verify file exists:**
```bash
ls -lh ../data/nsec3_cache/
```

**2. Verify domain matches:**
```bash
# Check cache domain
cat ../data/nsec3_cache/nsec3_*.json | jq '.domain'
```

**3. Verify parameters:**
```bash
# Check salt and iterations
cat ../data/nsec3_cache/nsec3_*.json | jq '{salt, iterations}'
```

### FAQ

#### Can I have a universal dictionary of subdomains?

**No.** NSEC3 hashes the complete FQDN (www.example.com), not just the subdomain (www). Each domain needs its own set of hashes.

#### How much space do cache files take?

- 10K subdomains: ~250 KB per cache file
- 100K subdomains: ~2.5 MB per cache file
- 1M subdomains: ~25 MB per cache file

#### Can I reuse caches between subdomains of the same domain?

**Yes.** If you generate for `example.com`, the cache works for any scan of `example.com`. You don't need to regenerate for each scan of the same domain.

#### What if the domain changes its salt?

You'll need to regenerate the cache with the new salt. Administrators rarely change these parameters, but it can happen.

#### How many cache files can I have?

Unlimited. DNSight automatically loads the cache that matches the domain and NSEC3 parameters. You can have caches for hundreds of domains.

---

## Resources

- [RFC 5155 - NSEC3](https://tools.ietf.org/html/rfc5155)
- [RFC 4648 - Base32 Encoding](https://tools.ietf.org/html/rfc4648)
- [DNSight](https://github.com/seifreed/DNSight)
- [SecLists Wordlists](https://github.com/danielmiessler/SecLists)
- [NSEC3 Hash Calculator (online)](https://www.sidnlabs.nl/downloads/nsec3-hash-calculator.html)

---

## Support

If you find this tool useful, consider supporting the development:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-support-yellow.svg?style=flat&logo=buy-me-a-coffee)](https://buymeacoffee.com/seifreed)

**[☕ Support on Buy Me a Coffee](https://buymeacoffee.com/seifreed)**

Your support helps maintain and improve this tool!

---

## Author

**Marc Rivero** ([@seifreed](https://github.com/seifreed))

- GitHub: [@seifreed](https://github.com/seifreed)
- Part of the [DNSight](https://github.com/seifreed/DNSight) ecosystem

## License

MIT License with Attribution and Share-Alike Requirements

Copyright (c) 2025 Marc Rivero (@seifreed)

**Key requirements:**
- ✅ Free to use, modify, and distribute
- ✅ **Must credit Marc Rivero (@seifreed)** and link to this project
- ✅ **Derivative works must be open source** (share-alike)
- ✅ **Must mention this project** in your documentation

See [LICENSE](LICENSE) file for full details.

## Contributing

Issues and pull requests are welcome at the GitHub repository.

When contributing, please ensure:
- Code follows Rust best practices
- Documentation is updated
- Tests pass (`cargo test`)
- Commit messages are descriptive

---

**Questions?** Open an issue on GitHub.
