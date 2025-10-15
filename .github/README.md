# GitHub Workflows

This directory contains GitHub Actions workflows for continuous integration and release automation.

## Workflows

### CI (`ci.yml`)

Runs on every push to `main` or `develop` branches and on pull requests.

**Jobs:**
- **Test Suite**: Runs tests on Linux, Windows, and macOS
- **Clippy**: Linting with Rust's clippy
- **Rustfmt**: Code formatting checks
- **Build Check**: Builds debug and release versions
- **Security Audit**: Checks for security vulnerabilities with `cargo-audit`

**Trigger:**
```bash
git push origin main
# Or create a pull request
```

### Release (`release.yml`)

Builds binaries for all supported platforms and creates a GitHub release.

**Supported Platforms:**
- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- Linux ARM64 (`aarch64-unknown-linux-gnu`)
- Windows x86_64 (`x86_64-pc-windows-msvc`)
- macOS x86_64 (`x86_64-apple-darwin`)
- macOS ARM64 (`aarch64-apple-darwin`)

**Trigger:**
```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

**Or manually trigger from GitHub Actions UI**

**Artifacts:**
Each platform generates:
- Binary archive (`.tar.gz` or `.zip`)
- SHA-256 checksum file

## Creating a Release

1. **Update version in `Cargo.toml`:**
   ```toml
   version = "1.0.0"
   ```

2. **Commit changes:**
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 1.0.0"
   ```

3. **Create and push tag:**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

4. **GitHub Actions will automatically:**
   - Build binaries for all platforms
   - Run tests
   - Create GitHub release
   - Upload binaries and checksums

## Manual Workflow Trigger

You can also manually trigger the release workflow from the GitHub Actions UI:

1. Go to **Actions** tab
2. Select **Build and Release** workflow
3. Click **Run workflow**
4. Select branch and run

## Badges

Add to your README:

```markdown
[![CI](https://github.com/seifreed/NSECGenerator/workflows/CI/badge.svg)](https://github.com/seifreed/NSECGenerator/actions/workflows/ci.yml)
[![Release](https://github.com/seifreed/NSECGenerator/workflows/Build%20and%20Release/badge.svg)](https://github.com/seifreed/NSECGenerator/actions/workflows/release.yml)
```
