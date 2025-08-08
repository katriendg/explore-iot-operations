# Standard Deviation Calculator WASM Module

This WASM module calculates standard deviation statistics for temperature and pressure data over configurable time windows. It's designed to work with Azure IoT Operations data flow graphs to provide real-time statistical analysis of sensor data.

## Features

- **Multi-sensor support**: Processes temperature and pressure data simultaneously
- **Configurable time windows**: Default 5-minute windows, configurable via parameters
- **Per-device statistics**: Maintains separate statistics for each device ID
- **Comprehensive metrics**: Calculates mean, standard deviation, min, max, and sample count
- **Unit preservation**: Maintains original measurement units in output
- **Robust error handling**: Gracefully handles malformed data and missing values

## Prerequisites

Before deploying this module, ensure you have:

- Azure IoT Operations instance, version 1.2 preview or later, deployed on an Arc-enabled Kubernetes cluster
- Azure Container Registry (ACR) to store WASM modules and graphs
- ORAS CLI installed for pushing WASM modules to the registry
- Access to kubectl to deploy data flow graphs
- MQTT client pod deployed in your cluster for testing
- ACR pull permissions granted to the IoT Operations managed identity

### Set required environment variables

These variables are used by the commands below. Set them once before you start.

```bash
# Azure subscription (optional but recommended)
export SUBSCRIPTION_ID="<your_subscription_id>"

# Arc-connected Kubernetes cluster details
export RESOURCE_GROUP="<your_resource_group>"
export CLUSTER_NAME="<your_arc_connected_k8s_cluster_name>"
export LOCATION="<azure_region>"

# Container Registry (name only, without domain)
export ACR_NAME="<your_acr_name_without_domain>"

# Namespace used by Azure IoT Operations (default)
export AIO_NAMESPACE="azure-iot-operations"
```

## Quick Start Deployment Guide

Follow these steps to deploy the standard deviation calculator in your Azure IoT Operations cluster, ensuring you have the necessary prerequisites in place.

### Build the WASM Module (if needed)

The pre-built module is available, but if you need to rebuild:

```bash
# Navigate to the module directory
cd samples/wasm/rust/examples/standard-deviation/

# Build using Docker (recommended)
docker run --rm -v "$(pwd):/workspace" \
  ghcr.io/azure-samples/explore-iot-operations/rust-wasm-builder \
  --app-name standard-deviation --build-mode release

# The built module will be at bin/x86_64/release/standard-deviation.wasm
```

### Push Modules to Your Registry

Push the required modules and graph definitions to your ACR:

```bash
# Log in to your ACR
az acr login --name $ACR_NAME

# Pull the required dependencies from the public registry
oras pull ghcr.io/azure-samples/explore-iot-operations/window:1.0.0

# Push the standard deviation module (use the pre-built one or your newly built one)
oras push $ACR_NAME.azurecr.io/standard-deviation:1.0.0 bin/x86_64/release/standard-deviation.wasm

# Push the window dependency
oras push $ACR_NAME.azurecr.io/window:1.0.0 window-1.0.0.wasm

# Push the graph definition
oras push $ACR_NAME.azurecr.io/graph-std-dev:1.0.0 graph-std-dev.yaml
```

### Create Registry Endpoint - skip if you already have it

Create a registry endpoint for your ACR:

```bash
# Get your IoT Operations extension name
export EXTENSION_NAME=$(az k8s-extension list \
  --resource-group $RESOURCE_GROUP \
  --cluster-name $CLUSTER_NAME \
  --cluster-type connectedClusters \
  --query "[?extensionType=='microsoft.iotoperations'].name" \
  --output tsv)

# Create registry endpoint
cat <<EOF | kubectl apply -f -
apiVersion: connectivity.iotoperations.azure.com/v1beta1
kind: RegistryEndpoint
metadata:
  name: regend-acr
  namespace: azure-iot-operations
spec:
  host: $ACR_NAME.azurecr.io
  authentication:
    method: SystemAssignedManagedIdentity
    systemAssignedManagedIdentitySettings:
      audience: https://management.azure.com/
EOF
```

### Deploy the Data Flow Graph

Create and deploy the data flow graph configuration:

```bash
cat <<EOF | kubectl apply -f -
apiVersion: connectivity.iotoperations.azure.com/v1beta1
kind: DataflowGraph
metadata:
  name: std-dev-calculator
  namespace: azure-iot-operations
spec:
  requestDiskPersistence: Enabled
  profileRef: default

  nodes:
    - nodeType: Source
      name: sensor-source
      sourceSettings:
        endpointRef: default
        dataSources:
          - sensor/+/data

    - nodeType: Graph
      name: std-dev-processor
      graphSettings:
        registryEndpointRef: regend-acr
        artifact: graph-std-dev:1.0.0
        configuration:
          - key: window_duration_ms
            value: "300000"  # 5 minutes

    - nodeType: Destination
      name: stats-destination
      destinationSettings:
        endpointRef: default
        dataDestination: analytics/statistics

  nodeConnections:
    - from:
        name: sensor-source
      to:
        name: std-dev-processor

    - from:
        name: std-dev-processor
      to:
        name: stats-destination
EOF
```

### Step 8: Deploy MQTT Client for Testing

If you don't have an MQTT client pod, deploy one:

```bash
# Deploy the MQTT client
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: mqtt-client
  namespace: azure-iot-operations
spec:
  serviceAccountName: default
  containers:
  - image: alpine
    name: mqtt-client
    command: ["sh", "-c"]
    args: ["apk add mosquitto-clients && sleep infinity"]
    volumeMounts:
    - name: broker-sat
      mountPath: /var/run/secrets/tokens
      readOnly: true
    - name: broker-ca
      mountPath: /var/run/certs
      readOnly: true
  volumes:
  - name: broker-sat
    projected:
      sources:
      - serviceAccountToken:
          path: broker-sat
          audience: aio-internal
          expirationSeconds: 86400
  - name: broker-ca
    secret:
      secretName: aio-ca-trust-bundle-test-only
EOF
```

Wait for the pod to be ready:

```bash
kubectl wait --for=condition=Ready pod/mqtt-client -n azure-iot-operations --timeout=60s
```

## Testing the Deployment

### Step 9: Send Test Data

Open two terminal sessions to test the data flow.

**Terminal 1 - Send sensor data:**

```bash
# Connect to the MQTT client pod
kubectl exec -it mqtt-client -n azure-iot-operations -- sh

# Create and run a script to send sensor data
cat > send_sensor_data.sh << 'EOF'
#!/bin/sh

# Array of device IDs for testing
DEVICES="sensor-001 sensor-002 sensor-003"

while true; do
  for device in $DEVICES; do
    # Generate random temperature (18-28°C) and pressure (101000-102000 Pa)
    temp=$(awk "BEGIN {printf \"%.1f\", 18 + rand() * 10}")
    pressure=$(awk "BEGIN {printf \"%.1f\", 101000 + rand() * 1000}")
    timestamp=$(date +%s)000  # Convert to milliseconds
    
    # Create the sensor reading JSON
    payload=$(cat <<JSON
{
  "temperature": $temp,
  "pressure": $pressure,
  "device_id": "$device",
  "timestamp": $timestamp,
  "unit": {
    "temperature": "C",
    "pressure": "Pa"
  }
}
JSON
)
    
    echo "Publishing data for $device: temp=${temp}°C, pressure=${pressure}Pa"
    
    # Publish to the sensor data topic
    mosquitto_pub -h aio-broker -p 18883 \
      -m "$payload" \
      -t "sensor/$device/data" \
      -d \
      --cafile /var/run/certs/ca.crt \
      -D CONNECT authentication-method 'K8S-SAT' \
      -D CONNECT authentication-data $(cat /var/run/secrets/tokens/broker-sat)
    
    sleep 2
  done
  sleep 10
done
EOF

chmod +x send_sensor_data.sh
./send_sensor_data.sh
```

**Terminal 2 - Subscribe to statistics:**

```bash
# Connect to the MQTT client pod in a second terminal
kubectl exec -it mqtt-client -n azure-iot-operations -- sh

# Subscribe to the statistics output
mosquitto_sub -h aio-broker -p 18883 \
  -t "analytics/statistics" \
  -v \
  -d \
  --cafile /var/run/certs/ca.crt \
  -D CONNECT authentication-method 'K8S-SAT' \
  -D CONNECT authentication-data $(cat /var/run/secrets/tokens/broker-sat)
```

### Expected Output

After about 5 minutes (the default window duration), you should see statistical output like:

```json
{
  "device_id": "sensor-001",
  "timestamp": 1672531500000,
  "window_start": 1672531200000,
  "window_end": 1672531500000,
  "temperature_stats": {
    "mean": 23.26,
    "std_dev": 0.64,
    "min": 22.5,
    "max": 24.2,
    "count": 15,
    "unit": "C"
  },
  "pressure_stats": {
    "mean": 101324.32,
    "std_dev": 25.67,
    "min": 101290.8,
    "max": 101355.1,
    "count": 15,
    "unit": "Pa"
  },
  "sample_count": 30
}
```

### Step 10: Monitor and Troubleshoot

Check the data flow graph status:

```bash
# Check if the data flow graph is running
kubectl get dataflowgraph std-dev-calculator -n azure-iot-operations -o yaml

# Check the data flow controller logs
kubectl logs -n azure-iot-operations -l app.kubernetes.io/name=aio-dataflow-controller

# Check for any errors in the tinykube controller
kubectl logs aio-dataflow-tinykube-controller-0 -n azure-iot-operations
```

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
