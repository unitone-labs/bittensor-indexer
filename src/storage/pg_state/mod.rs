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
use crate::handler::{Context, Handler};
use crate::types::decoded::{DecodedBlock, DecodedEvent, DecodedExtrinsic};
use crate::types::storage_config::PgStateConfig;
use crate::types::ChainEvent;

use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use subxt::Config;
use tracing::{debug, info, warn};

mod parsers;
pub mod queries;
pub mod schema;

/// Handler that persists decoded block data to PostgreSQL.
pub struct PostgresStateHandler {
    pool: PgPool,
}

impl PostgresStateHandler {
    pub async fn new(config: &PgStateConfig) -> Result<Self, IndexerError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await
            .map_err(|e| IndexerError::Database(Box::new(e)))?;

        if config.run_migrations {
            schema::run_migrations(&pool).await?;
            info!("pg state migrations complete");
        }

        Ok(Self { pool })
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl<C: Config + Send + Sync> Handler<C> for PostgresStateHandler {
    async fn handle_block(
        &self,
        ctx: &Context<C>,
        _events: &[ChainEvent<C>],
    ) -> Result<(), IndexerError> {
        let decoded_block = match ctx.peek_pipeline_data::<DecodedBlock>("decoded_block") {
            Some(b) => b,
            None => {
                warn!(block = ctx.block_number, "no decoded_block in pipeline");
                return Ok(());
            }
        };

        let decoded_events = ctx
            .peek_pipeline_data::<Vec<DecodedEvent>>("decoded_events")
            .unwrap_or_default();

        let decoded_extrinsics = ctx
            .peek_pipeline_data::<Vec<DecodedExtrinsic>>("decoded_extrinsics")
            .unwrap_or_default();

        let mut tx = self.pool.begin().await?;

        queries::upsert_block(&mut *tx, &decoded_block).await?;
        queries::batch_insert_extrinsics(&mut *tx, ctx.block_number, &decoded_extrinsics).await?;
        queries::batch_insert_events(&mut *tx, ctx.block_number, &decoded_events).await?;

        for event in &decoded_events {
            self.extract_entity(&mut *tx, ctx.block_number, event)
                .await?;
        }

        tx.commit().await?;

        debug!(
            block = ctx.block_number,
            events = decoded_events.len(),
            extrinsics = decoded_extrinsics.len(),
            "pg state block persisted"
        );

        Ok(())
    }
}

impl PostgresStateHandler {
    async fn extract_entity(
        &self,
        conn: &mut sqlx::PgConnection,
        block_number: u64,
        event: &DecodedEvent,
    ) -> Result<(), IndexerError> {
        match (event.pallet.as_str(), event.variant.as_str()) {
            ("SubtensorModule", "NeuronRegistered") => {
                if let Some(neuron) = parsers::parse_neuron(&event.fields, block_number) {
                    queries::upsert_neuron(conn, &neuron).await?;
                }
            }
            ("SubtensorModule", "StakeAdded" | "StakeRemoved") => {
                if let Some(stake) = parsers::parse_stake(&event.fields, block_number) {
                    queries::upsert_stake(conn, &stake).await?;
                }
            }
            ("SubtensorModule", "NetworkAdded") => {
                if let Some(subnet) = parsers::parse_subnet(&event.fields, block_number) {
                    queries::upsert_subnet(conn, &subnet).await?;
                }
            }
            ("Balances", "Transfer" | "Deposit" | "Withdraw") => {
                if let Some(account) = parsers::parse_account(&event.fields, block_number) {
                    queries::upsert_account(conn, &account).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
