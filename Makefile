# Makefile for NSEC3 Generator

.PHONY: help build release test clean run install wordlists generate-common generate-common-for deploy

help:
	@echo "🚀 NSEC3 Generator - Makefile Commands"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "  make build               - Build debug binary"
	@echo "  make release             - Build optimized release binary"
	@echo "  make test                - Run tests"
	@echo "  make clean               - Clean build artifacts"
	@echo "  make run                 - Run example (requires wordlist)"
	@echo "  make install             - Install binary to system"
	@echo "  make wordlists           - Download sample wordlists"
	@echo "  make generate-common     - Generate common configs for example.com"
	@echo "  make generate-common-for - Generate common configs for DOMAIN=<domain>"
	@echo "  make deploy              - Deploy cache files to DNSight"
	@echo ""

build:
	@echo "🔨 Building debug binary..."
	cargo build

release:
	@echo "⚡ Building optimized release binary..."
	cargo build --release
	@ls -lh target/release/nsec3-generator

test:
	@echo "🧪 Running tests..."
	cargo test

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf output/*.json

run: release
	@echo "🏃 Running example..."
	./target/release/nsec3-generator \
		--domain example.com \
		--wordlist wordlists/example.txt \
		--salt "" \
		--iterations 0

install: release
	@echo "📦 Installing to ~/.local/bin/..."
	mkdir -p ~/.local/bin
	cp target/release/nsec3-generator ~/.local/bin/
	@echo "✅ Installed! Make sure ~/.local/bin is in your PATH"

wordlists:
	@echo "📥 Downloading sample wordlists..."
	@mkdir -p wordlists
	@echo "  → Downloading 1K subdomains..."
	@curl -sL https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-5000.txt 2>/dev/null | head -n 1000 > wordlists/subdomains-1k.txt && echo "  ✓ subdomains-1k.txt" || echo "  ✗ Failed to download subdomains-1k.txt"
	@echo "  → Downloading 10K subdomains..."
	@curl -sL https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-20000.txt 2>/dev/null | head -n 10000 > wordlists/subdomains-10k.txt && echo "  ✓ subdomains-10k.txt" || echo "  ✗ Failed to download subdomains-10k.txt"
	@echo "  → Downloading 100K subdomains..."
	@curl -sL https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-110000.txt 2>/dev/null | head -n 100000 > wordlists/subdomains-100k.txt && echo "  ✓ subdomains-100k.txt" || echo "  ✗ Failed to download subdomains-100k.txt"
	@echo ""
	@echo "✅ Wordlists downloaded:"
	@ls -lh wordlists/*.txt 2>/dev/null || echo "  No wordlists found"

# Generate hashes for common configurations (example.com)
generate-common: release
	@echo "🔄 Generating common NSEC3 configurations for example.com..."
	@mkdir -p output
	./target/release/nsec3-generator -d example.com -w wordlists/subdomains-10k.txt -s "" -i 0 -o output
	./target/release/nsec3-generator -d example.com -w wordlists/subdomains-10k.txt -s "DEADBEEF" -i 5 -o output
	./target/release/nsec3-generator -d example.com -w wordlists/subdomains-10k.txt -s "CAFEBABE" -i 10 -o output
	@echo "✅ Generated $(shell ls output/*.json | wc -l) cache files"

# Generate hashes for common configurations (custom domain)
# Usage: make generate-common-for DOMAIN=target.com WORDLIST=wordlists/subdomains-10k.txt
generate-common-for: release
	@if [ -z "$(DOMAIN)" ]; then \
		echo "❌ Error: DOMAIN not specified"; \
		echo "Usage: make generate-common-for DOMAIN=target.com [WORDLIST=wordlists/subdomains-10k.txt]"; \
		exit 1; \
	fi
	@echo "🔄 Generating common NSEC3 configurations for $(DOMAIN)..."
	./scripts/generate-common-configs.sh $(DOMAIN) $(or $(WORDLIST),wordlists/example.txt)

# Deploy to DNSight
deploy:
	@echo "📋 Deploying cache files to DNSight..."
	mkdir -p ../data/nsec3_cache
	cp output/*.json ../data/nsec3_cache/
	@echo "✅ Deployed $(shell ls output/*.json | wc -l) cache files"
	@ls -lh ../data/nsec3_cache/

# Benchmark
benchmark: release
	@echo "⏱️  Running benchmark..."
	@echo ""
	@echo "Testing 1K subdomain wordlist..."
	time ./target/release/nsec3-generator -d bench.com -w wordlists/subdomains-1k.txt -s "" -i 0 -o /tmp
	@echo ""
	@echo "Testing 10K subdomain wordlist..."
	time ./target/release/nsec3-generator -d bench.com -w wordlists/subdomains-10k.txt -s "" -i 0 -o /tmp

# Development helpers
fmt:
	cargo fmt

clippy:
	cargo clippy

check:
	cargo check
