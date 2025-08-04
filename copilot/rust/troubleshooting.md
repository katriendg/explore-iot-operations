# Troubleshooting and Common Issues

**Core Principles:**
- Understand common Rust error patterns
- Use compiler error messages as learning tools
- Know debugging strategies for ownership and lifetime issues
- Recognize performance bottlenecks and their solutions
- Use appropriate tools for diagnosis

## Common Compilation Errors and Solutions

### 1. Borrow Checker Issues

```rust
// Problem: Cannot borrow as mutable while immutable borrow exists
fn borrow_error_example() {
    let mut data = vec![1, 2, 3, 4, 5];
    let first = &data[0];        // Immutable borrow
    data.push(6);               // ERROR: Cannot borrow as mutable
    println!("First: {}", first);
}

// Solution: Limit scope of immutable borrow
fn borrow_fix_example() {
    let mut data = vec![1, 2, 3, 4, 5];
    let first = data[0];        // Copy the value instead of borrowing
    data.push(6);              // OK: No active borrows
    println!("First: {}", first);
}

// Alternative solution: Restructure to avoid conflict
fn borrow_fix_alternative() {
    let mut data = vec![1, 2, 3, 4, 5];
    {
        let first = &data[0];   // Immutable borrow in limited scope
        println!("First: {}", first);
    }                          // Borrow ends here
    data.push(6);             // OK: No active borrows
}
```

### 2. Lifetime Parameter Issues

```rust
// Problem: Missing lifetime specifier
fn longest_error(x: &str, y: &str) -> &str {  // ERROR: Missing lifetime
    if x.len() > y.len() { x } else { y }
}

// Solution: Add explicit lifetime parameters
fn longest_fixed<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Problem: Lifetime of return value
struct Container {
    data: String,
}

impl Container {
    // ERROR: Cannot return reference to data that will be dropped
    fn get_data_error(&self) -> &str {
        let processed = self.data.to_uppercase();
        &processed  // ERROR: `processed` will be dropped
    }
    
    // Solution: Return owned data or borrow from self
    fn get_data_fixed(&self) -> String {
        self.data.to_uppercase()  // Return owned String
    }
    
    fn get_data_ref(&self) -> &str {
        &self.data  // Borrow from self, which lives long enough
    }
}
```

### 3. Move and Ownership Issues

```rust
// Problem: Use after move
fn move_error_example() {
    let data = String::from("hello");
    let moved_data = data;      // `data` is moved
    println!("{}", data);       // ERROR: Use after move
}

// Solution: Clone when you need multiple owners
fn move_fix_clone() {
    let data = String::from("hello");
    let cloned_data = data.clone();  // Clone instead of move
    println!("Original: {}", data);
    println!("Clone: {}", cloned_data);
}

// Solution: Use references when you don't need ownership
fn move_fix_reference() {
    let data = String::from("hello");
    let data_ref = &data;       // Borrow instead of move
    println!("Original: {}", data);
    println!("Reference: {}", data_ref);
}

// Problem: Cannot move out of borrowed content
fn move_from_borrow_error(data: &Vec<String>) -> String {
    data[0]  // ERROR: Cannot move out of index of borrowed content
}

// Solution: Clone the value
fn move_from_borrow_fix(data: &Vec<String>) -> String {
    data[0].clone()  // Clone the string
}

// Alternative: Return a reference
fn move_from_borrow_ref(data: &Vec<String>) -> &String {
    &data[0]  // Return reference instead
}
```

### 4. Trait Bound and Generic Issues

```rust
// Problem: Trait bound not satisfied
fn generic_error<T>(value: T) -> T {
    println!("{}", value);  // ERROR: T doesn't implement Display
    value
}

// Solution: Add appropriate trait bounds
fn generic_fixed<T: std::fmt::Display>(value: T) -> T {
    println!("{}", value);
    value
}

// Problem: Complex trait bounds
fn complex_bounds_error<T>(items: Vec<T>) -> T 
where 
    T: Clone + PartialOrd + std::fmt::Debug
{
    let mut items = items;
    items.sort();           // ERROR: sort() needs Ord, not PartialOrd
    items[0].clone()
}

// Solution: Use correct trait bounds
fn complex_bounds_fixed<T>(mut items: Vec<T>) -> T 
where 
    T: Clone + Ord + std::fmt::Debug  // Changed to Ord
{
    items.sort();
    items[0].clone()
}
```

### 5. Pattern Matching Issues

```rust
// Problem: Non-exhaustive patterns
enum Status {
    Active,
    Inactive,
    Pending,
}

fn match_error(status: Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Inactive => "inactive",
        // ERROR: Missing Status::Pending
    }
}

// Solution: Handle all cases or use wildcard
fn match_fixed(status: Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Inactive => "inactive",
        Status::Pending => "pending",
    }
}

// Alternative: Use wildcard for default case
fn match_wildcard(status: Status) -> &'static str {
    match status {
        Status::Active => "active",
        _ => "other",
    }
}
```

## Performance Debugging Strategies

### 1. Identifying Allocation Hotspots

```rust
// Problem: Unnecessary allocations in hot paths
fn inefficient_processing(items: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for item in items {
        let processed = format!("processed: {}", item);  // Allocation
        result.push(processed);
    }
    result
}

// Solution: Pre-allocate and avoid intermediate allocations
fn efficient_processing(items: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(items.len());  // Pre-allocate
    for item in items {
        let mut processed = String::with_capacity(item.len() + 11);
        processed.push_str("processed: ");
        processed.push_str(item);
        result.push(processed);
    }
    result
}

// Even better: Use iterator combinators
fn iterator_processing(items: &[String]) -> Vec<String> {
    items.iter()
        .map(|item| format!("processed: {}", item))
        .collect()
}
```

### 2. Debugging Slow Compilation

```bash
# Check what's taking time to compile
cargo build --timings

# Use cargo check for faster feedback
cargo check

# Check if incremental compilation is enabled
export CARGO_INCREMENTAL=1

# Reduce codegen units for release builds (in Cargo.toml)
[profile.release]
codegen-units = 1
```

### 3. Memory Usage Analysis

```bash
# Install memory profiling tools
cargo install cargo-bloat
cargo install cargo-audit

# Analyze binary size
cargo bloat --release
cargo bloat --release --crates

# Check for security vulnerabilities
cargo audit
```

## Runtime Debugging Techniques

### 1. Debug Printing and Logging

```rust
// Use dbg! macro for quick debugging
fn debug_example(data: &[i32]) -> i32 {
    let sum = data.iter().sum::<i32>();
    dbg!(sum);  // Prints: [src/main.rs:123] sum = 42
    
    let average = sum / data.len() as i32;
    dbg!(&average);  // Prints: [src/main.rs:126] &average = 8
    
    average
}

// Conditional debug prints
fn conditional_debug(value: i32) -> i32 {
    #[cfg(debug_assertions)]
    println!("Debug: Processing value {}", value);
    
    value * 2
}

// Using log crate for structured logging
use log::{debug, info, warn, error};

fn logging_example(input: &str) -> Result<String, &'static str> {
    info!("Processing input: {}", input);
    
    if input.is_empty() {
        warn!("Empty input received");
        return Err("Input cannot be empty");
    }
    
    debug!("Input length: {}", input.len());
    
    let result = input.to_uppercase();
    info!("Processing completed successfully");
    
    Ok(result)
}
```

### 2. Using Debug Traits Effectively

```rust
// Custom Debug implementation for selective field printing
use std::fmt;

struct LargeStruct {
    id: u64,
    name: String,
    data: Vec<u8>,  // Large field we don't want to print
    metadata: std::collections::HashMap<String, String>,
}

impl fmt::Debug for LargeStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LargeStruct")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("data_len", &self.data.len())  // Print length instead of data
            .field("metadata_keys", &self.metadata.keys().collect::<Vec<_>>())
            .finish()
    }
}
```

## Error Message Interpretation Guide

### Common Error Patterns

```text
E0382: borrow of moved value
- Solution: Use references, clone values, or restructure ownership

E0106: missing lifetime specifier  
- Solution: Add explicit lifetime parameters or return owned data

E0499: cannot borrow as mutable more than once
- Solution: Limit scope of mutable borrows or use RefCell for interior mutability

E0597: borrowed value does not live long enough
- Solution: Ensure borrowed value outlives the reference or return owned data

E0277: trait bound not satisfied
- Solution: Add required trait bounds or implement missing traits

E0308: mismatched types
- Solution: Convert types, use From/Into traits, or fix type annotations
```

## Performance Profiling

```rust
// Cargo.toml for profiling
[profile.release]
debug = true  # Enable debug symbols for profiling

// Using criterion for benchmarking
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    let data = vec![1; 1000];
    
    c.bench_function("sum", |b| {
        b.iter(|| {
            data.iter().sum::<i32>()
        })
    });
    
    c.bench_function("sum_with_black_box", |b| {
        b.iter(|| {
            black_box(&data).iter().sum::<i32>()
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

## Tools and Commands for Debugging

```bash
# Rust-specific debugging tools
rust-gdb ./target/debug/myapp     # GDB with Rust support
rust-lldb ./target/debug/myapp    # LLDB with Rust support

# Cargo commands for debugging
cargo check                       # Fast syntax check
cargo clippy                      # Linting for common issues
cargo expand                      # Show macro expansions
cargo tree                        # Show dependency tree
cargo outdated                    # Check outdated dependencies

# Memory debugging with Valgrind
valgrind --tool=memcheck ./target/debug/myapp

# Profiling with perf (Linux)
perf record ./target/release/myapp
perf report

# Environment variables for debugging
RUST_BACKTRACE=1 ./myapp          # Show backtrace on panic
RUST_BACKTRACE=full ./myapp       # Full backtrace
RUST_LOG=debug ./myapp            # Enable debug logging
```

## Common Gotchas and Solutions

1. **Integer Overflow in Debug Mode**: Use `wrapping_*` operations or handle overflow explicitly
2. **Slow Debug Builds**: Use `opt-level = 1` in debug profile for better performance
3. **Large Binary Size**: Use `strip = true` and `lto = true` in release profile
4. **Slow Incremental Compilation**: Clean build directory occasionally with `cargo clean`
5. **Dependency Conflicts**: Use `cargo tree --duplicates` to find conflicting versions

## Best Practices for Debugging

- Start with `cargo check` for fast feedback
- Use compiler error messages as learning tools
- Add debug prints strategically, remove them before committing
- Use proper logging levels in production code
- Profile before optimizing
- Keep debug builds fast with appropriate optimization levels
- Use version control to bisect when issues are introduced
