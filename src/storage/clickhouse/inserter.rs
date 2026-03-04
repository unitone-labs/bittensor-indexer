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

use crate::error::IndexerError;
use crate::types::decoded::{DecodedBlock, DecodedEvent};

use clickhouse::Client;
use serde::Serialize;
use tracing::debug;

#[derive(clickhouse::Row, Serialize)]
struct BlockRow {
    block_number: u64,
    block_hash: String,
    extrinsic_count: u32,
    event_count: u32,
}

#[derive(clickhouse::Row, Serialize)]
struct EventRow {
    block_number: u64,
    event_index: u32,
    pallet: String,
    variant: String,
    fields: String,
}

/// Handles batch inserts into ClickHouse tables.
pub struct BatchInserter {
    client: Client,
    batch_size: usize,
}

impl BatchInserter {
    pub fn new(client: Client, batch_size: usize) -> Self {
        Self { client, batch_size }
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// Insert decoded blocks into the `blocks` table.
    pub async fn flush_blocks(&self, blocks: &[DecodedBlock]) -> Result<(), IndexerError> {
        if blocks.is_empty() {
            return Ok(());
        }

        let mut insert = self
            .client
            .insert::<BlockRow>("blocks")
            .await
            .map_err(|e| IndexerError::ClickHouseError {
                operation: "insert_blocks_prepare".to_string(),
                source: Box::new(e),
            })?;

        for b in blocks {
            insert
                .write(&BlockRow {
                    block_number: b.block_number,
                    block_hash: b.block_hash.clone(),
                    extrinsic_count: b.extrinsic_count,
                    event_count: b.event_count,
                })
                .await
                .map_err(|e| IndexerError::ClickHouseError {
                    operation: "insert_blocks_write".to_string(),
                    source: Box::new(e),
                })?;
        }

        insert.end().await.map_err(|e| IndexerError::ClickHouseError {
            operation: "insert_blocks_end".to_string(),
            source: Box::new(e),
        })?;

        debug!(count = blocks.len(), "Flushed blocks to ClickHouse");
        Ok(())
    }

    /// Insert decoded events into the `events` table.
    pub async fn flush_events(
        &self,
        block_number: u64,
        events: &[DecodedEvent],
    ) -> Result<(), IndexerError> {
        if events.is_empty() {
            return Ok(());
        }

        let mut insert = self
            .client
            .insert::<EventRow>("events")
            .await
            .map_err(|e| IndexerError::ClickHouseError {
                operation: "insert_events_prepare".to_string(),
                source: Box::new(e),
            })?;

        for e in events {
            insert
                .write(&EventRow {
                    block_number,
                    event_index: e.index,
                    pallet: e.pallet.clone(),
                    variant: e.variant.clone(),
                    fields: e.fields.to_string(),
                })
                .await
                .map_err(|e| IndexerError::ClickHouseError {
                    operation: "insert_events_write".to_string(),
                    source: Box::new(e),
                })?;
        }

        insert.end().await.map_err(|e| IndexerError::ClickHouseError {
            operation: "insert_events_end".to_string(),
            source: Box::new(e),
        })?;

        debug!(
            block_number = block_number,
            count = events.len(),
            "Flushed events to ClickHouse"
        );
        Ok(())
    }
}
