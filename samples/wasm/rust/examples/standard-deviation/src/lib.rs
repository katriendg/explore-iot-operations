// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Standard deviation calculation module for temperature and pressure data
// Uses accumulate operator to collect data over time windows and calculate statistics
#![allow(clippy::missing_safety_doc)]

use tinykube_wasm_sdk::logger::{self, Level};
use tinykube_wasm_sdk::macros::accumulate_operator;
use tinykube_wasm_sdk::metrics::{self, CounterValue, Label};
use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};

// Data structures for sensor measurements
#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct SensorReading {
    pub temperature: Option<f64>,
    pub pressure: Option<f64>,
    pub device_id: String,
    pub timestamp: u64,
    pub unit: SensorUnit,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct SensorUnit {
    pub temperature: String,  // "C" or "F"
    pub pressure: String,     // "Pa", "kPa", "bar", "psi"
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StandardDeviationResult {
    pub device_id: String,
    pub timestamp: u64,
    pub window_start: u64,
    pub window_end: u64,
    pub temperature_stats: Option<StatisticsData>,
    pub pressure_stats: Option<StatisticsData>,
    pub sample_count: u64,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StatisticsData {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: u64,
    pub unit: String,
}

// Thread-safe storage for accumulating data across the time window
static ACCUMULATOR_STATE: LazyLock<Mutex<HashMap<String, WindowData>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
struct WindowData {
    temperature_values: Vec<f64>,
    pressure_values: Vec<f64>,
    window_start: u64,
    window_end: u64,
    device_id: String,
    temperature_unit: String,
    pressure_unit: String,
}

impl WindowData {
    fn new(device_id: String) -> Self {
        WindowData {
            temperature_values: Vec::new(),
            pressure_values: Vec::new(),
            window_start: 0,
            window_end: 0,
            device_id,
            temperature_unit: "C".to_string(),
            pressure_unit: "Pa".to_string(),
        }
    }

    fn add_reading(&mut self, reading: &SensorReading) {
        if let Some(temp) = reading.temperature {
            self.temperature_values.push(temp);
            self.temperature_unit = reading.unit.temperature.clone();
        }
        
        if let Some(pressure) = reading.pressure {
            self.pressure_values.push(pressure);
            self.pressure_unit = reading.unit.pressure.clone();
        }

        // Update window bounds
        if self.window_start == 0 {
            self.window_start = reading.timestamp;
        }
        self.window_end = reading.timestamp;
    }

    fn calculate_statistics(&self, values: &[f64]) -> Option<StatisticsData> {
        if values.is_empty() {
            return None;
        }

        let count = values.len() as u64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        // Calculate variance and standard deviation
        let variance: f64 = values.iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(StatisticsData {
            mean,
            std_dev,
            min,
            max,
            count,
            unit: if std::ptr::eq(values, &self.temperature_values[..]) {
                self.temperature_unit.clone()
            } else {
                self.pressure_unit.clone()
            },
        })
    }

    fn get_temperature_stats(&self) -> Option<StatisticsData> {
        let mut stats = self.calculate_statistics(&self.temperature_values)?;
        stats.unit = self.temperature_unit.clone();
        Some(stats)
    }

    fn get_pressure_stats(&self) -> Option<StatisticsData> {
        let mut stats = self.calculate_statistics(&self.pressure_values)?;
        stats.unit = self.pressure_unit.clone();
        Some(stats)
    }

    fn should_emit(&self, current_timestamp: u64, window_duration_ms: u64) -> bool {
        // Emit when window duration is reached
        (current_timestamp - self.window_start) >= window_duration_ms
    }
}

// Configuration for the standard deviation calculator
static WINDOW_DURATION_MS: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

fn std_dev_accumulate_init(configuration: ModuleConfiguration) -> bool {
    logger::log(
        Level::Info,
        "module-std-dev/accumulate",
        "Standard deviation accumulator initialization started",
    );

    // Default window duration: 5 minutes (300,000 ms)
    let mut window_duration = 300_000u64;

    // Parse configuration parameters
    for (key, value) in configuration.properties {
        logger::log(
            Level::Info,
            "module-std-dev/accumulate",
            &format!("Configuration parameter: {}={}", key, value),
        );

        match key.as_str() {
            "window_duration_ms" => {
                if let Ok(duration) = value.parse::<u64>() {
                    window_duration = duration;
                    logger::log(
                        Level::Info,
                        "module-std-dev/accumulate",
                        &format!("Window duration set to {} ms", duration),
                    );
                } else {
                    logger::log(
                        Level::Warn,
                        "module-std-dev/accumulate",
                        &format!("Invalid window_duration_ms value: {}", value),
                    );
                }
            }
            _ => {
                logger::log(
                    Level::Debug,
                    "module-std-dev/accumulate",
                    &format!("Unknown configuration parameter: {}", key),
                );
            }
        }
    }

    // Store the window duration
    WINDOW_DURATION_MS.set(window_duration).unwrap();

    logger::log(
        Level::Info,
        "module-std-dev/accumulate",
        &format!("Initialization complete. Window duration: {} ms", window_duration),
    );

    true
}

#[accumulate_operator(init = "std_dev_accumulate_init")]
fn std_dev_accumulate(staged: DataModel, inputs: Vec<DataModel>) -> DataModel {
    let labels = vec![Label {
        key: "module".to_owned(),
        value: "module-std-dev/accumulate".to_owned(),
    }];
    let _ = metrics::add_to_counter("requests", CounterValue::U64(1), Some(&labels));

    let DataModel::Message(mut result) = staged else {
        logger::log(Level::Error, "module-std-dev/accumulate", "Expected Message input for staged data");
        return staged;
    };

    let window_duration = *WINDOW_DURATION_MS.get().unwrap_or(&300_000);

    // Process all inputs in the accumulation window
    for input in inputs {
        let DataModel::Message(message) = input else {
            logger::log(Level::Warn, "module-std-dev/accumulate", "Skipping non-message input");
            continue;
        };

        let payload = message.payload.read();
        
        // Parse sensor reading
        let reading: SensorReading = match serde_json::from_slice(&payload) {
            Ok(reading) => reading,
            Err(e) => {
                logger::log(
                    Level::Error,
                    "module-std-dev/accumulate",
                    &format!("Failed to parse sensor reading: {}", e),
                );
                continue;
            }
        };

        logger::log(
            Level::Debug,
            "module-std-dev/accumulate",
            &format!("Processing reading from device: {}", reading.device_id),
        );

        // Update accumulator state
        let mut state = ACCUMULATOR_STATE.lock().unwrap();
        let window_data = state.entry(reading.device_id.clone())
            .or_insert_with(|| WindowData::new(reading.device_id.clone()));
        
        window_data.add_reading(&reading);

        // Check if we should emit results for this device
        if window_data.should_emit(reading.timestamp, window_duration) {
            let sample_count = window_data.temperature_values.len() + window_data.pressure_values.len();
            
            let result_data = StandardDeviationResult {
                device_id: reading.device_id.clone(),
                timestamp: reading.timestamp,
                window_start: window_data.window_start,
                window_end: window_data.window_end,
                temperature_stats: window_data.get_temperature_stats(),
                pressure_stats: window_data.get_pressure_stats(),
                sample_count: sample_count as u64,
            };

            logger::log(
                Level::Info,
                "module-std-dev/accumulate",
                &format!("Emitting statistics for device {}: {} samples over {}ms window", 
                    reading.device_id, sample_count, window_duration),
            );

            // Log the calculated statistics
            if let Some(ref temp_stats) = result_data.temperature_stats {
                logger::log(
                    Level::Info,
                    "module-std-dev/accumulate",
                    &format!("Temperature stats - Mean: {:.2}°{}, StdDev: {:.2}, Min: {:.2}, Max: {:.2}", 
                        temp_stats.mean, temp_stats.unit, temp_stats.std_dev, temp_stats.min, temp_stats.max),
                );
            }

            if let Some(ref pressure_stats) = result_data.pressure_stats {
                logger::log(
                    Level::Info,
                    "module-std-dev/accumulate",
                    &format!("Pressure stats - Mean: {:.2} {}, StdDev: {:.2}, Min: {:.2}, Max: {:.2}", 
                        pressure_stats.mean, pressure_stats.unit, pressure_stats.std_dev, pressure_stats.min, pressure_stats.max),
                );
            }

            // Serialize the result
            match serde_json::to_vec(&result_data) {
                Ok(result_payload) => {
                    result.payload = BufferOrBytes::Bytes(result_payload);
                    result.timestamp = message.timestamp;
                    result.topic = message.topic;
                    
                    // Reset the window data for this device
                    *window_data = WindowData::new(reading.device_id.clone());
                    
                    let _ = metrics::add_to_counter("results_emitted", CounterValue::U64(1), Some(&labels));
                    return DataModel::Message(result);
                }
                Err(e) => {
                    logger::log(
                        Level::Error,
                        "module-std-dev/accumulate",
                        &format!("Failed to serialize result: {}", e),
                    );
                }
            }
        }
    }

    // If no results were emitted, return empty message
    result.payload = BufferOrBytes::Bytes(Vec::new());
    DataModel::Message(result)
}
