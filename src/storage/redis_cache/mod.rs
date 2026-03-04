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

pub mod keys;

use crate::error::IndexerError;
use crate::handler::{Context, Handler};
use crate::types::decoded::{DecodedBlock, DecodedEvent};
use crate::types::storage_config::RedisCacheConfig;
use crate::types::ChainEvent;

use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::Client;
use subxt::Config;
use tracing::debug;

/// Caches decoded blocks and events in Redis.
pub struct RedisCacheHandler {
    conn: ConnectionManager,
    _config: RedisCacheConfig,
}

impl RedisCacheHandler {
    /// Creates a new handler from the given config.
    pub async fn new(config: RedisCacheConfig) -> Result<Self, IndexerError> {
        let client =
            Client::open(config.url.as_str()).map_err(|e| IndexerError::RedisError {
                operation: "connect".to_string(),
                source: Box::new(e),
            })?;

        let conn =
            ConnectionManager::new(client)
                .await
                .map_err(|e| IndexerError::RedisError {
                    operation: "connection_manager".to_string(),
                    source: Box::new(e),
                })?;

        Ok(Self { conn, _config: config })
    }
}

#[async_trait]
impl<C: Config + Send + Sync> Handler<C> for RedisCacheHandler {
    async fn handle_block(
        &self,
        ctx: &Context<C>,
        _events: &[ChainEvent<C>],
    ) -> Result<(), IndexerError> {
        let decoded_block = match ctx.peek_pipeline_data::<DecodedBlock>("decoded_block") {
            Some(b) => b,
            None => return Ok(()),
        };

        let decoded_events = ctx
            .peek_pipeline_data::<Vec<DecodedEvent>>("decoded_events")
            .unwrap_or_default();

        let block_json = serde_json::to_string(&decoded_block)?;
        let events_json = serde_json::to_string(&decoded_events)?;

        let mut conn = self.conn.clone();
        let mut pipe = redis::pipe();

        // Cache latest block pointer
        pipe.set_ex(keys::LATEST_BLOCK, &block_json, keys::LATEST_BLOCK_TTL);

        // Cache block by number
        pipe.set_ex(
            keys::block_key(decoded_block.block_number),
            &block_json,
            keys::BLOCK_TTL,
        );

        // Cache events by block number
        pipe.set_ex(
            keys::events_key(decoded_block.block_number),
            &events_json,
            keys::EVENTS_TTL,
        );

        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(|e| IndexerError::RedisError {
                operation: "cache_block".to_string(),
                source: Box::new(e),
            })?;

        debug!(
            block_number = decoded_block.block_number,
            event_count = decoded_events.len(),
            "Cached block in Redis"
        );

        Ok(())
    }
}
