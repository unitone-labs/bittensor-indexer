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
use crate::storage::CheckpointStore;
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub struct PostgreSQLStore {
    pool: PgPool,
}

impl PostgreSQLStore {
    pub async fn new(database_url: &str) -> Result<Self, IndexerError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| IndexerError::CheckpointError {
                operation: "connect".into(),
                backend: "postgres".into(),
                source: Box::new(e),
            })?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS indexer_checkpoint (
                id TEXT PRIMARY KEY,
                last_block BIGINT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| IndexerError::CheckpointError {
            operation: "init".into(),
            backend: "postgres".into(),
            source: Box::new(e),
        })?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl CheckpointStore for PostgreSQLStore {
    async fn load_checkpoint(&self) -> Result<Option<u64>, IndexerError> {
        let row: Option<i64> =
            sqlx::query_scalar("SELECT last_block FROM indexer_checkpoint WHERE id = $1")
                .bind("bittensor")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| IndexerError::CheckpointError {
                    operation: "load_checkpoint".into(),
                    backend: "postgres".into(),
                    source: Box::new(e),
                })?;

        Ok(row.map(|v| v as u64))
    }

    async fn store_checkpoint(&self, block: u64) -> Result<(), IndexerError> {
        sqlx::query(
            "INSERT INTO indexer_checkpoint (id, last_block) 
             VALUES ($1, $2) 
             ON CONFLICT (id) DO UPDATE SET last_block = EXCLUDED.last_block",
        )
        .bind("bittensor")
        .bind(block as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| IndexerError::CheckpointError {
            operation: "store_checkpoint".into(),
            backend: "postgres".into(),
            source: Box::new(e),
        })?;

        Ok(())
    }
}
