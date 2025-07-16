# Changelog

## [0.1.0] - 2025-07-10
### Added
- Initial release of Flamewire Bittensor Indexer
- High-performance async blockchain indexing
- Multiple storage backends (JSON, SQLite, PostgreSQL)  
- Advanced handler system with pipelines and parallel execution
- Circuit breaker pattern and retry mechanisms
- Comprehensive event filtering and processing
- Type-safe configuration with builder pattern

## [0.1.1] - 2025-07-11
### Changed

* **Context**: Now generic (`Context<C: Config>`) and includes a new public field `block_hash` representing the hash of the processed block. This field is available in all handlers and enables unique identification of the current block within the pipeline.
* **ChainEvent**: Introduces a new public field `index`, which indicates the event's position (`event_index`) within the block’s event list. This allows for unique identification of every event in a block.
* **Propagation**: All references to `Context` and `ChainEvent` in examples, pipelines, and tests were updated to use the new structures and constructors.

## [0.1.2] - 2025-07-16
### Added

* `IndexerBuilder::max_blocks_per_minute(u32)` method to control block processing rate in minutes.
* New throttling system in `Indexer::run()` that enforces a minimum delay between blocks using `tokio::time::sleep()`.
* `tracing::debug!` logs for throttling activity, including per-block delay timing.
* Optional `handle_event` method in the `Handler` trait with default `Ok(())` implementation, allowing handlers to define only `handle_block` if desired.
* Updated examples to use `tracing::info!` instead of `println!` for structured and async-safe logging.

## [0.1.3] - 2025-07-16
### Changed

* **Handler Trait**: `handle_block` now receives a `&[ChainEvent<C>]` instead of `&Events<C>`, improving decoupling from Subxt internals and enabling clearer handler logic with access to fully decoded events.
* **HandlerGroup**: Updated to pass `&[ChainEvent<C>]` to each internal handler in both sequential and parallel modes. Maintains `strict` and `parallel` logic as before.
* **ConditionalHandler**: Also refactored to support the new `&[ChainEvent<C>]` block interface.
* **Tests**: All unit and integration tests updated to reflect the new `handle_block` interface. Test events are now constructed via `ChainEvent::new()` and passed as slices.

### Added

* **process_block method**: Introduced `Indexer::process_block()` to eliminate code duplication in `Indexer::run()`. This method encapsulates block metadata updating, event fetching, event processing, checkpointing, and throttling logic.

### Removed

* **Code duplication**: Removed repetitive code for event decoding, checkpoint storing, and throttling from both sync and live block loops inside `Indexer::run()` by extracting it into `process_block()`.