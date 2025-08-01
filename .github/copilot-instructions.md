# Azure IoT Operations - Development Guide

This repository contains tools, samples, and tutorials for Azure IoT Operations customers. The codebase emphasizes cloud-native IoT patterns with strong support for WASM modules, DAPR integration, and container-based development.

## Architecture Overview

**Core Structure:**
- `samples/` - Self-contained demos organized by technology (Go, .NET, Rust, Python)  
- `lib/` - Shared Go libraries (`mage`, `proto`, `env`, `logger`) used across samples
- `scripts/` - Setup automation (`iotopsQuickstart.sh`, `arcConnect.sh`)
- `tutorials/` - Step-by-step walkthroughs with integrated code/docs

**Key Integration Points:**
- Azure IoT Operations + K3s cluster via K3d (`.devcontainer/`)
- DAPR runtime for pub/sub and state management (`samples/dapr-*`)
- **WASM modules for data flow graphs (`samples/wasm/`) - Primary focus area**
- Container registries for artifact distribution (GHCR workflows)

## Development Patterns

### Go Projects with Mage
Most Go samples use `lib/mage` for standardized build/test/lint workflows:

```go
// In magefile.go
func CI() error {
    return mage.CI(
        "github.com/explore-iot-ops/samples/my-sample/",
        map[string]any{"cmd": nil}, // test exclusions
        3000,  // timeout
        0.80,  // block coverage
        0.85,  // overall coverage
    )
}
```

Run with: `mage ci` (installs tools, lints, tests, validates coverage)

### WASM Development Workflow
**Rust modules:** Use Docker builder for compilation:
```bash
docker run --rm -v "$(pwd):/workspace" \
  ghcr.io/azure-samples/explore-iot-operations/rust-wasm-builder \
  --app-name temperature --build-mode release
```

**Python modules:** Similar pattern with Python builder:
```bash
docker run --rm -v "$(pwd):/workspace" \
  ghcr.io/azure-samples/explore-iot-operations/python-wasm-builder \
  --app-name my_module --app-type map
```

**Operator Types:** Map (transform), Filter (allow/reject), Branch (route), Accumulate (aggregate), Delay (timing)

## WASM Data Flow Development Guide

**For deployment questions:** Reference `samples/wasm/README.md` for comprehensive deployment guide and graph configuration examples

**For data flow graph design:** 
- Simple workflows: `samples/wasm/graph-simple.yaml`
- Complex multi-sensor processing: `samples/wasm/graph-complex.yaml`
- Graph structure patterns and operator chaining examples

**For Rust development:**
- Examples: `samples/wasm/rust/examples/` (temperature, humidity, format, etc.)
- Build environment: `samples/wasm/rust/Dockerfile`
- Implementation patterns: See temperature example for operator macros

**For Python development:**
- Examples: `samples/wasm/python/examples/` (map, filter, branch operators)
- Build environment: `samples/wasm/python/Dockerfile`
- WIT schemas: `samples/wasm/python/schema/`

## Critical Commands

**Environment Setup:**
```bash
# Initial cluster connection
./scripts/iotopsQuickstart.sh  # Interactive Azure setup
source ./scripts/env_vars.txt  # Load generated variables

# Manual Arc connection  
az connectedk8s connect -n $CLUSTER_NAME -g $RESOURCE_GROUP -l $LOCATION
```

**Development Workflows:**
```bash
# Go projects
mage ci                    # Full validation pipeline
mage format               # Code formatting with golines
mage lint                 # golangci-lint execution
mage cover 3000           # Tests with coverage

# WASM deployment
oras push myregistry.azurecr.io/module:1.0.0 module.wasm
oras push myregistry.azurecr.io/graph:1.0.0 graph.yaml
```

## Project Conventions

**File Organization:**
- Each sample includes `README.md` with prerequisites, build steps, deployment guide
- Dockerfiles for containerized samples use multi-stage builds
- Go modules follow `github.com/explore-iot-ops/samples/[name]` import paths

**Testing Strategy:**
- Unit tests required for all Go packages (enforced by `mage.EnsureTests`)
- Coverage thresholds: typically 80% block, 85% overall
- Integration tests via container builds in CI/CD

**Container Strategy:**  
- Base images: `mcr.microsoft.com/devcontainers/` for development
- WASM builders: Custom images with Rust/Python toolchains  
- GHCR for artifact distribution (`ghcr.io/azure-samples/explore-iot-operations/`)

## Key Libraries

**`lib/proto`:** Generic message encoding/decoding for data flows
**`lib/mage`:** Build automation (linting, testing, coverage, documentation)  
**`lib/env`:** Environment variable management with validation
**`lib/logger`:** Structured logging for distributed scenarios

When adding new samples, follow the established patterns in `samples/anomaly-detection/` (Go) or `samples/wasm/rust/examples/temperature/` (Rust) for project structure and build integration.

**WASM Development:** Always start with `samples/wasm/README.md` for overview, then navigate to language-specific examples and build environments based on requirements.
