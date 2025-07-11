# Flamewire Bittensor Indexer

A high-performance, production-ready Rust indexer for the Bittensor blockchain with advanced event processing, resilient error handling, and flexible storage options.

[![Crates.io](https://img.shields.io/crates/v/flamewire-bittensor-indexer.svg)](https://crates.io/crates/flamewire-bittensor-indexer)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.88.0%2B-blue.svg)](https://www.rust-lang.org)

## Overview

Flamewire Bittensor Indexer is an enterprise-grade solution for indexing and processing Bittensor blockchain data. Built with Rust for maximum performance and safety, it provides a comprehensive framework for real-time blockchain data processing with advanced features like circuit breakers, automatic retries, parallel processing, and multiple storage backends.

## 🚀 Key Features

### Performance & Reliability
- **High-Performance Architecture**: Built with async Rust for maximum throughput
- **Circuit Breaker Pattern**: Automatic failure detection and recovery
- **Exponential Backoff**: Smart retry mechanisms for transient failures  
- **Connection Pooling**: Efficient database connection management
- **Memory Efficient**: Streaming event processing with minimal memory footprint

### Flexible Event Processing
- **Event Filtering**: Process specific pallets, events, or all blockchain events
- **Handler Groups**: Sequential or parallel execution modes
- **Conditional Processing**: Execute handlers based on custom conditions
- **Pipeline Data Sharing**: Pass data between handlers in processing pipelines
- **Strict Mode**: Stop processing on first error for critical operations

### Storage & Persistence
- **Multiple Backends**: JSON, SQLite, and PostgreSQL support
- **Checkpoint System**: Automatic resume from last processed block
- **Transaction Safety**: ACID compliance for database operations
- **Schema Migration**: Automatic database schema setup

### Developer Experience
- **Builder Pattern**: Intuitive API for configuration
- **Type Safety**: Compile-time validation of configurations
- **Comprehensive Error Handling**: Detailed error reporting and recovery
- **Rich Examples**: Complete examples for common use cases

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
flamewire-bittensor-indexer = "0.1.0"

# With optional features
flamewire-bittensor-indexer = { version = "0.1.0", features = ["postgres", "sqlite"] }
```

### Available Features

- `json-storage` (default): JSON file-based checkpoint storage
- `postgres`: PostgreSQL database backend
- `sqlite`: SQLite database backend  
- `testing`: Additional testing utilities

## 🎯 Quick Start

### Basic Event Processing

```rust
use flamewire_bittensor_indexer::prelude::*;

struct EventLogger;

#[async_trait]
impl Handler<SubstrateConfig> for EventLogger {
    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        println!(
            "Block {}: {}.{}", 
            ctx.block_number,
            event.pallet_name(), 
            event.variant_name()
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(WebSocketUrl::parse("wss://archive.chain.opentensor.ai:443")?)
        .start_from_block(1000)
        .add_handler(EventLogger)
        .build()
        .await?;
    
    indexer.run().await?;
    Ok(())
}
```

### Advanced: Processing Transfer Events

```rust
use flamewire_bittensor_indexer::prelude::*;

#[derive(Debug, Decode, DecodeAsType)]
struct TransferEvent {
    from: AccountId32,
    to: AccountId32,
    amount: u128,
}

impl StaticEvent for TransferEvent {
    const PALLET: &'static str = "Balances";
    const EVENT: &'static str = "Transfer";
}

struct TransferProcessor;

#[async_trait]
impl Handler<SubstrateConfig> for TransferProcessor {
    fn event_filter(&self) -> EventFilter {
        EventFilter::event("Balances", "Transfer")
    }

    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        if let Some(transfer) = event.as_event::<TransferEvent>()? {
            println!(
                "Transfer: {} -> {} (Amount: {})",
                transfer.from, transfer.to, transfer.amount
            );
        }
        Ok(())
    }

    async fn handle_error(&self, error: &IndexerError, ctx: &Context<SubstrateConfig>) {
        eprintln!("Error processing transfer at block {}: {}", ctx.block_number, error);
    }
}
```

## 🏗️ Handler Groups & Pipelines

### Sequential Processing Pipeline

```rust
let pipeline = HandlerGroup::new()
    .add(DataExtractor)      // Extract data from events
    .pipe_to(DataTransformer) // Transform extracted data
    .pipe_to(DataSaver);     // Save to database

let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
    .connect(WebSocketUrl::parse("wss://node.url")?)
    .add_handler_group(pipeline)
    .build()
    .await?;
```

### Parallel Processing for Performance

```rust
let parallel_handlers = HandlerGroup::parallel()
    .add(DatabaseSaver)    // Save to primary database
    .add(BackupSaver)      // Save to backup storage  
    .add(MetricsCollector) // Update metrics
    .add(CacheUpdater);    // Update cache

let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
    .connect(WebSocketUrl::parse("wss://node.url")?)
    .add_handler_group(parallel_handlers)
    .build()
    .await?;
```

### Strict Mode for Critical Operations

```rust
let critical_pipeline = HandlerGroup::new()
    .strict()  // Stop on first error
    .add(DataValidator)
    .add(CriticalDataSaver);
```

### Conditional Handler Execution

```rust
let conditional_group = HandlerGroup::new()
    .add_conditional(TransferHandler, |event| {
        event.pallet_name() == "Balances" && 
        event.variant_name() == "Transfer"
    })
    .add_conditional(StakingHandler, |event| {
        event.pallet_name() == "Staking"
    });
```

## 💾 Storage Configuration

### JSON Storage (Default)

```rust
let indexer = IndexerBuilder::<SubstrateConfig>::new()
    .connect(WebSocketUrl::parse("wss://node.url")?)
    // JSON storage in ./database/checkpoint.json (default)
    .build()
    .await?;
```

### SQLite Database

```rust
let indexer = IndexerBuilder::<SubstrateConfig>::new()
    .connect(WebSocketUrl::parse("wss://node.url")?)
    .with_sqlite("sqlite://./indexer.db")
    .build()
    .await?;
```

### PostgreSQL Database

```rust
let indexer = IndexerBuilder::<SubstrateConfig>::new()
    .connect(WebSocketUrl::parse("wss://node.url")?)
    .with_postgres("postgres://user:password@localhost:5432/bittensor_data")
    .build()
    .await?;
```

## 🔧 Advanced Configuration

### Block Range Processing

```rust
let indexer = IndexerBuilder::<SubstrateConfig>::new()
    .connect(WebSocketUrl::parse("wss://node.url")?)
    .start_from_block(1_000_000)  // Start from specific block
    .end_at_block(2_000_000)      // Process until this block
    .build()
    .await?;
```

### Custom Retry Configuration

```rust
use flamewire_bittensor_indexer::{RetryConfig, CircuitBreaker};
use std::time::Duration;

let retry_config = RetryConfig {
    max_retries: 5,
    initial_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(30),
    backoff_multiplier: 2.0,
};

let circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(60));
```

## 🛡️ Error Handling & Resilience

### Comprehensive Error Types

```rust
#[async_trait]
impl Handler<SubstrateConfig> for RobustHandler {
    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        // Your processing logic
        Ok(())
    }

    async fn handle_error(&self, error: &IndexerError, ctx: &Context<SubstrateConfig>) {
        match error {
            IndexerError::ConnectionFailed { url, source } => {
                eprintln!("Connection failed to {}: {}", url, source);
            }
            IndexerError::EventDecodingFailed { pallet, event, block, .. } => {
                eprintln!("Failed to decode {}.{} at block {}", pallet, event, block);
            }
            IndexerError::HandlerFailed { handler, block, .. } => {
                eprintln!("Handler {} failed at block {}", handler, block);
            }
            IndexerError::CheckpointError { operation, backend, .. } => {
                eprintln!("Checkpoint {} failed on {}", operation, backend);
            }
            _ => eprintln!("Other error: {}", error),
        }
    }
}
```

### Circuit Breaker for External Services

```rust
use flamewire_bittensor_indexer::{CircuitBreaker, retry_with_backoff, RetryConfig};

struct ExternalServiceHandler {
    circuit_breaker: Arc<CircuitBreaker>,
}

#[async_trait]
impl Handler<SubstrateConfig> for ExternalServiceHandler {
    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        if self.circuit_breaker.is_open() {
            println!("Circuit breaker open - skipping external service call");
            return Ok(());
        }

        // Attempt external service call with retry
        let result = retry_with_backoff(
            || async { /* external service call */ Ok(()) },
            &RetryConfig::default(),
            &self.circuit_breaker,
        ).await;

        match result {
            Ok(_) => self.circuit_breaker.record_success(),
            Err(_) => self.circuit_breaker.record_failure(),
        }

        result
    }
}
```

## 🎯 Event Filtering

### Filter Types

```rust
// Process all events
EventFilter::all()

// Process all events from a specific pallet
EventFilter::pallet("Balances")

// Process specific events only
EventFilter::event("Balances", "Transfer")
```

### Dynamic Filtering in Handlers

```rust
struct DynamicHandler {
    target_pallets: Vec<String>,
}

#[async_trait]
impl Handler<SubstrateConfig> for DynamicHandler {
    fn event_filter(&self) -> EventFilter {
        EventFilter::all() // We'll filter manually
    }

    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        if self.target_pallets.contains(&event.pallet_name().to_string()) {
            // Process this event
            println!("Processing {} event", event.pallet_name());
        }
        Ok(())
    }
}
```

## 🔄 Pipeline Data Sharing

```rust
struct DataExtractor;

#[async_trait]
impl Handler<SubstrateConfig> for DataExtractor {
    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        if let Some(transfer) = event.as_event::<TransferEvent>()? {
            // Store data for next handler in pipeline
            ctx.set_pipeline_data("transfer", transfer);
        }
        Ok(())
    }
}

struct DataProcessor;

#[async_trait]
impl Handler<SubstrateConfig> for DataProcessor {
    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        // Retrieve data from previous handler
        if let Some(transfer) = ctx.get_pipeline_data::<TransferEvent>("transfer") {
            println!("Processing transfer: {:?}", transfer);
        }
        Ok(())
    }
}
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run unit tests only
cargo test --test unit

# Run integration tests
cargo test --test integration

# Run with logging
RUST_LOG=debug cargo test --all-features
```

### Property-Based Testing

The indexer includes comprehensive property-based tests using `proptest`:

```bash
# Run property-based tests
cargo test prop_ --all-features
```

## 🏎️ Performance Optimization

### Parallel Handler Execution

```rust
// CPU-intensive handlers benefit from parallel execution
let cpu_intensive = HandlerGroup::parallel()
    .add(DataAnalyzer)
    .add(MetricsCalculator)
    .add(ReportGenerator);
```

### Memory Efficiency

- Events are processed in a streaming fashion
- Minimal memory allocation during event processing
- Efficient connection pooling for database operations

### Database Performance

```rust
// For high-throughput scenarios, use PostgreSQL
let indexer = IndexerBuilder::<SubstrateConfig>::new()
    .connect(WebSocketUrl::parse("wss://node.url")?)
    .with_postgres("postgres://user:pass@localhost/db?sslmode=require")
    .build()
    .await?;
```

## 📚 Architecture

### Core Components

- **Indexer**: Main orchestration engine
- **Builder**: Type-safe configuration builder
- **Handlers**: Event processing logic
- **Handler Groups**: Execution orchestration
- **Storage Layer**: Pluggable persistence backends
- **Retry System**: Resilience and fault tolerance
- **Event Types**: Type-safe event definitions

### Design Principles

- **Modularity**: Pluggable components for flexibility
- **Type Safety**: Compile-time validation where possible
- **Resilience**: Built-in error handling and recovery
- **Performance**: Async-first architecture
- **Developer Experience**: Intuitive APIs and clear error messages

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/unitone-labs/bittensor-indexer.git
   cd bittensor-indexer
   ```

2. Install Rust (1.88.0 or later):
   ```bash
   rustup update stable
   ```

3. Run tests:
   ```bash
   cargo test --all-features
   ```

4. Check formatting and linting:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

### Code Style

- Use `cargo fmt` for formatting
- Follow Rust naming conventions
- Add tests for new functionality
- Update documentation for public APIs

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Repository**: [GitHub](https://github.com/unitone-labs/bittensor-indexer)
- **Crates.io**: [flamewire-bittensor-indexer](https://crates.io/crates/flamewire-bittensor-indexer)
- **Documentation**: [docs.rs](https://docs.rs/flamewire-bittensor-indexer)

## 🆘 Support

If you encounter issues or have questions:

1. Check existing [GitHub Issues](https://github.com/unitone-labs/bittensor-indexer/issues)
2. Create a new issue with:
   - Detailed error description
   - Code examples
   - Environment information
   - Steps to reproduce

## 🙏 Acknowledgments

Built with ❤️ for the Bittensor ecosystem by [Flamewire](https://flamewire.io).

Special thanks to the Bittensor community and the Rust ecosystem maintainers.