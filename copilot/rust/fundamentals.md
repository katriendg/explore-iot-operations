# Rust Fundamentals

## Ownership System and Memory Safety

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

## Error Handling and Result Patterns  

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

## Type System and Traits

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
