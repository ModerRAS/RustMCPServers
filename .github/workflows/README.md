# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the RustMCPServers project.

## Workflows

### CI Workflow (`.github/workflows/ci.yml`)
- **Purpose**: Continuous integration testing
- **Triggers**: Push to master/develop branches, pull requests to master
- **Jobs**:
  - Test: Run unit tests, formatting checks, and clippy
  - Security: Run security audit with cargo-audit
  - Coverage: Generate and upload test coverage reports
  - Integration: Run integration tests
  - Build: Cross-platform build verification

### Release Workflow (`.github/workflows/release.yml`)
- **Purpose**: Automated releases and publishing
- **Triggers**: Git tags matching `v*`, manual workflow dispatch
- **Jobs**:
  - Build: Cross-platform binary builds
  - Test: Comprehensive testing before release
  - Security: Security audit for release
  - Publish: Create GitHub releases and publish to crates.io
  - Docker: Build and push Docker images

### Security Scan Workflow (`.github/workflows/security-scan.yml`)
- **Purpose**: Comprehensive security scanning
- **Triggers**: Push to master/develop, pull requests, weekly schedule
- **Jobs**:
  - Security Audit: Vulnerability scanning with cargo-audit
  - Dependency Check: Dependency validation with cargo-deny
  - License Check: License compliance verification
  - Secret Scan: Secret detection with TruffleHog
  - CodeQL: Static code analysis
  - SAST Scans: Bandit and Semgrep security scanning
  - SBOM Analysis: Software Bill of Materials generation

## Quality Improvements

### Timeout Configuration
All jobs have appropriate timeout configurations:
- Test jobs: 15-30 minutes
- Security jobs: 10-30 minutes
- Build jobs: 30-45 minutes
- Release jobs: 30 minutes

### Cache Configuration
Comprehensive caching strategies:
- Cargo registry and git dependencies
- Target directories for different build types
- Separate cache keys for different tools (audit, tarpaulin, etc.)
- Fallback restore-keys for better cache hit rates

### Security Enhancements
- Multiple security scanning tools
- Dependency validation
- License compliance checking
- Secret detection
- SBOM generation
- SAST scanning

### Cross-Platform Support
- Multi-OS builds (Ubuntu, Windows, macOS)
- Multi-architecture support (x86_64, aarch64)
- Docker multi-platform builds

### Coverage and Quality
- Test coverage reporting with Codecov
- Integration testing
- Static code analysis
- Comprehensive error reporting

## Usage

The workflows are automatically triggered on appropriate events. Manual triggers are available for release workflows.

## Configuration

Most configuration is self-contained within the workflow files. Secrets are managed through GitHub repository secrets.