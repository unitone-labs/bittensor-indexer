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

pub use crate::builder::IndexerBuilder;
pub use crate::decoder::BittensorDecoder;
pub use crate::config::IndexerConfig;
pub use crate::error::IndexerError;
pub use crate::handler::{Context, EventFilter, Handler};
pub use crate::handler_group::HandlerGroup;
pub use crate::indexer::Indexer;
pub use crate::storage::CheckpointStore;
pub use crate::types::decoded::{DecodedBlock, DecodedEvent, DecodedExtrinsic};
pub use crate::types::{BlockNumber, ChainEvent};
pub use crate::validated_types::{PostgresUrl, SqliteUrl, WebSocketUrl};

#[cfg(feature = "postgres-state")]
pub use crate::storage::pg_state::PostgresStateHandler;
#[cfg(feature = "clickhouse")]
pub use crate::storage::clickhouse::ClickHouseHandler;
#[cfg(feature = "redis-cache")]
pub use crate::storage::redis_cache::RedisCacheHandler;

pub use crate::types::entity::{Account, Neuron, StakePosition, Subnet};
pub use crate::types::storage_config::{ClickHouseConfig, PgStateConfig, RedisCacheConfig};

pub use async_trait::async_trait;
pub use parity_scale_codec::Decode;
pub use scale_decode::DecodeAsType;
pub use subxt::{config::substrate::SubstrateConfig, events::StaticEvent, utils::AccountId32};

pub use parity_scale_codec;
pub use scale_decode;
