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