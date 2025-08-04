# Advanced Rust Patterns

## Async/Await and Concurrency Patterns

**Core Principles:**
- Use async/await for I/O-bound operations
- Understand Future trait and execution models
- Leverage Tokio runtime for production async applications
- Handle Send/Sync traits for concurrent programming
- Manage resources carefully in async contexts

**Basic Async/Await Usage:**

```rust
use tokio;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Concurrent execution of multiple async operations
    let task1 = fetch_data_from_api("service1");
    let task2 = fetch_data_from_api("service2");
    let task3 = process_local_data();
    
    // Wait for all tasks to complete
    let (data1, data2, processed) = tokio::try_join!(task1, task2, task3)?;
    
    println!("Results: {:?}, {:?}, {:?}", data1, data2, processed);
    Ok(())
}

async fn fetch_data_from_api(service: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://{}.example.com/data", service);
    let response = reqwest::get(&url).await?;
    let text = response.text().await?;
    Ok(text)
}

async fn process_local_data() -> Result<u32, std::io::Error> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(42)
}
```

**Production Async Server Pattern (from auth-server-template):**

```rust
use tokio::net::{TcpListener, TcpStream};
use hyper::server::conn::http1::Builder as ServerBuilder;
use hyper_util::rt::TokioIo;
use std::pin::Pin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on 0.0.0.0:8080");
    
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("Failed to accept connection: {}", err);
                continue;
            }
        };
        
        // Spawn task for each connection
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream).await {
                eprintln!("Connection error: {}", err);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let io = TokioIo::new(stream);
    
    ServerBuilder::new()
        .serve_connection(io, hyper::service::service_fn(process_request))
        .await?;
    
    Ok(())
}

async fn process_request(
    req: hyper::Request<hyper::body::Incoming>
) -> Result<hyper::Response<String>, std::convert::Infallible> {
    // Process request asynchronously
    Ok(hyper::Response::new("Hello, World!".to_string()))
}
```

**Send and Sync Traits for Concurrency:**

```rust
use std::sync::{Arc, Mutex};
use tokio::task;

// Send: Can be transferred between threads
// Sync: Can be shared between threads (via references)

#[derive(Clone)]
struct SharedCounter {
    value: Arc<Mutex<u32>>,
}

impl SharedCounter {
    fn new() -> Self {
        SharedCounter {
            value: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn increment(&self) -> u32 {
        // Mutex guard is not Send, so we drop it before await
        let new_value = {
            let mut guard = self.value.lock().unwrap();
            *guard += 1;
            *guard
        }; // guard dropped here
        
        // Simulate some async work
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        new_value
    }
}

async fn concurrent_operations() {
    let counter = SharedCounter::new();
    let mut tasks = Vec::new();
    
    // Spawn multiple tasks that share the counter
    for i in 0..10 {
        let counter_clone = counter.clone();
        let task = task::spawn(async move {
            let value = counter_clone.increment().await;
            println!("Task {}: Counter = {}", i, value);
        });
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}
```

**Async Resource Management:**

```rust
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct AsyncResource {
    file: Option<File>,
}

impl AsyncResource {
    async fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(path).await?;
        Ok(AsyncResource { file: Some(file) })
    }
    
    async fn read_data(&mut self) -> Result<Vec<u8>, std::io::Error> {
        if let Some(ref mut file) = self.file {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;
            Ok(buffer)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Resource not available"
            ))
        }
    }
    
    async fn close(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut file) = self.file.take() {
            file.flush().await?;
            // File is automatically closed when dropped
        }
        Ok(())
    }
}

// RAII pattern for async resources
impl Drop for AsyncResource {
    fn drop(&mut self) {
        if self.file.is_some() {
            eprintln!("Warning: AsyncResource dropped without calling close()");
        }
    }
}
```

**Stream Processing:**

```rust
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;

async fn process_data_stream<S>(mut stream: S) -> Result<Vec<String>, Box<dyn std::error::Error>>
where
    S: Stream<Item = Result<String, std::io::Error>> + Unpin,
{
    let mut results = Vec::new();
    
    while let Some(item) = stream.next().await {
        match item {
            Ok(data) => {
                // Process data asynchronously
                let processed = process_single_item(data).await?;
                results.push(processed);
            },
            Err(e) => {
                eprintln!("Stream error: {}", e);
                // Decide whether to continue or abort
                continue;
            }
        }
    }
    
    Ok(results)
}

async fn process_single_item(data: String) -> Result<String, std::io::Error> {
    // Simulate async processing
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    Ok(format!("processed: {}", data))
}
```

**Error Handling in Async Code:**

```rust
use tokio::time::{timeout, Duration};

async fn robust_async_operation() -> Result<String, Box<dyn std::error::Error>> {
    // Timeout for long-running operations
    let result = timeout(Duration::from_secs(30), async {
        // Multiple operations with error propagation
        let data = fetch_external_data().await?;
        let validated = validate_data(data).await?;
        let processed = transform_data(validated).await?;
        Ok::<String, Box<dyn std::error::Error>>(processed)
    }).await;
    
    match result {
        Ok(Ok(data)) => Ok(data),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Operation timed out".into()),
    }
}

async fn fetch_external_data() -> Result<String, Box<dyn std::error::Error>> {
    // Implementation with proper error handling
    Ok("data".to_string())
}

async fn validate_data(data: String) -> Result<String, Box<dyn std::error::Error>> {
    if data.is_empty() {
        return Err("Empty data".into());
    }
    Ok(data)
}

async fn transform_data(data: String) -> Result<String, Box<dyn std::error::Error>> {
    // Transform the data
    Ok(data.to_uppercase())
}
```

## Unsafe Rust and FFI Guidelines

**Core Principles:**
- Use unsafe only when necessary and keep blocks minimal
- Document safety invariants with SAFETY comments
- Wrap unsafe code in safe abstractions
- Understand the five unsafe superpowers
- Use Miri for testing unsafe code

**The Five Unsafe Superpowers:**
1. Dereference raw pointers
2. Call unsafe functions or methods
3. Access or modify mutable static variables
4. Implement unsafe traits
5. Access fields of unions

**Raw Pointer Operations:**

```rust
fn safe_raw_pointer_usage() {
    let mut value = 42;
    
    // Creating raw pointers is safe
    let raw_ptr: *const i32 = &value;
    let raw_mut_ptr: *mut i32 = &mut value;
    
    unsafe {
        // SAFETY: We know these pointers are valid because they were
        // created from valid references in the same function
        println!("Value through const pointer: {}", *raw_ptr);
        *raw_mut_ptr = 100;
        println!("Modified value: {}", *raw_mut_ptr);
    }
}

// Safe abstraction over unsafe code
struct SafeBuffer {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
}

impl SafeBuffer {
    fn new(capacity: usize) -> Self {
        let layout = std::alloc::Layout::array::<u8>(capacity).unwrap();
        let ptr = unsafe {
            // SAFETY: We're allocating with a valid layout
            std::alloc::alloc(layout)
        };
        
        if ptr.is_null() {
            panic!("Failed to allocate memory");
        }
        
        SafeBuffer {
            ptr,
            len: 0,
            capacity,
        }
    }
    
    fn push(&mut self, value: u8) -> Result<(), &'static str> {
        if self.len >= self.capacity {
            return Err("Buffer full");
        }
        
        unsafe {
            // SAFETY: We've checked that len < capacity, so this write
            // is within the allocated memory range
            std::ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
        Ok(())
    }
    
    fn get(&self, index: usize) -> Option<u8> {
        if index >= self.len {
            return None;
        }
        
        unsafe {
            // SAFETY: We've bounds-checked the index
            Some(std::ptr::read(self.ptr.add(index)))
        }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                // SAFETY: ptr was allocated with the same layout in new()
                let layout = std::alloc::Layout::array::<u8>(self.capacity).unwrap();
                std::alloc::dealloc(self.ptr, layout);
            }
        }
    }
}
```

**FFI and C Interoperability:**

```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// External C function declarations
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

// Safe wrapper for C string functions
fn safe_strlen(s: &str) -> usize {
    let c_string = CString::new(s).expect("String contains null byte");
    unsafe {
        // SAFETY: c_string.as_ptr() returns a valid null-terminated C string
        strlen(c_string.as_ptr())
    }
}

// Exposing Rust functions to C
#[no_mangle]
pub extern "C" fn rust_add(a: c_int, b: c_int) -> c_int {
    a + b
}

#[no_mangle]
pub extern "C" fn rust_process_string(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    
    let c_str = unsafe {
        // SAFETY: Caller guarantees input is a valid null-terminated C string
        CStr::from_ptr(input)
    };
    
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    
    let processed = format!("Processed: {}", rust_str);
    
    match CString::new(processed) {
        Ok(c_string) => c_string.into_raw(), // Caller must free this
        Err(_) => std::ptr::null_mut(),
    }
}

// Free function for C callers
#[no_mangle]
pub extern "C" fn rust_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            // SAFETY: s was allocated by rust_process_string
            let _ = CString::from_raw(s);
        }
    }
}
```

**WASM FFI Pattern (from temperature example):**

```rust
// WASM-specific unsafe patterns for data flow graphs
use tinykube_wasm_sdk::macros::map_operator;

#[map_operator]
fn process_wasm_data(input: DataModel) -> DataModel {
    let DataModel::Message(mut message) = input else {
        // In WASM context, panic behavior is controlled
        panic!("Unexpected input type");
    };
    
    // Extract payload with proper buffer handling
    let payload = &message.payload.read();
    
    // Safe JSON processing
    let measurement: Measurement = match serde_json::from_slice(payload) {
        Ok(m) => m,
        Err(e) => {
            // Log error and return original message
            logger::log(Level::Error, "processor", 
                &format!("JSON parsing failed: {}", e));
            return DataModel::Message(message);
        }
    };
    
    // Process safely
    if let Measurement::Temperature(mut temp) = measurement {
        temp.value = temp.value.map(|v| (v - 32.0) * 5.0 / 9.0);
        temp.unit = MeasurementTemperatureUnit::Celsius;
        
        let processed_payload = serde_json::to_vec(&Measurement::Temperature(temp))
            .unwrap_or_else(|_| payload.clone());
        
        message.payload = BufferOrBytes::Bytes(processed_payload);
    }
    
    DataModel::Message(message)
}
```

**Mutable Static Variables:**

```rust
use std::sync::OnceLock;

// Prefer OnceLock over mutable statics
static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

struct Config {
    max_connections: usize,
    timeout_ms: u64,
}

fn get_config() -> &'static Config {
    GLOBAL_CONFIG.get_or_init(|| Config {
        max_connections: 100,
        timeout_ms: 5000,
    })
}

// If mutable static is absolutely necessary
static mut COUNTER: u64 = 0;

unsafe fn increment_counter() -> u64 {
    // SAFETY: This function is marked unsafe to indicate that the caller
    // must ensure this is called from a single thread or properly synchronized
    COUNTER += 1;
    COUNTER
}

// Safer alternative with proper synchronization
use std::sync::atomic::{AtomicU64, Ordering};
static SAFE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn safe_increment_counter() -> u64 {
    SAFE_COUNTER.fetch_add(1, Ordering::Relaxed)
}
```

**Unsafe Trait Implementation:**

```rust
// Unsafe trait - implementer guarantees certain properties
unsafe trait TrustedProcessor {
    fn process_trusted(&self, data: &[u8]) -> Vec<u8>;
}

// Implementation must uphold the safety contract
struct VerifiedProcessor;

unsafe impl TrustedProcessor for VerifiedProcessor {
    fn process_trusted(&self, data: &[u8]) -> Vec<u8> {
        // Implementation that upholds the safety guarantees
        // of the TrustedProcessor trait
        data.to_vec()
    }
}
```

**Testing Unsafe Code with Miri:**

```bash
# Install Miri
rustup +nightly component add miri

# Run tests with Miri
cargo +nightly miri test

# Run specific unsafe code with Miri
cargo +nightly miri run --bin unsafe_example
```

**Safety Guidelines:**
- Always document unsafe code with SAFETY comments
- Keep unsafe blocks as small as possible
- Provide safe abstractions over unsafe code
- Use tools like Miri to validate unsafe code
- Prefer safe alternatives when possible (AtomicU64 vs mutable static)
- Be extra careful with raw pointers and ensure proper bounds checking

## Performance Optimization Patterns

**Core Principles:**
- Zero-cost abstractions: high-level code compiles to efficient machine code
- Minimize heap allocations in hot paths
- Use appropriate data structures for access patterns
- Profile before optimizing and measure the impact
- Leverage Rust's ownership system for memory efficiency

**Memory Allocation Optimization:**

```rust
// Avoid unnecessary allocations
fn inefficient_string_processing(items: &[&str]) -> String {
    let mut result = String::new();
    for item in items {
        // Each push_str may cause reallocation
        result.push_str(item);
        result.push_str(", ");
    }
    result
}

fn efficient_string_processing(items: &[&str]) -> String {
    // Pre-calculate capacity to avoid reallocations
    let total_len: usize = items.iter().map(|s| s.len() + 2).sum();
    let mut result = String::with_capacity(total_len);
    
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(item);
    }
    result
}

// Even better: use iterators and collect
fn iterator_string_processing(items: &[&str]) -> String {
    items.join(", ")
}
```

**Stack vs Heap Allocation:**

```rust
// Stack allocation for known sizes
const BUFFER_SIZE: usize = 1024;

fn stack_buffer_processing(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.len() > BUFFER_SIZE {
        return Err("Data too large for stack buffer");
    }
    
    // Stack-allocated array
    let mut buffer = [0u8; BUFFER_SIZE];
    buffer[..data.len()].copy_from_slice(data);
    
    // Process in-place on stack
    for byte in &mut buffer[..data.len()] {
        *byte = byte.wrapping_add(1);
    }
    
    Ok(buffer[..data.len()].to_vec())
}

// Use SmallVec for small vectors that might not need heap allocation
use smallvec::{SmallVec, smallvec};

fn small_vec_optimization() -> SmallVec<[u32; 8]> {
    // If <= 8 elements, stored on stack
    // If > 8 elements, automatically moves to heap
    smallvec![1, 2, 3, 4, 5]
}
```

**Zero-Cost Abstractions:**

```rust
// Iterator chains compile to efficient loops
fn iterator_optimization(numbers: &[i32]) -> i32 {
    numbers
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .fold(0, |acc, x| acc + x)
}

// Equivalent hand-optimized loop
fn manual_loop(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &num in numbers {
        if num > 0 {
            sum += num * 2;
        }
    }
    sum
}

// Both compile to nearly identical assembly code
```

**Hot Path Optimization (from WASM examples):**

```rust
use tinykube_wasm_sdk::logger::{self, Level};

// Optimized data processing for WASM operators
#[map_operator]
fn optimized_temperature_processor(input: DataModel) -> DataModel {
    let DataModel::Message(mut message) = input else {
        return input; // Early return avoids processing
    };
    
    // Minimize allocations by working with slices
    let payload_slice = message.payload.read();
    
    // Use serde_json's zero-copy deserialization when possible
    let measurement: Measurement = match serde_json::from_slice(&payload_slice) {
        Ok(m) => m,
        Err(_) => return DataModel::Message(message), // Fast error path
    };
    
    // In-place processing when possible
    if let Measurement::Temperature(mut temp) = measurement {
        // Avoid floating point operations if not needed
        if temp.unit == MeasurementTemperatureUnit::Fahrenheit {
            if let Some(value) = temp.value {
                // Single calculation, stored once
                temp.value = Some((value - 32.0) * 5.0 / 9.0);
                temp.unit = MeasurementTemperatureUnit::Celsius;
                
                // Reuse buffer if possible
                if let Ok(new_payload) = serde_json::to_vec(&Measurement::Temperature(temp)) {
                    message.payload = BufferOrBytes::Bytes(new_payload);
                }
            }
        }
    }
    
    DataModel::Message(message)
}
```

**Data Structure Selection:**

```rust
use std::collections::{HashMap, BTreeMap, VecDeque};
use indexmap::IndexMap;

fn choose_right_data_structure() {
    // HashMap: Fast random access, no ordering
    let mut cache: HashMap<String, i32> = HashMap::new();
    
    // BTreeMap: Ordered keys, range queries
    let mut sorted_data: BTreeMap<i32, String> = BTreeMap::new();
    
    // IndexMap: Insertion order preserved, fast access
    let mut ordered_cache: IndexMap<String, i32> = IndexMap::new();
    
    // VecDeque: Efficient push/pop from both ends
    let mut buffer: VecDeque<u8> = VecDeque::new();
    
    // Vec: Best for append-only, sequential access
    let mut sequential_data: Vec<i32> = Vec::new();
}

// Specialized data structures for performance
use std::collections::BinaryHeap;

struct PriorityQueue<T> {
    heap: BinaryHeap<(u32, T)>, // (priority, item)
}

impl<T> PriorityQueue<T> {
    fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
        }
    }
    
    fn push(&mut self, priority: u32, item: T) {
        self.heap.push((priority, item));
    }
    
    fn pop(&mut self) -> Option<T> {
        self.heap.pop().map(|(_, item)| item)
    }
}
```

**Profiling and Benchmarking:**

```rust
// Cargo.toml
// [dev-dependencies]
// criterion = "0.5"

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_string_processing(c: &mut Criterion) {
    let items = vec!["hello", "world", "rust", "performance"];
    
    c.bench_function("inefficient", |b| {
        b.iter(|| inefficient_string_processing(black_box(&items)))
    });
    
    c.bench_function("efficient", |b| {
        b.iter(|| efficient_string_processing(black_box(&items)))
    });
    
    c.bench_function("iterator", |b| {
        b.iter(|| iterator_string_processing(black_box(&items)))
    });
}

criterion_group!(benches, benchmark_string_processing);
criterion_main!(benches);
```

**Compile-Time Optimizations:**

```rust
// Cargo.toml optimizations
/*
[profile.release]
opt-level = 3          # Maximum optimization
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization, slower compile
panic = "abort"       # Smaller binary, no unwinding
*/

// Use const evaluation for compile-time computation
const fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// Computed at compile time
const FIB_10: u32 = fibonacci(10);

// Inline hints for hot functions
#[inline(always)]
fn critical_function(x: i32) -> i32 {
    x * 2 + 1
}

#[inline(never)]
fn cold_function() {
    // Function that shouldn't be inlined
}
```

**SIMD and Vectorization:**

```rust
// Explicit SIMD for performance-critical code
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn vectorized_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len());
    let mut result = vec![0.0; a.len()];
    
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        return unsafe { vectorized_add_avx2(a, b) };
    }
    
    // Fallback scalar implementation
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
    
    result
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn vectorized_add_avx2(a: &[f32], b: &[f32]) -> Vec<f32> {
    let mut result = vec![0.0; a.len()];
    let chunks = a.len() / 8;
    
    for i in 0..chunks {
        let offset = i * 8;
        let va = _mm256_loadu_ps(a.as_ptr().add(offset));
        let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
        let vr = _mm256_add_ps(va, vb);
        _mm256_storeu_ps(result.as_mut_ptr().add(offset), vr);
    }
    
    // Handle remaining elements
    for i in (chunks * 8)..a.len() {
        result[i] = a[i] + b[i];
    }
    
    result
}
```

**Performance Guidelines:**
- Profile with `perf`, `valgrind`, or `criterion` before optimizing
- Use `--release` flag for performance testing
- Prefer stack allocation for small, fixed-size data
- Choose appropriate collection types for access patterns
- Use iterators instead of index-based loops
- Minimize string allocations in hot paths
- Consider SIMD for numerical computations
- Use `black_box()` in benchmarks to prevent over-optimization
