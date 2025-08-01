---
applyTo: '**/*.rs'
---

# Rust Development Instructions

You are an expert Rust developer with comprehensive knowledge of Rust fundamentals, advanced patterns, WASM development, and Azure IoT Operations integration. You excel at managing the developer inner loop including tools, testing, debugging, and performance optimization.

## Implementation Requirements

When implementing any Rust-related functionality:

- You must follow all Rust ownership, borrowing, and memory safety principles
- You must implement error handling using `Result<T, E>` patterns and avoid panic-inducing code
- You must adhere to the performance and safety guidelines provided
- You must use appropriate async/await patterns for concurrent programming
- You must implement complete, working functionality that follows established project patterns

## Purpose

This document provides comprehensive prompt instructions for Rust implementation that ensure consistency, proper adherence to Rust principles, memory safety, performance optimization, and alignment with Azure IoT Operations patterns.

## Rust Fundamentals

### Ownership System and Memory Safety

**Core Principles:**
- **Ownership**: Each value has exactly one owner at a time
- **Borrowing**: References allow access without taking ownership
- **Lifetimes**: Ensure references are valid for as long as needed
- **Move semantics**: Values are moved by default, preventing double-free errors

**Ownership Rules:**
1. Each value in Rust has an owner
2. There can only be one owner at a time
3. When the owner goes out of scope, the value is dropped

**Memory Management Patterns:**

```rust
// Ownership transfer (move)
let s1 = String::from("hello");
let s2 = s1; // s1 is moved to s2, s1 is no longer valid

// Borrowing with immutable references
let s1 = String::from("hello");
let len = calculate_length(&s1); // s1 is borrowed, still valid after call

fn calculate_length(s: &String) -> usize {
    s.len()
} // s goes out of scope but doesn't drop the String (it was borrowed)

// Borrowing with mutable references
let mut s = String::from("hello");
change(&mut s);

fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
```

**Stack vs Heap Data:**

```rust
// Stack data (Copy trait) - cheap to copy
let x = 5;
let y = x; // x is still valid, data copied

// Heap data (no Copy trait) - expensive to copy, moved by default
let s1 = String::from("hello");
let s2 = s1; // s1 moved to s2

// Explicit cloning for heap data when needed
let s1 = String::from("hello");
let s2 = s1.clone(); // Both s1 and s2 are valid, data copied
```

**RAII and Drop Semantics:**

```rust
{
    let s = String::from("hello"); // s comes into scope
    // s is valid here
} // s goes out of scope, drop is called, memory freed

// Custom drop implementation
struct CustomResource {
    data: String,
}

impl Drop for CustomResource {
    fn drop(&mut self) {
        println!("Cleaning up {}", self.data);
    }
}
```

**Borrowing Rules:**
- You can have either one mutable reference OR any number of immutable references
- References must always be valid (no dangling references)
- The borrow checker prevents data races at compile time

```rust
let mut s = String::from("hello");

let r1 = &s; // no problem
let r2 = &s; // no problem
// let r3 = &mut s; // BIG PROBLEM - cannot have mutable reference while immutable references exist

println!("{r1} and {r2}");
// r1 and r2 no longer used after this point

let r3 = &mut s; // no problem - immutable references are out of scope
println!("{r3}");
```

### Error Handling and Result Patterns  

**Core Principles:**
- Use `Result<T, E>` for recoverable errors
- Use `Option<T>` for nullable values  
- Avoid `panic!` in production code unless truly exceptional
- Propagate errors using the `?` operator
- Provide meaningful error context

**Result Type Usage:**

```rust
use std::fs::File;
use std::io::{self, Read};

// Basic Result usage
fn read_file_contents(filename: &str) -> Result<String, io::Error> {
    let mut file = File::open(filename)?; // ? propagates error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Handling Result in calling code
fn main() {
    match read_file_contents("hello.txt") {
        Ok(contents) => println!("File contents: {}", contents),
        Err(error) => eprintln!("Error reading file: {}", error),
    }
}
```

**Error Propagation with ? Operator:**

```rust
// Chain operations with error propagation
fn process_data() -> Result<ProcessedData, ProcessingError> {
    let raw_data = fetch_data()?;           // Propagate fetch errors
    let validated = validate_data(raw_data)?; // Propagate validation errors
    let processed = transform_data(validated)?; // Propagate transformation errors
    Ok(processed)
}
```

**Custom Error Types:**

```rust
use std::fmt;

#[derive(Debug)]
enum DataProcessingError {
    InvalidFormat(String),
    NetworkError(std::io::Error),
    ValidationFailed { field: String, reason: String },
}

impl fmt::Display for DataProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataProcessingError::InvalidFormat(msg) => 
                write!(f, "Invalid data format: {}", msg),
            DataProcessingError::NetworkError(err) => 
                write!(f, "Network error: {}", err),
            DataProcessingError::ValidationFailed { field, reason } => 
                write!(f, "Validation failed for {}: {}", field, reason),
        }
    }
}

impl std::error::Error for DataProcessingError {}

// Convert from other error types
impl From<std::io::Error> for DataProcessingError {
    fn from(error: std::io::Error) -> Self {
        DataProcessingError::NetworkError(error)
    }
}
```

**Option Type Patterns:**

```rust
// Option for nullable values
fn find_user(id: u32) -> Option<User> {
    // Return Some(user) if found, None if not found
}

// Option combinators
let user_name = find_user(123)
    .map(|user| user.name)
    .unwrap_or_else(|| "Unknown".to_string());

// Option with ?
fn get_user_email(id: u32) -> Option<String> {
    let user = find_user(id)?; // Return None if user not found
    Some(user.email)
}
```

**Production Error Handling Pattern (from auth-server-template):**

```rust
use tinykube_wasm_sdk::logger::{self, Level};

fn safe_processor(input: DataModel) -> DataModel {
    match process_data_safely(input.clone()) {
        Ok(result) => result,
        Err(e) => {
            // Log error without exposing internal details
            logger::log(Level::Error, "processor", 
                &format!("Processing failed: {}", e));
            // Return original data as fallback
            input
        }
    }
}

fn process_data_safely(input: DataModel) -> Result<DataModel, ProcessingError> {
    // Validate input
    let validated = validate_input(&input)
        .map_err(|e| ProcessingError::InvalidInput(e.to_string()))?;
    
    // Process with error context
    let result = perform_transformation(validated)
        .map_err(|e| ProcessingError::TransformationFailed(e.to_string()))?;
    
    Ok(result)
}
```

**No-Panic Production Guidelines:**
- Always handle `Result` and `Option` explicitly
- Use `unwrap_or`, `unwrap_or_else`, `unwrap_or_default` instead of `unwrap()`
- Use `expect()` only with descriptive messages for truly exceptional cases
- Validate inputs at boundaries
- Use defensive programming patterns

### Type System and Traits

**Core Principles:**
- Use Rust's type system to prevent bugs at compile time
- Implement traits to provide shared behavior
- Use generics for code reuse without runtime cost
- Leverage associated types for cleaner APIs
- Understand Copy vs Clone semantics

**Generic Programming:**

```rust
// Generic function
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut largest = list[0];
    for &item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// Generic struct
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Implementation for specific types
impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

**Trait Definitions and Implementation:**

```rust
// Define a trait
trait Processor {
    type Output; // Associated type
    type Error;
    
    fn process(&self, input: &[u8]) -> Result<Self::Output, Self::Error>;
    
    // Default implementation
    fn name(&self) -> &str {
        "generic_processor"
    }
}

// Implement trait for concrete type
struct TemperatureProcessor {
    conversion_factor: f64,
}

impl Processor for TemperatureProcessor {
    type Output = f64;
    type Error = String;
    
    fn process(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let raw_value = std::str::from_utf8(input)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?
            .parse::<f64>()
            .map_err(|e| format!("Invalid number: {}", e))?;
        
        Ok(raw_value * self.conversion_factor)
    }
    
    fn name(&self) -> &str {
        "temperature_processor"
    }
}
```

**Trait Bounds and Where Clauses:**

```rust
// Multiple trait bounds
fn process_and_display<T>(item: T) -> String 
where
    T: Processor + Clone + std::fmt::Debug,
    T::Output: std::fmt::Display,
{
    let cloned = item.clone();
    match cloned.process(&[]) {
        Ok(output) => format!("Result: {}", output),
        Err(e) => format!("Error: {:?}", e),
    }
}

// Trait bounds in function parameters
fn compare_processors<T, U>(p1: &T, p2: &U) -> bool
where
    T: Processor,
    U: Processor,
    T::Output: PartialEq<U::Output>,
{
    // Implementation would compare processors
    true
}
```

**Copy vs Clone Semantics:**

```rust
// Copy trait - automatic bitwise copy, cheap
#[derive(Copy, Clone, Debug)]
struct Point2D {
    x: i32,
    y: i32,
}

// Clone trait - explicit copying, potentially expensive
#[derive(Clone, Debug)]
struct ComplexData {
    name: String,
    values: Vec<i32>,
}

fn demonstrate_copy_clone() {
    // Copy semantics
    let p1 = Point2D { x: 1, y: 2 };
    let p2 = p1; // Automatic copy, p1 still valid
    println!("p1: {:?}, p2: {:?}", p1, p2);
    
    // Clone semantics
    let data1 = ComplexData {
        name: "test".to_string(),
        values: vec![1, 2, 3],
    };
    let data2 = data1.clone(); // Explicit clone required
    // data1 is still valid after clone
    println!("data1: {:?}, data2: {:?}", data1, data2);
}
```

**Associated Types vs Generic Parameters:**

```rust
// Associated types - one implementation per type
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Generic parameters - multiple implementations possible
trait From<T> {
    fn from(value: T) -> Self;
}

// Can implement From for multiple types
impl From<i32> for String {
    fn from(value: i32) -> Self {
        value.to_string()
    }
}

impl From<f64> for String {
    fn from(value: f64) -> Self {
        value.to_string()
    }
}
```

**Trait Objects for Dynamic Dispatch:**

```rust
// Trait object for runtime polymorphism
trait Drawable {
    fn draw(&self);
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

fn draw_shapes(shapes: Vec<Box<dyn Drawable>>) {
    for shape in shapes {
        shape.draw(); // Dynamic dispatch
    }
}
```

**Lifetime Parameters:**

```rust
// Lifetime annotations ensure references are valid
struct DataRef<'a> {
    value: &'a str,
}

impl<'a> DataRef<'a> {
    fn new(value: &'a str) -> Self {
        DataRef { value }
    }
    
    fn get_value(&self) -> &str {
        self.value // Same lifetime as input
    }
}

// Function with lifetime parameters
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

## Advanced Rust Patterns

### Async/Await and Concurrency Patterns

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

### Unsafe Rust and FFI Guidelines

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

### Performance Optimization Patterns

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

## WASM and IoT-Specific Patterns

### WASM Development Guidelines

[TODO: Add WASM compilation and development patterns]

### Azure IoT Operations SDK Integration

[TODO: Add SDK usage patterns]

### Data Flow Operator Patterns

[TODO: Add operator implementation patterns]

## Developer Inner Loop

### Build and Dependency Management

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

### Testing and Debugging Guidelines

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

### Development Workflow and Tooling

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

## Code Examples and Best Practices

### Comprehensive Code Examples

**Core Principles:**
- Provide complete, working examples for common patterns
- Show both basic and advanced usage scenarios
- Include error handling and edge cases
- Demonstrate performance considerations
- Follow established Rust conventions

**Complete HTTP Client Example:**

```rust
// Cargo.toml
// [dependencies]
// tokio = { version = "1.0", features = ["full"] }
// reqwest = { version = "0.11", features = ["json"] }
// serde = { version = "1.0", features = ["derive"] }
// anyhow = "1.0"

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    status: String,
    data: Option<serde_json::Value>,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    action: String,
    payload: serde_json::Value,
}

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(ApiClient {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send GET request")?;

        self.handle_response(response).await
    }

    pub async fn post<T, U>(&self, endpoint: &str, data: &T) -> Result<U>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(data)
            .send()
            .await
            .context("Failed to send POST request")?;

        self.handle_response(response).await
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status {}: {}",
                status,
                text
            ));
        }

        serde_json::from_str(&text)
            .context("Failed to parse JSON response")
    }
}

// Usage example
#[tokio::main]
async fn main() -> Result<()> {
    let client = ApiClient::new(
        "https://api.example.com".to_string(),
        "your-api-key".to_string(),
    )?;

    // GET request
    let user_data: serde_json::Value = client
        .get("users/123")
        .await
        .context("Failed to fetch user data")?;

    println!("User data: {:#?}", user_data);

    // POST request
    let request = ApiRequest {
        action: "create_user".to_string(),
        payload: serde_json::json!({
            "name": "John Doe",
            "email": "john@example.com"
        }),
    };

    let response: ApiResponse = client
        .post("users", &request)
        .await
        .context("Failed to create user")?;

    println!("Create user response: {:#?}", response);

    Ok(())
}
```

**Configuration Management Pattern:**

```rust
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: u32,
    pub request_timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub log_level: String,
    pub environment: Environment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
                request_timeout_ms: 30000,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "user".to_string(),
                password: "password".to_string(),
                database: "myapp".to_string(),
                max_connections: 10,
            },
            log_level: "info".to_string(),
            environment: Environment::Development,
        }
    }
}

impl AppConfig {
    /// Load configuration from multiple sources with precedence:
    /// 1. Environment variables (highest)
    /// 2. Config file
    /// 3. Default values (lowest)
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        // Load from config file if it exists
        if let Ok(config_path) = env::var("CONFIG_FILE") {
            if Path::new(&config_path).exists() {
                config = Self::from_file(&config_path)?;
            }
        } else if Path::new("config.toml").exists() {
            config = Self::from_file("config.toml")?;
        }

        // Override with environment variables
        config.apply_env_overrides()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path))?;

        toml::from_str(&content)
            .context("Failed to parse config file")
    }

    fn apply_env_overrides(&mut self) -> Result<()> {
        // Server configuration
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            self.server.port = port.parse()
                .context("Invalid SERVER_PORT")?;
        }

        // Database configuration
        if let Ok(db_host) = env::var("DATABASE_HOST") {
            self.database.host = db_host;
        }
        if let Ok(db_port) = env::var("DATABASE_PORT") {
            self.database.port = db_port.parse()
                .context("Invalid DATABASE_PORT")?;
        }
        if let Ok(db_user) = env::var("DATABASE_USERNAME") {
            self.database.username = db_user;
        }
        if let Ok(db_pass) = env::var("DATABASE_PASSWORD") {
            self.database.password = db_pass;
        }

        // Environment
        if let Ok(env_str) = env::var("ENVIRONMENT") {
            self.environment = serde_json::from_str(&format!("\"{}\"", env_str.to_lowercase()))
                .context("Invalid ENVIRONMENT value")?;
        }

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }
        if self.database.port == 0 {
            return Err(anyhow::anyhow!("Database port cannot be 0"));
        }
        if self.database.username.is_empty() {
            return Err(anyhow::anyhow!("Database username cannot be empty"));
        }
        if self.server.workers == 0 {
            return Err(anyhow::anyhow!("Server workers must be > 0"));
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }
}
```

**Generic Data Processing Pipeline:**

```rust
use std::marker::PhantomData;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc;
use anyhow::Result;

// Trait for processing steps
#[async_trait]
pub trait Processor<T, U>: Send + Sync {
    async fn process(&self, input: T) -> Result<U>;
}

// Generic pipeline that chains processors
pub struct Pipeline<T> {
    _phantom: PhantomData<T>,
}

impl<T> Pipeline<T>
where
    T: Send + 'static,
{
    pub fn new() -> Self {
        Pipeline {
            _phantom: PhantomData,
        }
    }

    pub async fn process_stream<U, P>(
        &self,
        mut input_rx: mpsc::Receiver<T>,
        processor: Arc<P>,
        output_tx: mpsc::Sender<U>,
    ) -> Result<()>
    where
        U: Send + 'static,
        P: Processor<T, U> + 'static,
    {
        while let Some(item) = input_rx.recv().await {
            match processor.process(item).await {
                Ok(result) => {
                    if output_tx.send(result).await.is_err() {
                        break; // Receiver dropped
                    }
                }
                Err(e) => {
                    eprintln!("Processing error: {}", e);
                    // Continue processing other items
                }
            }
        }
        Ok(())
    }
}

// Example processors
pub struct StringToUppercase;

#[async_trait]
impl Processor<String, String> for StringToUppercase {
    async fn process(&self, input: String) -> Result<String> {
        Ok(input.to_uppercase())
    }
}

pub struct NumberDoubler;

#[async_trait]
impl Processor<i32, i32> for NumberDoubler {
    async fn process(&self, input: i32) -> Result<i32> {
        Ok(input * 2)
    }
}

// Usage example
async fn pipeline_example() -> Result<()> {
    let (input_tx, input_rx) = mpsc::channel::<String>(100);
    let (output_tx, mut output_rx) = mpsc::channel::<String>(100);

    let pipeline = Pipeline::new();
    let processor = Arc::new(StringToUppercase);

    // Start processing pipeline
    let processing_task = tokio::spawn(async move {
        pipeline.process_stream(input_rx, processor, output_tx).await
    });

    // Send data
    input_tx.send("hello".to_string()).await?;
    input_tx.send("world".to_string()).await?;
    drop(input_tx); // Close input

    // Receive results
    while let Some(result) = output_rx.recv().await {
        println!("Processed: {}", result);
    }

    processing_task.await??;
    Ok(())
}
```

**Error Handling with Custom Error Types:**

```rust
use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {source}")]
    Database {
        #[from]
        source: sqlx::Error,
    },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Validation error in field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn validation(field: &str, message: &str) -> Self {
        AppError::Validation {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound {
            resource: resource.to_string(),
        }
    }

    pub fn network(message: &str) -> Self {
        AppError::Network {
            message: message.to_string(),
        }
    }

    pub fn permission_denied(action: &str) -> Self {
        AppError::PermissionDenied {
            action: action.to_string(),
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Database { .. } => 500,
            AppError::Network { .. } => 502,
            AppError::Validation { .. } => 400,
            AppError::NotFound { .. } => 404,
            AppError::AuthenticationFailed => 401,
            AppError::PermissionDenied { .. } => 403,
            AppError::Internal(_) => 500,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            AppError::Database { .. } => "A database error occurred".to_string(),
            AppError::Network { .. } => "A network error occurred".to_string(),
            AppError::Validation { field, message } => {
                format!("Validation error in {}: {}", field, message)
            }
            AppError::NotFound { resource } => format!("{} not found", resource),
            AppError::AuthenticationFailed => "Authentication required".to_string(),
            AppError::PermissionDenied { action } => {
                format!("Permission denied for action: {}", action)
            }
            AppError::Internal(_) => "An internal error occurred".to_string(),
        }
    }
}

// Usage in application functions
type AppResult<T> = Result<T, AppError>;

pub struct UserService {
    // ... fields
}

impl UserService {
    pub async fn get_user(&self, id: u64) -> AppResult<User> {
        // Validate input
        if id == 0 {
            return Err(AppError::validation("id", "ID must be greater than 0"));
        }

        // Fetch from database
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    pub async fn create_user(&self, data: CreateUserRequest) -> AppResult<User> {
        // Validate request
        self.validate_create_request(&data)?;

        // Check if user already exists
        if self.user_exists(&data.email).await? {
            return Err(AppError::validation("email", "Email already exists"));
        }

        // Create user
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, name) VALUES ($1, $2) RETURNING *",
            data.email,
            data.name
        )
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    fn validate_create_request(&self, data: &CreateUserRequest) -> AppResult<()> {
        if data.email.is_empty() {
            return Err(AppError::validation("email", "Email is required"));
        }
        if !data.email.contains('@') {
            return Err(AppError::validation("email", "Invalid email format"));
        }
        if data.name.is_empty() {
            return Err(AppError::validation("name", "Name is required"));
        }
        Ok(())
    }

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE email = $1",
            email
        )
        .fetch_one(&self.db)
        .await?;

        Ok(count > 0)
    }
}
```

**Resource Management with RAII:**

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

pub struct Resource {
    id: String,
    data: Vec<u8>,
}

impl Resource {
    fn new(id: String, size: usize) -> Self {
        Resource {
            id,
            data: vec![0; size],
        }
    }

    fn cleanup(&mut self) {
        println!("Cleaning up resource: {}", self.id);
        self.data.clear();
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// Resource pool for managing multiple resources
pub struct ResourcePool {
    resources: Mutex<Vec<Resource>>,
    max_size: usize,
}

impl ResourcePool {
    pub fn new(max_size: usize) -> Arc<Self> {
        Arc::new(ResourcePool {
            resources: Mutex::new(Vec::new()),
            max_size,
        })
    }

    pub async fn acquire(&self, id: String) -> Result<ResourceGuard> {
        let mut resources = self.resources.lock().await;
        
        if resources.len() < self.max_size {
            let resource = Resource::new(id.clone(), 1024);
            resources.push(resource);
            
            let index = resources.len() - 1;
            Ok(ResourceGuard {
                pool: self as *const ResourcePool,
                index,
                id,
            })
        } else {
            Err(anyhow::anyhow!("Resource pool exhausted"))
        }
    }

    async fn release(&self, index: usize) {
        let mut resources = self.resources.lock().await;
        if index < resources.len() {
            resources.remove(index);
        }
    }
}

// RAII guard for automatic resource cleanup
pub struct ResourceGuard {
    pool: *const ResourcePool,
    index: usize,
    id: String,
}

impl ResourceGuard {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn process_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let mut result = data.to_vec();
        result.reverse();
        Ok(result)
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        // Note: In real code, you'd want to use a channel or other async mechanism
        // for cleanup since Drop is not async
        println!("Releasing resource: {}", self.id);
    }
}

// Usage example
async fn resource_example() -> Result<()> {
    let pool = ResourcePool::new(5);
    
    {
        let resource = pool.acquire("test-resource-1".to_string()).await?;
        let result = resource.process_data(b"hello world").await?;
        println!("Processed: {:?}", String::from_utf8_lossy(&result));
        
        // Resource automatically released when guard goes out of scope
    }
    
    Ok(())
}
```

### Architectural Guidelines

**Core Principles:**
- Design for modularity and composability
- Separate concerns with clear boundaries
- Use appropriate abstraction levels
- Follow dependency inversion principle
- Design APIs that are hard to misuse

**Module Organization Patterns:**

```rust
// src/lib.rs - Library root
//! # MyApp Library
//! 
//! This library provides core functionality for the MyApp application.

pub mod config;
pub mod database;
pub mod services;
pub mod models;
pub mod errors;
pub mod utils;

// Re-export commonly used types
pub use errors::{AppError, AppResult};
pub use config::AppConfig;

// Prelude module for common imports
pub mod prelude {
    pub use crate::{AppError, AppResult, AppConfig};
    pub use crate::services::UserService;
    pub use crate::models::{User, CreateUserRequest};
}

// src/models/mod.rs - Data models
pub mod user;
pub mod organization;
pub mod common;

pub use user::{User, CreateUserRequest, UpdateUserRequest};
pub use organization::{Organization, CreateOrgRequest};
pub use common::{Id, Timestamp, Pagination};

// src/services/mod.rs - Business logic
mod user_service;
mod auth_service;
mod notification_service;

pub use user_service::UserService;
pub use auth_service::AuthService;
pub use notification_service::NotificationService;

// Create a service container
pub struct Services {
    pub user: UserService,
    pub auth: AuthService,
    pub notification: NotificationService,
}

impl Services {
    pub async fn new(config: &AppConfig, db: sqlx::Pool<sqlx::Postgres>) -> AppResult<Self> {
        Ok(Services {
            user: UserService::new(db.clone()).await?,
            auth: AuthService::new(config.auth.clone(), db.clone()).await?,
            notification: NotificationService::new(config.notification.clone()).await?,
        })
    }
}
```

**Layered Architecture Pattern:**

```rust
// Domain layer - Core business logic
pub mod domain {
    use async_trait::async_trait;
    use crate::errors::AppResult;

    // Domain entities
    #[derive(Debug, Clone)]
    pub struct User {
        pub id: UserId,
        pub email: String,
        pub name: String,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserId(pub u64);

    // Domain services (business logic)
    pub struct UserDomainService {
        repository: Box<dyn UserRepository>,
        email_service: Box<dyn EmailService>,
    }

    impl UserDomainService {
        pub fn new(
            repository: Box<dyn UserRepository>,
            email_service: Box<dyn EmailService>,
        ) -> Self {
            UserDomainService {
                repository,
                email_service,
            }
        }

        pub async fn create_user(&self, email: String, name: String) -> AppResult<User> {
            // Business logic validation
            if !self.is_valid_email(&email) {
                return Err(AppError::validation("email", "Invalid email format"));
            }

            // Check business rules
            if self.repository.exists_by_email(&email).await? {
                return Err(AppError::validation("email", "Email already exists"));
            }

            // Create user
            let user = User {
                id: UserId(0), // Will be set by repository
                email: email.clone(),
                name,
                created_at: chrono::Utc::now(),
            };

            let saved_user = self.repository.save(user).await?;

            // Send welcome email (side effect)
            self.email_service.send_welcome_email(&saved_user).await?;

            Ok(saved_user)
        }

        fn is_valid_email(&self, email: &str) -> bool {
            email.contains('@') && email.len() > 3
        }
    }

    // Repository traits (domain interfaces)
    #[async_trait]
    pub trait UserRepository: Send + Sync {
        async fn save(&self, user: User) -> AppResult<User>;
        async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;
        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
        async fn exists_by_email(&self, email: &str) -> AppResult<bool>;
        async fn delete(&self, id: UserId) -> AppResult<()>;
    }

    #[async_trait]
    pub trait EmailService: Send + Sync {
        async fn send_welcome_email(&self, user: &User) -> AppResult<()>;
        async fn send_password_reset(&self, user: &User, token: &str) -> AppResult<()>;
    }
}

// Infrastructure layer - External dependencies
pub mod infrastructure {
    use async_trait::async_trait;
    use sqlx::{PgPool, FromRow};
    use crate::domain::{User, UserId, UserRepository, EmailService};
    use crate::errors::AppResult;

    // Database implementation
    pub struct PostgresUserRepository {
        pool: PgPool,
    }

    impl PostgresUserRepository {
        pub fn new(pool: PgPool) -> Self {
            PostgresUserRepository { pool }
        }
    }

    #[derive(FromRow)]
    struct UserRow {
        id: i64,
        email: String,
        name: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    impl From<UserRow> for User {
        fn from(row: UserRow) -> Self {
            User {
                id: UserId(row.id as u64),
                email: row.email,
                name: row.name,
                created_at: row.created_at,
            }
        }
    }

    #[async_trait]
    impl UserRepository for PostgresUserRepository {
        async fn save(&self, mut user: User) -> AppResult<User> {
            let row = sqlx::query_as!(
                UserRow,
                "INSERT INTO users (email, name, created_at) VALUES ($1, $2, $3) RETURNING *",
                user.email,
                user.name,
                user.created_at
            )
            .fetch_one(&self.pool)
            .await?;

            Ok(row.into())
        }

        async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
            let row = sqlx::query_as!(
                UserRow,
                "SELECT * FROM users WHERE id = $1",
                id.0 as i64
            )
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(Into::into))
        }

        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
            let row = sqlx::query_as!(
                UserRow,
                "SELECT * FROM users WHERE email = $1",
                email
            )
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(Into::into))
        }

        async fn exists_by_email(&self, email: &str) -> AppResult<bool> {
            let count: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM users WHERE email = $1",
                email
            )
            .fetch_one(&self.pool)
            .await?;

            Ok(count > 0)
        }

        async fn delete(&self, id: UserId) -> AppResult<()> {
            sqlx::query!("DELETE FROM users WHERE id = $1", id.0 as i64)
                .execute(&self.pool)
                .await?;

            Ok(())
        }
    }

    // Email service implementation
    pub struct SmtpEmailService {
        smtp_host: String,
        smtp_port: u16,
        username: String,
        password: String,
    }

    impl SmtpEmailService {
        pub fn new(smtp_host: String, smtp_port: u16, username: String, password: String) -> Self {
            SmtpEmailService {
                smtp_host,
                smtp_port,
                username,
                password,
            }
        }
    }

    #[async_trait]
    impl EmailService for SmtpEmailService {
        async fn send_welcome_email(&self, user: &User) -> AppResult<()> {
            // Implementation would use an SMTP library
            println!("Sending welcome email to: {}", user.email);
            Ok(())
        }

        async fn send_password_reset(&self, user: &User, token: &str) -> AppResult<()> {
            println!("Sending password reset to: {} with token: {}", user.email, token);
            Ok(())
        }
    }
}

// Application layer - Use cases and coordination
pub mod application {
    use crate::domain::{UserDomainService, User, UserId};
    use crate::errors::AppResult;

    pub struct UserApplicationService {
        domain_service: UserDomainService,
    }

    impl UserApplicationService {
        pub fn new(domain_service: UserDomainService) -> Self {
            UserApplicationService { domain_service }
        }

        pub async fn register_user(&self, email: String, name: String) -> AppResult<UserId> {
            let user = self.domain_service.create_user(email, name).await?;
            Ok(user.id)
        }

        pub async fn get_user(&self, id: UserId) -> AppResult<Option<User>> {
            self.domain_service.repository.find_by_id(id).await
        }
    }
}
```

**API Design Patterns:**

```rust
// Builder pattern for complex configuration
pub struct DatabaseConnectionBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    max_connections: Option<u32>,
    timeout: Option<std::time::Duration>,
}

impl DatabaseConnectionBuilder {
    pub fn new() -> Self {
        DatabaseConnectionBuilder {
            host: None,
            port: None,
            username: None,
            password: None,
            database: None,
            max_connections: None,
            timeout: None,
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn connect(self) -> AppResult<DatabaseConnection> {
        let config = DatabaseConfig {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(5432),
            username: self.username.ok_or_else(|| {
                AppError::validation("username", "Username is required")
            })?,
            password: self.password.ok_or_else(|| {
                AppError::validation("password", "Password is required")
            })?,
            database: self.database.ok_or_else(|| {
                AppError::validation("database", "Database name is required")
            })?,
            max_connections: self.max_connections.unwrap_or(10),
            timeout: self.timeout.unwrap_or(std::time::Duration::from_secs(30)),
        };

        DatabaseConnection::new(config).await
    }
}

// Usage:
// let db = DatabaseConnectionBuilder::new()
//     .host("localhost")
//     .port(5432)
//     .credentials("user", "password")
//     .database("myapp")
//     .max_connections(20)
//     .connect()
//     .await?;

// Type-safe query builder
pub struct QueryBuilder<T> {
    table: String,
    conditions: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> QueryBuilder<T> {
    pub fn from_table(table: &str) -> Self {
        QueryBuilder {
            table: table.to_string(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn where_eq(mut self, column: &str, value: &str) -> Self {
        self.conditions.push(format!("{} = '{}'", column, value));
        self
    }

    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        let dir = match direction {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        };
        self.order_by.push(format!("{} {}", column, dir));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(self) -> String {
        let mut query = format!("SELECT * FROM {}", self.table);

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }

        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            query.push_str(&self.order_by.join(", "));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}

pub enum OrderDirection {
    Asc,
    Desc,
}
```

**Dependency Injection Pattern:**

```rust
use std::sync::Arc;

// Container for dependency injection
pub struct ServiceContainer {
    user_service: Arc<UserApplicationService>,
    auth_service: Arc<AuthService>,
    config: Arc<AppConfig>,
}

impl ServiceContainer {
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        let config = Arc::new(config);
        
        // Create database connection
        let db_pool = create_db_pool(&config.database).await?;
        
        // Create repositories
        let user_repo = Box::new(PostgresUserRepository::new(db_pool.clone()));
        let email_service = Box::new(SmtpEmailService::new(
            config.smtp.host.clone(),
            config.smtp.port,
            config.smtp.username.clone(),
            config.smtp.password.clone(),
        ));
        
        // Create domain services
        let user_domain_service = UserDomainService::new(user_repo, email_service);
        
        // Create application services
        let user_service = Arc::new(UserApplicationService::new(user_domain_service));
        let auth_service = Arc::new(AuthService::new(config.clone(), db_pool));
        
        Ok(ServiceContainer {
            user_service,
            auth_service,
            config,
        })
    }

    pub fn user_service(&self) -> Arc<UserApplicationService> {
        self.user_service.clone()
    }

    pub fn auth_service(&self) -> Arc<AuthService> {
        self.auth_service.clone()
    }

    pub fn config(&self) -> Arc<AppConfig> {
        self.config.clone()
    }
}

// Usage in main application
#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::load()?;
    let container = ServiceContainer::new(config).await?;
    
    // Start web server with container
    start_web_server(container).await?;
    
    Ok(())
}
```

**Architectural Best Practices:**
- Keep domain logic independent of infrastructure
- Use dependency injection for testability
- Design APIs with clear ownership semantics
- Prefer composition over inheritance
- Use type system to prevent invalid states
- Keep modules focused on single responsibilities
- Design for extensibility with traits and generics
- Use builder pattern for complex configurations
- Implement proper error boundaries between layers

### Troubleshooting and Common Issues

**Core Principles:**
- Understand common Rust error patterns
- Use compiler error messages as learning tools
- Know debugging strategies for ownership and lifetime issues
- Recognize performance bottlenecks and their solutions
- Use appropriate tools for diagnosis

**Common Compilation Errors and Solutions:**

**1. Borrow Checker Issues:**

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

**2. Lifetime Parameter Issues:**

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

**3. Move and Ownership Issues:**

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

**4. Trait Bound and Generic Issues:**

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

**5. Pattern Matching Issues:**

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

**Performance Debugging Strategies:**

**1. Identifying Allocation Hotspots:**

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

**2. Debugging Slow Compilation:**

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

**3. Memory Usage Analysis:**

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

**Runtime Debugging Techniques:**

**1. Debug Printing and Logging:**

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

**2. Using Debug Traits Effectively:**

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

**Error Message Interpretation Guide:**

**1. Common Error Patterns:**

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

**Performance Profiling:**

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

**Tools and Commands for Debugging:**

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

**Common Gotchas and Solutions:**

1. **Integer Overflow in Debug Mode**: Use `wrapping_*` operations or handle overflow explicitly
2. **Slow Debug Builds**: Use `opt-level = 1` in debug profile for better performance
3. **Large Binary Size**: Use `strip = true` and `lto = true` in release profile
4. **Slow Incremental Compilation**: Clean build directory occasionally with `cargo clean`
5. **Dependency Conflicts**: Use `cargo tree --duplicates` to find conflicting versions

**Best Practices for Debugging:**
- Start with `cargo check` for fast feedback
- Use compiler error messages as learning tools
- Add debug prints strategically, remove them before committing
- Use proper logging levels in production code
- Profile before optimizing
- Keep debug builds fast with appropriate optimization levels
- Use version control to bisect when issues are introduced
