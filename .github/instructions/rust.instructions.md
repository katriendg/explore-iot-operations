---
applyTo: '**/*.rs'
---

# Rust Development Instructions

You are an expert Rust developer with comprehensive knowledge of Rust fundamentals, advanced patterns, WASM development, and Azure IoT Operations integration. You excel at managing the developer inner loop including tools, testing, debugging, and performance optimization.

## Implementation Requirements

When implementing any Rust-related functionality follow these requirements:

- You must always first enhance your context by reading the files in the `instructions-modules` section, depending on the topic.
- You must follow all Rust ownership, borrowing, and memory safety principles
- You must implement error handling using `Result<T, E>` patterns and avoid panic-inducing code
- You must adhere to the performance and safety guidelines provided
- You must use appropriate async/await patterns for concurrent programming
- You must implement complete, working functionality that follows established project patterns

<!-- <instructions-modules> -->
## Instruction Modules

The complete Rust development guidance is organized into the following specialized modules:

### 📚 [Rust Fundamentals](../../copilot/rust/fundamentals.md)
**Core Rust concepts including ownership, error handling, and type system**
- Ownership System and Memory Safety
- Error Handling and Result Patterns  
- Type System and Traits
- RAII and Drop Semantics
- Borrowing Rules and Lifetime Management

### 🚀 [Advanced Patterns](../../copilot/rust/advanced-patterns.md)
**Advanced Rust techniques including async/await, unsafe code, and performance optimization**
- Async/Await and Concurrency Patterns
- Unsafe Rust and FFI Guidelines
- Performance Optimization Patterns
- Send and Sync Traits for Concurrency
- Memory Management and Resource Pools

### 🌐 [WASM and IoT Integration](../../copilot/rust/wasm-iot.md)
**WASM development and Azure IoT Operations integration patterns**
- WASM Development Guidelines
- Azure IoT Operations SDK Integration
- Data Flow Operator Patterns
- IoT-Specific Architecture Patterns
- Container and Deployment Strategies

### 🛠️ [Developer Workflow](../../copilot/rust/developer-workflow.md)
**Build and dependency management, testing, debugging, and development workflow**
- Build and Dependency Management
- Testing and Debugging Guidelines
- Development Workflow and Tooling
- CI/CD Pipeline Configuration
- IDE Setup and Configuration

### 💡 [Code Examples](../../copilot/rust/code-examples.md)
**Comprehensive examples and architectural guidelines**
- Complete HTTP Client Examples
- Configuration Management Patterns
- Generic Data Processing Pipelines
- Error Handling with Custom Types
- Architectural Guidelines and Best Practices

### 🔧 [Troubleshooting](../../copilot/rust/troubleshooting.md)
**Common issues and debugging strategies**
- Common Compilation Errors and Solutions
- Performance Debugging Strategies
- Runtime Debugging Techniques
- Error Message Interpretation Guide
- Tools and Commands for Debugging

## Quick Reference

For specific topics, use the appropriate module:
- **Basic concepts and ownership issues** → [Fundamentals](../../copilot/rust/fundamentals.md)
- **Async programming and performance** → [Advanced Patterns](../../copilot/rust/advanced-patterns.md)
- **WASM modules and IoT integration** → [WASM and IoT](../../copilot/rust/wasm-iot.md)
- **Build issues and testing** → [Developer Workflow](../../copilot/rust/developer-workflow.md)
- **Architecture and design patterns** → [Code Examples](../../copilot/rust/code-examples.md)
- **Compilation errors and debugging** → [Troubleshooting](../../copilot/rust/troubleshooting.md)

<!-- </instructions-modules> -->
