/*
 * Copyright 2025 Flamewire
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

pub mod builder;
pub mod config;
pub mod error;
pub mod handler;
pub mod handler_group;
pub mod indexer;
pub mod prelude;
pub mod retry;
pub mod storage;
pub mod types;
pub mod validated_types;

pub use crate::builder::IndexerBuilder;
pub use crate::config::IndexerConfig;
pub use crate::error::IndexerError;
pub use crate::handler::{Context, EventFilter, Handler};
pub use crate::handler_group::HandlerGroup;
pub use crate::indexer::Indexer;
pub use crate::retry::{retry_with_backoff, CircuitBreaker, RetryConfig};
pub use crate::storage::CheckpointStore;
pub use crate::types::{BlockNumber, ChainEvent};
pub use crate::validated_types::{PostgresUrl, SqliteUrl, WebSocketUrl};
