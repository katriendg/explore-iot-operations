# WASM and IoT-Specific Patterns

## WASM Development Guidelines

**Core Principles:**
- Target `wasm32-unknown-unknown` for web deployment
- Use `wasm-pack` for building and packaging WASM modules
- Minimize binary size with careful dependency selection
- Handle JavaScript interop safely and efficiently
- Design for memory-constrained environments

**WASM Project Setup:**

```toml
# Cargo.toml for WASM projects
[package]
name = "my-wasm-module"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.4"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "Document",
  "Element",
  "HtmlElement",
  "Window",
]

[profile.release]
# Optimize for size instead of speed
opt-level = "s"
lto = true
```

**Basic WASM Module Structure:**

```rust
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Import the `console.log` function from the `console` module
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Export a function to JavaScript
#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}!", name);
}

// Complex data structure that can be passed to/from JavaScript
#[derive(Serialize, Deserialize)]
pub struct TemperatureReading {
    pub value: f64,
    pub unit: String,
    pub timestamp: u64,
}

#[wasm_bindgen]
pub fn process_temperature_data(js_value: JsValue) -> Result<JsValue, JsValue> {
    let reading: TemperatureReading = serde_wasm_bindgen::from_value(js_value)?;
    
    // Convert Fahrenheit to Celsius
    let celsius_value = if reading.unit == "F" {
        (reading.value - 32.0) * 5.0 / 9.0
    } else {
        reading.value
    };
    
    let processed = TemperatureReading {
        value: celsius_value,
        unit: "C".to_string(),
        timestamp: reading.timestamp,
    };
    
    Ok(serde_wasm_bindgen::to_value(&processed)?)
}
```

**Memory Management in WASM:**

```rust
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

// Use thread-local storage for state in WASM
thread_local! {
    static SENSOR_DATA: std::cell::RefCell<HashMap<String, Vec<f64>>> = 
        std::cell::RefCell::new(HashMap::new());
}

#[wasm_bindgen]
pub fn add_sensor_reading(sensor_id: &str, value: f64) {
    SENSOR_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.entry(sensor_id.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    });
}

#[wasm_bindgen]
pub fn get_sensor_average(sensor_id: &str) -> Option<f64> {
    SENSOR_DATA.with(|data| {
        let data = data.borrow();
        data.get(sensor_id).map(|readings| {
            readings.iter().sum::<f64>() / readings.len() as f64
        })
    })
}

// Clear memory when done
#[wasm_bindgen]
pub fn clear_sensor_data() {
    SENSOR_DATA.with(|data| {
        data.borrow_mut().clear();
    });
}
```

**Error Handling in WASM:**

```rust
use wasm_bindgen::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid sensor data: {0}")]
    InvalidData(String),
    #[error("Sensor not found: {0}")]
    SensorNotFound(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

// Convert Rust errors to JavaScript-friendly format
impl From<ProcessingError> for JsValue {
    fn from(error: ProcessingError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

#[wasm_bindgen]
pub fn safe_process_data(input: &str) -> Result<String, ProcessingError> {
    // Validate input
    if input.is_empty() {
        return Err(ProcessingError::InvalidData("Input cannot be empty".to_string()));
    }
    
    // Parse and process
    let value: f64 = input.parse()
        .map_err(|_| ProcessingError::InvalidData("Invalid number format".to_string()))?;
    
    if value < -273.15 {
        return Err(ProcessingError::CalculationError("Temperature below absolute zero".to_string()));
    }
    
    Ok(format!("Processed: {:.2}", value))
}
```

## Azure IoT Operations SDK Integration

**Core Principles:**
- Use the tinykube-wasm-sdk for data flow graph integration
- Implement proper operator patterns (Map, Filter, Branch, etc.)
- Handle IoT message formats correctly
- Ensure efficient resource usage in containerized environments
- Follow Azure IoT Operations naming and tagging conventions

**Data Flow Operator Implementation:**

```rust
use tinykube_wasm_sdk::prelude::*;
use tinykube_wasm_sdk::logger::{self, Level};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IoTMessage {
    pub device_id: String,
    pub timestamp: u64,
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemperatureMeasurement {
    pub value: Option<f64>,
    pub unit: String,
    pub sensor_id: String,
}

// Map operator for temperature conversion
#[map_operator]
fn temperature_converter(input: DataModel) -> DataModel {
    let DataModel::Message(mut message) = input else {
        logger::log(Level::Error, "temperature_converter", "Expected Message input");
        return input;
    };
    
    // Read payload safely
    let payload_bytes = message.payload.read();
    
    // Parse the IoT message
    let iot_message: IoTMessage = match serde_json::from_slice(&payload_bytes) {
        Ok(msg) => msg,
        Err(e) => {
            logger::log(Level::Error, "temperature_converter", 
                &format!("Failed to parse IoT message: {}", e));
            return DataModel::Message(message);
        }
    };
    
    // Extract temperature measurement
    let mut temp_measurement: TemperatureMeasurement = match serde_json::from_value(iot_message.payload.clone()) {
        Ok(temp) => temp,
        Err(e) => {
            logger::log(Level::Warn, "temperature_converter", 
                &format!("Failed to parse temperature data: {}", e));
            return DataModel::Message(message);
        }
    };
    
    // Convert Fahrenheit to Celsius
    if temp_measurement.unit == "F" || temp_measurement.unit == "Fahrenheit" {
        if let Some(value) = temp_measurement.value {
            temp_measurement.value = Some((value - 32.0) * 5.0 / 9.0);
            temp_measurement.unit = "C".to_string();
            
            logger::log(Level::Info, "temperature_converter", 
                &format!("Converted temperature for sensor {}", temp_measurement.sensor_id));
        }
    }
    
    // Create updated IoT message
    let updated_iot_message = IoTMessage {
        device_id: iot_message.device_id,
        timestamp: iot_message.timestamp,
        payload: serde_json::to_value(&temp_measurement).unwrap_or(iot_message.payload),
    };
    
    // Serialize back to bytes
    match serde_json::to_vec(&updated_iot_message) {
        Ok(new_payload) => {
            message.payload = BufferOrBytes::Bytes(new_payload);
            DataModel::Message(message)
        }
        Err(e) => {
            logger::log(Level::Error, "temperature_converter", 
                &format!("Failed to serialize result: {}", e));
            DataModel::Message(message)
        }
    }
}

// Filter operator for temperature validation
#[filter_operator]
fn temperature_validator(input: DataModel) -> bool {
    let DataModel::Message(message) = input else {
        return false;
    };
    
    let payload_bytes = message.payload.read();
    
    // Parse and validate temperature range
    if let Ok(iot_message) = serde_json::from_slice::<IoTMessage>(&payload_bytes) {
        if let Ok(temp_measurement) = serde_json::from_value::<TemperatureMeasurement>(iot_message.payload) {
            if let Some(value) = temp_measurement.value {
                // Valid temperature range: -50°C to 100°C
                let is_valid = value >= -50.0 && value <= 100.0;
                
                if !is_valid {
                    logger::log(Level::Warn, "temperature_validator", 
                        &format!("Temperature {} out of valid range for sensor {}", 
                            value, temp_measurement.sensor_id));
                }
                
                return is_valid;
            }
        }
    }
    
    false
}
```

**Configuration and Environment Integration:**

```rust
use std::collections::HashMap;

// Configuration structure for Azure IoT Operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessorConfig {
    pub conversion_enabled: bool,
    pub target_unit: String,
    pub validation_range: (f64, f64),
    pub device_filters: Vec<String>,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        ProcessorConfig {
            conversion_enabled: true,
            target_unit: "C".to_string(),
            validation_range: (-50.0, 100.0),
            device_filters: vec![],
        }
    }
}

// Load configuration from environment or use defaults
fn load_processor_config() -> ProcessorConfig {
    // In Azure IoT Operations, configuration can come from ConfigMaps
    // For now, use defaults with environment variable overrides
    let mut config = ProcessorConfig::default();
    
    // Check for environment variables (these would be set via Kubernetes)
    if let Ok(enabled) = std::env::var("TEMP_CONVERSION_ENABLED") {
        config.conversion_enabled = enabled.to_lowercase() == "true";
    }
    
    if let Ok(unit) = std::env::var("TARGET_TEMPERATURE_UNIT") {
        config.target_unit = unit;
    }
    
    config
}

// Configurable temperature processor
#[map_operator]
fn configurable_temperature_processor(input: DataModel) -> DataModel {
    static CONFIG: std::sync::OnceLock<ProcessorConfig> = std::sync::OnceLock::new();
    let config = CONFIG.get_or_init(load_processor_config);
    
    let DataModel::Message(mut message) = input else {
        return input;
    };
    
    // Apply device filtering if configured
    if !config.device_filters.is_empty() {
        let payload_bytes = message.payload.read();
        if let Ok(iot_message) = serde_json::from_slice::<IoTMessage>(&payload_bytes) {
            if !config.device_filters.contains(&iot_message.device_id) {
                logger::log(Level::Debug, "temperature_processor", 
                    &format!("Filtering out device: {}", iot_message.device_id));
                return DataModel::Message(message);
            }
        }
    }
    
    // Apply conversion if enabled
    if config.conversion_enabled {
        // Use the temperature converter logic here
        temperature_converter(DataModel::Message(message))
    } else {
        DataModel::Message(message)
    }
}
```

## Data Flow Operator Patterns

**Core Principles:**
- Each operator should have a single, well-defined responsibility
- Use appropriate operator types (Map, Filter, Branch, Accumulate, Delay)
- Handle errors gracefully without breaking the data flow
- Log meaningful information for debugging and monitoring
- Design for composability and reusability

**Map Operator Pattern:**

```rust
// Template for map operators that transform data
#[map_operator]
fn data_transformer(input: DataModel) -> DataModel {
    let DataModel::Message(mut message) = input else {
        logger::log(Level::Error, "data_transformer", "Expected Message input");
        return input;
    };
    
    // Extract payload
    let payload = message.payload.read();
    
    // Parse input data
    let input_data: InputType = match serde_json::from_slice(&payload) {
        Ok(data) => data,
        Err(e) => {
            logger::log(Level::Error, "data_transformer", 
                &format!("Failed to parse input: {}", e));
            return DataModel::Message(message);
        }
    };
    
    // Transform the data
    let output_data = transform_logic(input_data);
    
    // Serialize result
    match serde_json::to_vec(&output_data) {
        Ok(new_payload) => {
            message.payload = BufferOrBytes::Bytes(new_payload);
            DataModel::Message(message)
        }
        Err(e) => {
            logger::log(Level::Error, "data_transformer", 
                &format!("Failed to serialize output: {}", e));
            DataModel::Message(message)
        }
    }
}

fn transform_logic(input: InputType) -> OutputType {
    // Implement your transformation logic here
    // This function should be pure and testable
    todo!()
}
```

**Filter Operator Pattern:**

```rust
// Template for filter operators that allow/reject data
#[filter_operator]
fn data_filter(input: DataModel) -> bool {
    let DataModel::Message(message) = input else {
        logger::log(Level::Warn, "data_filter", "Non-message input rejected");
        return false;
    };
    
    let payload = message.payload.read();
    
    // Parse and validate data
    match serde_json::from_slice::<DataType>(&payload) {
        Ok(data) => {
            let is_valid = validation_logic(&data);
            
            if !is_valid {
                logger::log(Level::Debug, "data_filter", 
                    &format!("Data rejected: {:?}", data));
            }
            
            is_valid
        }
        Err(e) => {
            logger::log(Level::Error, "data_filter", 
                &format!("Failed to parse data for filtering: {}", e));
            false
        }
    }
}

fn validation_logic(data: &DataType) -> bool {
    // Implement your validation logic here
    // Return true to allow data through, false to reject
    todo!()
}
```

**Branch Operator Pattern:**

```rust
// Template for branch operators that route data
#[branch_operator]
fn data_router(input: DataModel) -> Vec<DataModel> {
    let DataModel::Message(message) = input else {
        logger::log(Level::Error, "data_router", "Expected Message input");
        return vec![input];
    };
    
    let payload = message.payload.read();
    
    // Parse routing information
    let routing_data: RoutingType = match serde_json::from_slice(&payload) {
        Ok(data) => data,
        Err(e) => {
            logger::log(Level::Error, "data_router", 
                &format!("Failed to parse routing data: {}", e));
            return vec![DataModel::Message(message)];
        }
    };
    
    // Determine routing destinations
    let destinations = routing_logic(&routing_data);
    
    // Create copies for each destination
    destinations.into_iter().map(|dest| {
        let mut routed_message = message.clone();
        
        // Add routing metadata
        let mut routing_info = HashMap::new();
        routing_info.insert("destination".to_string(), serde_json::Value::String(dest));
        
        // Could add routing info to message metadata here
        DataModel::Message(routed_message)
    }).collect()
}

fn routing_logic(data: &RoutingType) -> Vec<String> {
    // Implement your routing logic here
    // Return list of destination identifiers
    todo!()
}
```

**Accumulate Operator Pattern:**

```rust
use std::collections::HashMap;
use std::sync::Mutex;

// Thread-safe accumulator state
static ACCUMULATOR_STATE: Mutex<HashMap<String, AccumulatorData>> = Mutex::new(HashMap::new());

#[derive(Debug, Clone)]
struct AccumulatorData {
    count: u64,
    sum: f64,
    last_update: u64,
}

#[accumulate_operator]
fn data_accumulator(input: DataModel) -> Vec<DataModel> {
    let DataModel::Message(message) = input else {
        return vec![input];
    };
    
    let payload = message.payload.read();
    
    // Parse accumulation data
    let data: AccumulationInput = match serde_json::from_slice(&payload) {
        Ok(data) => data,
        Err(e) => {
            logger::log(Level::Error, "data_accumulator", 
                &format!("Failed to parse accumulation data: {}", e));
            return vec![DataModel::Message(message)];
        }
    };
    
    // Update accumulator state
    let mut state = ACCUMULATOR_STATE.lock().unwrap();
    let accumulator = state.entry(data.key.clone()).or_insert(AccumulatorData {
        count: 0,
        sum: 0.0,
        last_update: 0,
    });
    
    accumulator.count += 1;
    accumulator.sum += data.value;
    accumulator.last_update = data.timestamp;
    
    // Check if we should emit accumulated result
    if should_emit_result(accumulator, &data) {
        let result = AccumulationResult {
            key: data.key.clone(),
            count: accumulator.count,
            average: accumulator.sum / accumulator.count as f64,
            timestamp: accumulator.last_update,
        };
        
        // Reset accumulator
        *accumulator = AccumulatorData {
            count: 0,
            sum: 0.0,
            last_update: data.timestamp,
        };
        
        // Return accumulated result
        match serde_json::to_vec(&result) {
            Ok(result_payload) => {
                let mut result_message = message.clone();
                result_message.payload = BufferOrBytes::Bytes(result_payload);
                vec![DataModel::Message(result_message)]
            }
            Err(e) => {
                logger::log(Level::Error, "data_accumulator", 
                    &format!("Failed to serialize result: {}", e));
                vec![]
            }
        }
    } else {
        // Don't emit anything yet
        vec![]
    }
}

fn should_emit_result(accumulator: &AccumulatorData, current_data: &AccumulationInput) -> bool {
    // Emit after 10 data points or 60 seconds
    accumulator.count >= 10 || 
    (current_data.timestamp - accumulator.last_update) >= 60_000
}
```

**Best Practices for Data Flow Operators:**
- Keep operators stateless when possible; use thread-safe state management when needed
- Always handle parsing errors gracefully
- Use structured logging with appropriate log levels
- Design for testability by separating business logic from WASM bindings
- Document operator behavior and expected input/output formats
- Consider performance implications of data copying and serialization
- Use appropriate data structures for your use case (HashMap for lookups, Vec for sequences, etc.)
