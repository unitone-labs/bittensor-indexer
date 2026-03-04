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

pub mod inserter;
pub mod schema;

use crate::error::IndexerError;
use crate::handler::{Context, Handler};
use crate::types::decoded::{DecodedBlock, DecodedEvent};
use crate::types::storage_config::ClickHouseConfig;
use crate::types::ChainEvent;

use self::inserter::BatchInserter;
use async_trait::async_trait;
use clickhouse::Client;
use subxt::Config;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, info};
use std::collections::HashMap;
use std::time::Duration;

/// Stores decoded blocks and events in ClickHouse with buffered batch inserts.
pub struct ClickHouseHandler {
    inserter: BatchInserter,
    config: ClickHouseConfig,
    block_buffer: Mutex<Vec<DecodedBlock>>,
    event_buffer: Mutex<Vec<(u64, DecodedEvent)>>,
    last_flush: Mutex<Instant>,
}

impl ClickHouseHandler {
    /// Create a new handler and run migrations.
    pub async fn new(config: ClickHouseConfig) -> Result<Self, IndexerError> {
        let client = Client::default()
            .with_url(&config.url)
            .with_database(&config.database);

        schema::run_migrations(&client).await?;

        let inserter = BatchInserter::new(client, config.batch_size);
        info!(
            batch_size = config.batch_size,
            flush_interval_ms = config.flush_interval_ms,
            "ClickHouse handler initialized"
        );

        Ok(Self {
            inserter,
            config,
            block_buffer: Mutex::new(Vec::new()),
            event_buffer: Mutex::new(Vec::new()),
            last_flush: Mutex::new(Instant::now()),
        })
    }

    /// Flush buffers if batch size or time threshold is reached.
    async fn maybe_flush(&self) -> Result<(), IndexerError> {
        let block_count = self.block_buffer.lock().await.len();
        let elapsed = self.last_flush.lock().await.elapsed();
        let interval = Duration::from_millis(self.config.flush_interval_ms);

        if block_count < self.inserter.batch_size() && elapsed < interval {
            return Ok(());
        }

        let blocks: Vec<DecodedBlock> = self.block_buffer.lock().await.drain(..).collect();
        let events: Vec<(u64, DecodedEvent)> = self.event_buffer.lock().await.drain(..).collect();

        if !blocks.is_empty() {
            self.inserter.flush_blocks(&blocks).await?;
        }

        let mut grouped: HashMap<u64, Vec<DecodedEvent>> = HashMap::new();
        for (block_num, event) in events {
            grouped.entry(block_num).or_default().push(event);
        }
        for (block_num, evts) in &grouped {
            self.inserter.flush_events(*block_num, evts).await?;
        }

        *self.last_flush.lock().await = Instant::now();
        debug!(
            blocks = blocks.len(),
            events = grouped.values().map(|v| v.len()).sum::<usize>(),
            "Flushed ClickHouse buffers"
        );

        Ok(())
    }
}

#[async_trait]
impl<C: Config + Send + Sync> Handler<C> for ClickHouseHandler {
    async fn handle_block(
        &self,
        ctx: &Context<C>,
        _events: &[ChainEvent<C>],
    ) -> Result<(), IndexerError> {
        if let Some(block) = ctx.peek_pipeline_data::<DecodedBlock>("decoded_block") {
            self.block_buffer.lock().await.push(block);
        }

        if let Some(events) = ctx.peek_pipeline_data::<Vec<DecodedEvent>>("decoded_events") {
            let mut buf = self.event_buffer.lock().await;
            for event in events {
                buf.push((ctx.block_number, event));
            }
        }

        self.maybe_flush().await
    }
}
