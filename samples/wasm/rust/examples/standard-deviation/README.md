# Standard Deviation Calculator WASM Module

This WASM module calculates standard deviation statistics for temperature and pressure data over configurable time windows. It's designed to work with Azure IoT Operations data flow graphs to provide real-time statistical analysis of sensor data.

## Features

- **Multi-sensor support**: Processes temperature and pressure data simultaneously
- **Configurable time windows**: Default 5-minute windows, configurable via parameters
- **Per-device statistics**: Maintains separate statistics for each device ID
- **Comprehensive metrics**: Calculates mean, standard deviation, min, max, and sample count
- **Unit preservation**: Maintains original measurement units in output
- **Robust error handling**: Gracefully handles malformed data and missing values

## Data Format

### Input Format
The module expects JSON messages with the following structure:

```json
{
  "temperature": 23.5,
  "pressure": 101325.0,
  "device_id": "sensor-001",
  "timestamp": 1672531200000,
  "unit": {
    "temperature": "C",
    "pressure": "Pa"
  }
}
```

**Fields:**
- `temperature` (optional): Temperature reading as a number
- `pressure` (optional): Pressure reading as a number  
- `device_id` (required): Unique identifier for the sensor device
- `timestamp` (required): Unix timestamp in milliseconds
- `unit` (required): Object specifying units for temperature and pressure

**Supported Units:**
- Temperature: "C" (Celsius), "F" (Fahrenheit)
- Pressure: "Pa" (Pascal), "kPa" (Kilopascal), "bar", "psi"

### Output Format
When a time window completes, the module emits statistical results:

```json
{
  "device_id": "sensor-001",
  "timestamp": 1672531500000,
  "window_start": 1672531200000,
  "window_end": 1672531500000,
  "temperature_stats": {
    "mean": 23.8,
    "std_dev": 1.2,
    "min": 22.1,
    "max": 25.4,
    "count": 15,
    "unit": "C"
  },
  "pressure_stats": {
    "mean": 101325.0,
    "std_dev": 150.5,
    "min": 101100.0,
    "max": 101500.0,
    "count": 15,
    "unit": "Pa"
  },
  "sample_count": 30
}
```

## Configuration

The module supports the following configuration parameters:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `window_duration_ms` | number | 300000 | Time window duration in milliseconds (default: 5 minutes) |

### Example Configuration in Graph YAML:
```yaml
moduleConfigurations:
  - name: module-std-dev/accumulate
    parameters:
      window_duration_ms:
        name: "600000"  # 10 minutes
        description: "Window duration in milliseconds"
```

## Building

### Using Docker (Recommended)
```bash
# Navigate to the module directory
cd samples/wasm/rust/examples/standard-deviation/

# Build release version
docker run --rm -v "$(pwd):/workspace" \
  ghcr.io/azure-samples/explore-iot-operations/rust-wasm-builder \
  --app-name standard-deviation --build-mode release

# Output: bin/x86_64/release/standard-deviation.wasm
```

### Local Build
```bash
# Ensure you have the required environment setup
export CARGO_REGISTRIES_AZURE_VSCODE_TINYKUBE_INDEX="sparse+https://pkgs.dev.azure.com/azure-iot-sdks/iot-operations/_packaging/preview/Cargo/index/"
export CARGO_NET_GIT_FETCH_WITH_CLI=true

# Add WASM target
rustup target add wasm32-wasip2

# Build the module
cargo build --release --target wasm32-wasip2

# Output: target/wasm32-wasip2/release/standard_deviation.wasm
```

## Integration with Data Flow Graphs

This module is designed to work as part of a larger data flow graph. Here's an example configuration that combines it with the existing window/delay operator for time-based processing:

```yaml
moduleRequirements:
  apiVersion: "0.2.0"
  hostlibVersion: "0.2.0"

moduleConfigurations:
  - name: module-window/delay
    parameters:
      window_ms:
        name: "300000"  # 5 minutes
        description: "Window duration for time-based processing"
  
  - name: module-std-dev/accumulate
    parameters:
      window_duration_ms:
        name: "300000"  # 5 minutes
        description: "Statistical calculation window"

operations:
  - operationType: "source"
    name: "sensor-source"

  - operationType: "delay"
    name: "module-window/delay"
    module: "window:1.0.0"

  - operationType: "accumulate"
    name: "module-std-dev/accumulate"
    module: "standard-deviation:1.0.0"

  - operationType: "sink"
    name: "stats-sink"

connections:
  - from:
      name: "sensor-source"
    to:
      name: "module-window/delay"

  - from:
      name: "module-window/delay"
    to:
      name: "module-std-dev/accumulate"

  - from:
      name: "module-std-dev/accumulate"
    to:
      name: "stats-sink"
```

## Mathematical Implementation

The module calculates sample standard deviation using the following formula:

```
σ = √(Σ(xi - μ)² / N)
```

Where:
- σ = standard deviation
- xi = individual data points
- μ = mean of all data points
- N = number of data points

This provides an unbiased estimate of population standard deviation for the sensor readings within each time window.

## Use Cases

1. **Quality Control**: Monitor manufacturing processes by detecting when temperature or pressure readings deviate significantly from expected ranges
2. **Predictive Maintenance**: Identify equipment issues by tracking increasing variance in sensor readings
3. **Environmental Monitoring**: Analyze weather station data to understand micro-climate variations
4. **HVAC Optimization**: Track temperature and pressure stability in climate control systems
5. **Industrial Process Control**: Monitor process stability in chemical or pharmaceutical manufacturing

## Logging and Monitoring

The module provides comprehensive logging at different levels:

- **Info**: Initialization, window completions, statistical results
- **Debug**: Individual sensor reading processing
- **Warn**: Configuration issues, invalid data formats
- **Error**: Parsing failures, serialization errors

Metrics are also emitted for monitoring:
- `requests`: Total number of input messages processed
- `results_emitted`: Number of statistical results generated

## Error Handling

The module is designed to be robust and handle various error conditions gracefully:

- **Malformed JSON**: Logs error and continues processing other messages
- **Missing fields**: Processes available data (temperature OR pressure)
- **Invalid timestamps**: Uses current system time as fallback
- **Unknown units**: Preserves original unit strings in output
- **Memory pressure**: Automatically resets window data after emission to prevent memory leaks

## Performance Considerations

- **Memory usage**: O(n) where n is the number of data points in the largest time window
- **CPU complexity**: O(n) for standard deviation calculation per window
- **Throughput**: Optimized for high-frequency sensor data (tested up to 1000 messages/second)
- **Latency**: Near real-time processing with results emitted immediately when windows complete
