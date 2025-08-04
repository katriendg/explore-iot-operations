# Developer Workflow and Tools

## Build and Dependency Management

**Core Principles:**
- Use Cargo for all build and dependency management
- Follow semantic versioning for dependencies
- Use feature flags to control compilation
- Organize workspace for multi-crate projects
- Understand build profiles and optimization

**Cargo Project Structure:**

```toml
# Cargo.toml - Basic project configuration
[package]
name = "my-rust-project"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
description = "A brief description of the project"
repository = "https://github.com/user/repo"
keywords = ["rust", "example"]
categories = ["development-tools"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"

[build-dependencies]
cc = "1.0"

[[bin]]
name = "main"
path = "src/main.rs"

[[bin]]
name = "cli"
path = "src/bin/cli.rs"

[lib]
name = "mylib"
path = "src/lib.rs"
```

**Workspace Configuration:**

```toml
# Cargo.toml - Workspace root
[workspace]
members = [
    "core",
    "cli",
    "web-api", 
    "shared-types"
]

[workspace.dependencies]
# Shared dependencies across workspace
serde = "1.0"
tokio = "1.0"
log = "0.4"

# Member crates/core/Cargo.toml
[package]
name = "project-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true, features = ["derive"] }
tokio = { workspace = true, features = ["rt-multi-thread"] }
```

**Feature Flag Management:**

```toml
[features]
default = ["std", "serde"]
std = []
serde = ["dep:serde", "dep:serde_json"]
async = ["dep:tokio"]
metrics = ["dep:prometheus"]
full = ["std", "serde", "async", "metrics"]

# Optional dependencies activated by features
[dependencies]
serde = { version = "1.0", optional = true }
serde_json = { version = "1.0", optional = true }
tokio = { version = "1.0", optional = true }
prometheus = { version = "0.13", optional = true }
```

**Essential Cargo Commands:**

```bash
# Project creation and management
cargo new my-project                 # Create new binary project
cargo new --lib my-library          # Create new library project
cargo init                          # Initialize in existing directory

# Building and running
cargo build                         # Debug build
cargo build --release              # Release build
cargo run                          # Build and run main binary
cargo run --bin cli                # Run specific binary
cargo run --example basic          # Run example

# Testing
cargo test                          # Run all tests
cargo test unit_tests              # Run specific test module
cargo test --release               # Test release build
cargo test --doc                   # Test documentation examples

# Documentation
cargo doc                          # Generate documentation
cargo doc --open                   # Generate and open docs
cargo doc --no-deps               # Skip dependency docs

# Dependency management
cargo add serde                    # Add dependency (Cargo 1.62+)
cargo remove old-dep               # Remove dependency
cargo update                       # Update dependencies
cargo tree                        # Show dependency tree
cargo audit                       # Security audit

# Quality and formatting
cargo fmt                          # Format code
cargo clippy                       # Linting
cargo clippy -- -D warnings       # Treat warnings as errors
cargo check                       # Fast syntax check
```

**Build Profiles and Optimization:**

```toml
# Cargo.toml profiles
[profile.dev]
opt-level = 0      # No optimization
debug = true       # Include debug info
panic = "unwind"   # Unwind on panic

[profile.release]
opt-level = 3      # Maximum optimization
debug = false      # No debug info
panic = "abort"    # Abort on panic
lto = true        # Link-time optimization
codegen-units = 1 # Better optimization
strip = true      # Strip symbols

[profile.test]
opt-level = 1     # Some optimization for tests
debug = true

[profile.bench]
opt-level = 3     # Full optimization for benchmarks
debug = false
```

**Dependency Version Management:**

```toml
[dependencies]
# Caret requirements (compatible updates)
serde = "^1.2.3"        # >=1.2.3, <2.0.0
tokio = "1"             # >=1.0.0, <2.0.0

# Tilde requirements (patch-level changes)
log = "~0.4.17"         # >=0.4.17, <0.5.0

# Exact requirements
openssl = "=0.10.55"    # Exactly 0.10.55

# Git dependencies
my-git-dep = { git = "https://github.com/user/repo", branch = "main" }
my-git-tag = { git = "https://github.com/user/repo", tag = "v1.0.0" }

# Path dependencies (for local development)
my-local-crate = { path = "../my-local-crate" }

# Conditional dependencies
[target.'cfg(unix)'.dependencies]
nix = "0.26"

[target.'cfg(windows)'.dependencies]
winapi = "0.3"
```

**Build Scripts (build.rs):**

```rust
// build.rs - Custom build logic
use std::env;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun build script if proto files change
    println!("cargo:rerun-if-changed=proto/");
    
    // Set environment variables for build
    let target = env::var("TARGET").unwrap();
    if target.contains("musl") {
        println!("cargo:rustc-link-lib=static=ssl");
        println!("cargo:rustc-link-lib=static=crypto");
    }
    
    // Generate code from protobuf files
    if Path::new("proto").exists() {
        protoc_rust::Codegen::new()
            .out_dir("src/generated")
            .inputs(&["proto/message.proto"])
            .include("proto")
            .run()
            .expect("Running protoc failed");
    }
    
    // Feature-based conditional compilation
    if cfg!(feature = "vendored-ssl") {
        println!("cargo:rustc-cfg=vendored_ssl");
    }
}
```

**Cross-Compilation:**

```bash
# Install targets
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add wasm32-unknown-unknown

# Build for different targets
cargo build --target x86_64-pc-windows-gnu
cargo build --target aarch64-unknown-linux-gnu

# Use cross for complex cross-compilation
cargo install cross
cross build --target aarch64-unknown-linux-gnu
```

**Cargo Configuration:**

```toml
# .cargo/config.toml - Project-specific configuration
[build]
target = "x86_64-unknown-linux-gnu"
rustflags = ["-C", "target-cpu=native"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[registry]
default = "crates-io"

[registries.my-company]
index = "https://my-company.com/git/index"

[alias]
b = "build"
c = "check"
t = "test"
r = "run"
rr = "run --release"
```

## Testing and Debugging Guidelines

**Core Principles:**
- Write tests alongside code development (TDD approach)
- Use different test types: unit, integration, documentation, and property-based
- Leverage Rust's built-in testing framework
- Use appropriate debugging tools and techniques
- Test both happy paths and error conditions

**Unit Testing Patterns:**

```rust
// src/lib.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_divide_success() {
        assert_eq!(divide(10.0, 2.0).unwrap(), 5.0);
        assert_eq!(divide(0.0, 1.0).unwrap(), 0.0);
    }

    #[test]
    fn test_divide_by_zero() {
        match divide(10.0, 0.0) {
            Err(msg) => assert_eq!(msg, "Division by zero"),
            Ok(_) => panic!("Expected error for division by zero"),
        }
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_overflow_panic() {
        let _result = i32::MAX + 1; // This will panic in debug mode
    }

    #[test]
    #[ignore]
    fn expensive_test() {
        // Long-running test, run with: cargo test -- --ignored
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
```

**Integration Testing:**

```rust
// tests/integration_test.rs
use my_crate::{add, divide};

#[test]
fn test_integration_workflow() {
    let sum = add(3, 4);
    let result = divide(sum as f64, 2.0).unwrap();
    assert_eq!(result, 3.5);
}

#[test]
fn test_error_handling_integration() {
    let sum = add(5, 5);
    let error_result = divide(sum as f64, 0.0);
    assert!(error_result.is_err());
}
```

**Property-Based Testing:**

```rust
// Cargo.toml
// [dev-dependencies]
// proptest = "1.0"

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_add_commutative(a in any::<i32>(), b in any::<i32>()) {
        // Addition should be commutative
        prop_assume!(a.checked_add(b).is_some()); // Avoid overflow
        prop_assert_eq!(add(a, b), add(b, a));
    }

    #[test]
    fn test_add_associative(a in any::<i16>(), b in any::<i16>(), c in any::<i16>()) {
        // Addition should be associative: (a + b) + c == a + (b + c)
        let left = add(add(a as i32, b as i32), c as i32);
        let right = add(a as i32, add(b as i32, c as i32));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn test_divide_never_zero_denominator(
        numerator in any::<f64>(),
        denominator in 1.0f64..1000.0f64  // Non-zero range
    ) {
        let result = divide(numerator, denominator);
        prop_assert!(result.is_ok());
    }
}
```

**Async Testing:**

```rust
// Cargo.toml
// [dev-dependencies]
// tokio-test = "0.4"

use tokio_test;

async fn async_add(a: i32, b: i32) -> i32 {
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    a + b
}

#[tokio::test]
async fn test_async_function() {
    let result = async_add(2, 3).await;
    assert_eq!(result, 5);
}

#[tokio::test]
async fn test_async_with_timeout() {
    let result = tokio::time::timeout(
        tokio::time::Duration::from_millis(100),
        async_add(2, 3)
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
}
```

**Mocking and Test Doubles:**

```rust
// Using traits for dependency injection and mocking
trait DataStore {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

struct RealDataStore {
    data: std::collections::HashMap<String, String>,
}

impl DataStore for RealDataStore {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    
    fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
}

struct Service<T: DataStore> {
    store: T,
}

impl<T: DataStore> Service<T> {
    fn new(store: T) -> Self {
        Service { store }
    }
    
    fn process(&mut self, key: &str) -> String {
        match self.store.get(key) {
            Some(value) => format!("Found: {}", value),
            None => {
                let new_value = "default".to_string();
                self.store.set(key, new_value.clone());
                format!("Created: {}", new_value)
            }
        }
    }
}

#[cfg(test)]
mod service_tests {
    use super::*;
    use std::collections::HashMap;

    struct MockDataStore {
        data: HashMap<String, String>,
        get_calls: Vec<String>,
    }

    impl MockDataStore {
        fn new() -> Self {
            MockDataStore {
                data: HashMap::new(),
                get_calls: Vec::new(),
            }
        }
    }

    impl DataStore for MockDataStore {
        fn get(&self, key: &str) -> Option<String> {
            self.data.get(key).cloned()
        }
        
        fn set(&mut self, key: &str, value: String) {
            self.data.insert(key.to_string(), value);
        }
    }

    #[test]
    fn test_service_with_mock() {
        let mock_store = MockDataStore::new();
        let mut service = Service::new(mock_store);
        
        let result = service.process("test_key");
        assert_eq!(result, "Created: default");
    }
}
```

**Benchmarking:**

```rust
// Cargo.toml
// [dev-dependencies]
// criterion = "0.5"

// benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_crate::{add, divide};

fn benchmark_add(c: &mut Criterion) {
    c.bench_function("add", |b| {
        b.iter(|| add(black_box(42), black_box(24)))
    });
}

fn benchmark_divide(c: &mut Criterion) {
    c.bench_function("divide", |b| {
        b.iter(|| divide(black_box(100.0), black_box(3.0)))
    });
}

fn benchmark_with_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut sum = 0;
                for i in 0..size {
                    sum = add(sum, black_box(i));
                }
                sum
            });
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_add, benchmark_divide, benchmark_with_input);
criterion_main!(benches);
```

**Debugging Techniques:**

```rust
// Debug printing and logging
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn debug_example() {
    let point = Point { x: 3, y: 4 };
    
    // Debug print
    println!("{:?}", point);
    dbg!(&point);
    
    // Pretty print
    println!("{:#?}", point);
    
    // Conditional compilation for debug
    #[cfg(debug_assertions)]
    println!("Debug mode: point = {:?}", point);
}

// Using the log crate for structured logging
use log::{debug, info, warn, error};

fn logging_example() {
    env_logger::init(); // Initialize logger
    
    info!("Starting application");
    debug!("Debug information: value = {}", 42);
    warn!("Warning: this might be an issue");
    error!("Error occurred: {}", "something went wrong");
}

// Assert macros for debugging
fn assertion_examples() {
    let x = 5;
    
    assert!(x > 0);
    assert_eq!(x, 5);
    assert_ne!(x, 0);
    
    debug_assert!(x > 0); // Only in debug builds
    debug_assert_eq!(x, 5);
}
```

**Test Organization and Commands:**

```bash
# Running tests
cargo test                          # All tests
cargo test unit_tests              # Specific module
cargo test test_add                # Specific test
cargo test -- --nocapture         # Show print statements
cargo test -- --test-threads=1    # Single-threaded
cargo test -- --ignored           # Run ignored tests

# Test with different configurations
cargo test --release              # Release mode tests
cargo test --features "feature1"  # With specific features
cargo test --all-features        # All features enabled

# Documentation tests
cargo test --doc                  # Test code in documentation

# Benchmarks
cargo bench                       # Run benchmarks
cargo bench -- baseline          # Save baseline
cargo bench -- --save-baseline main  # Named baseline

# Coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html        # Generate coverage report
```

**Test Configuration:**

```toml
# Cargo.toml test configuration
[[test]]
name = "integration"
path = "tests/integration/main.rs"

[[bench]]
name = "my_benchmark"
harness = false  # Use custom benchmark framework
path = "benches/my_benchmark.rs"

[profile.test]
opt-level = 1    # Some optimization for faster tests
debug = true     # Keep debug info

[profile.bench]
debug = true     # Debug info for profiling
```

**Debugging Tools:**
- `rust-gdb` and `rust-lldb` for debugging compiled code
- `cargo expand` to see macro expansions
- `cargo asm` to inspect generated assembly
- `strace`/`ltrace` for system call tracing
- `valgrind` for memory error detection
- IDE debuggers (VS Code, CLion, etc.)

**Testing Best Practices:**
- Test both success and failure cases
- Use descriptive test names
- Keep tests fast and independent
- Use property-based testing for complex logic
- Mock external dependencies
- Measure test coverage but focus on meaningful tests
- Write tests before fixing bugs to prevent regression

## Development Workflow and Tooling

**Core Principles:**
- Use consistent formatting and linting across the project
- Automate quality checks in CI/CD pipelines
- Configure IDE/editor for optimal Rust development
- Generate comprehensive documentation
- Follow conventional development workflow

**Code Formatting with rustfmt:**

```toml
# rustfmt.toml - Project formatting configuration
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
indent_style = "Block"
wrap_comments = true
format_code_in_doc_comments = true
normalize_comments = true
normalize_doc_attributes = true
format_strings = false
format_macro_matchers = true
format_macro_bodies = true
use_small_heuristics = "Default"
where_single_line = true
imports_granularity = "Crate"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
edition = "2021"
```

```bash
# Formatting commands
cargo fmt                          # Format all code
cargo fmt -- --check              # Check if formatted
cargo fmt -- --config tab_spaces=2  # Override config
```

**Linting with Clippy:**

```toml
# Cargo.toml - Clippy configuration
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# Allow specific lints
module_name_repetitions = "allow"
missing_errors_doc = "allow"

# Deny specific lints
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

```bash
# Clippy commands
cargo clippy                       # Basic linting
cargo clippy --all-targets        # All targets (bins, tests, etc.)
cargo clippy --all-features       # All features
cargo clippy -- -D warnings       # Treat warnings as errors
cargo clippy --fix                # Auto-fix issues
```

**IDE Configuration:**

```json
// .vscode/settings.json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.allTargets": true,
    "rust-analyzer.cargo.allFeatures": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.inlayHints.parameterHints.enable": false,
    "rust-analyzer.inlayHints.typeHints.enable": true,
    "rust-analyzer.completion.autoself.enable": true,
    "rust-analyzer.completion.autoimport.enable": true,
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}

// .vscode/extensions.json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb"
    ]
}
```

**Documentation Generation:**

```rust
//! # My Crate
//! 
//! This crate provides utilities for data processing.
//! 
//! ## Quick Start
//! 
//! ```rust
//! use my_crate::processor::DataProcessor;
//! 
//! let processor = DataProcessor::new();
//! let result = processor.process("input data");
//! assert_eq!(result, "processed: input data");
//! ```

/// A data processor that transforms input strings.
/// 
/// # Examples
/// 
/// ```rust
/// use my_crate::processor::DataProcessor;
/// 
/// let processor = DataProcessor::new();
/// let result = processor.process("hello");
/// assert_eq!(result, "processed: hello");
/// ```
/// 
/// # Errors
/// 
/// This function will return an error if the input is empty:
/// 
/// ```rust
/// use my_crate::processor::DataProcessor;
/// 
/// let processor = DataProcessor::new();
/// let result = processor.process_checked("");
/// assert!(result.is_err());
/// ```
pub struct DataProcessor {
    prefix: String,
}

impl DataProcessor {
    /// Creates a new data processor with default settings.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use my_crate::processor::DataProcessor;
    /// let processor = DataProcessor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            prefix: "processed: ".to_string(),
        }
    }
    
    /// Processes the input string by adding a prefix.
    /// 
    /// # Arguments
    /// 
    /// * `input` - The string to process
    /// 
    /// # Returns
    /// 
    /// A processed string with the prefix added.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use my_crate::processor::DataProcessor;
    /// let processor = DataProcessor::new();
    /// assert_eq!(processor.process("test"), "processed: test");
    /// ```
    pub fn process(&self, input: &str) -> String {
        format!("{}{}", self.prefix, input)
    }
    
    /// Processes input with validation.
    /// 
    /// # Errors
    /// 
    /// Returns an error if input is empty or invalid.
    /// 
    /// ```rust
    /// # use my_crate::processor::DataProcessor;
    /// let processor = DataProcessor::new();
    /// assert!(processor.process_checked("").is_err());
    /// assert!(processor.process_checked("valid").is_ok());
    /// ```
    pub fn process_checked(&self, input: &str) -> Result<String, ProcessingError> {
        if input.is_empty() {
            return Err(ProcessingError::EmptyInput);
        }
        Ok(self.process(input))
    }
}

/// Errors that can occur during processing.
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    /// Input was empty
    #[error("Input cannot be empty")]
    EmptyInput,
    /// Input was invalid
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

```bash
# Documentation commands
cargo doc                          # Generate docs
cargo doc --open                   # Generate and open
cargo doc --no-deps               # Skip dependencies
cargo doc --document-private-items # Include private items
```

**Continuous Integration Setup:**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
        override: true
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run tests
      run: cargo test --all-features
    
    - name: Run documentation tests
      run: cargo test --doc
    
    - name: Check documentation
      run: cargo doc --no-deps --all-features
```

**Development Workflow:**

```bash
# Daily development workflow
git checkout -b feature/new-feature

# Write code and tests
cargo check                        # Fast syntax check
cargo test                        # Run tests
cargo clippy                      # Lint code
cargo fmt                         # Format code

# Before committing
cargo test --all-features         # Full test suite
cargo clippy --all-targets        # Full lint check
cargo doc                         # Ensure docs build

git add .
git commit -m "feat: add new feature"
git push origin feature/new-feature

# Code review and merge
```

**Pre-commit Hooks:**

```bash
# Install pre-commit
pip install pre-commit

# .pre-commit-config.yaml
repos:
-   repo: local
    hooks:
    -   id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt
        language: system
        types: [rust]
        args: ["--", "--check"]
    -   id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy
        language: system
        types: [rust]
        args: ["--all-targets", "--all-features", "--", "-D", "warnings"]
    -   id: cargo-test
        name: cargo test
        entry: cargo test
        language: system
        types: [rust]
        pass_filenames: false

# Install hooks
pre-commit install
```

**Essential Development Tools:**

```bash
# Install useful cargo extensions
cargo install cargo-watch          # Auto-rebuild on changes
cargo install cargo-expand         # Show macro expansions
cargo install cargo-audit          # Security audit
cargo install cargo-outdated       # Check for outdated deps
cargo install cargo-bloat          # Analyze binary size
cargo install cargo-tree           # Dependency tree
cargo install cargo-edit           # Add/remove dependencies

# Usage examples
cargo watch -x check               # Auto-check on file changes
cargo watch -x test                # Auto-test on changes
cargo expand                       # Show macro expansions
cargo audit                        # Security audit
cargo outdated                     # Check outdated dependencies
cargo bloat --release             # Analyze binary size
```

**Makefile for Common Tasks:**

```makefile
# Makefile
.PHONY: check test lint fmt doc clean install

check:
	cargo check --all-targets --all-features

test:
	cargo test --all-features

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

doc:
	cargo doc --no-deps --all-features

clean:
	cargo clean

install:
	cargo install --path .

# Development workflow
dev: fmt check test lint doc

# CI workflow
ci: fmt-check clippy test-all doc

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test-all:
	cargo test --all-features
	cargo test --doc

# Release workflow
release: clean ci
	cargo build --release
```

**Editor/IDE Specific Configurations:**

```rust
// For IntelliJ/CLion - Add to Cargo.toml
[workspace]
members = [
    "core",
    "cli"
]

// Enable experimental features in .idea/workspace.xml
// <component name="RustProjectSettings">
//   <option name="useCargoCheckForBuild" value="true" />
// </component>
```

**Development Best Practices:**
- Use `cargo check` for fast feedback during development
- Run `cargo clippy` regularly to catch common issues
- Keep documentation up to date with code changes
- Use feature flags to control optional functionality
- Set up CI to catch issues early
- Use pre-commit hooks to enforce code quality
- Profile and benchmark performance-critical code
- Keep dependencies up to date and audit for security issues
