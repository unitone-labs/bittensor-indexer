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

#[cfg(not(feature = "sqlite"))]
fn main() {
    eprintln!("Run with sqlite feature: cargo run --example live_sqlite_checkpoint --features sqlite");
}

#[cfg(feature = "sqlite")]
use flamewire_bittensor_indexer::prelude::{
    async_trait, ChainEvent, Context, Handler, IndexerBuilder, IndexerError, SubstrateConfig,
    WebSocketUrl,
};
#[cfg(feature = "sqlite")]
use scale_value::{Value, ValueDef};
#[cfg(feature = "sqlite")]
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
#[cfg(feature = "sqlite")]
use std::error::Error;
#[cfg(feature = "sqlite")]
use tracing::{info, Level};

#[cfg(feature = "sqlite")]
const HANDLER_NAME: &str = "SQLiteEventStore";

#[cfg(feature = "sqlite")]
struct SQLiteEventStore {
    pool: SqlitePool,
}

#[cfg(feature = "sqlite")]
impl SQLiteEventStore {
    async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS indexed_blocks (
                block_number INTEGER PRIMARY KEY,
                block_hash TEXT NOT NULL,
                event_count INTEGER NOT NULL,
                indexed_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS indexed_events (
                block_number INTEGER NOT NULL,
                event_index INTEGER NOT NULL,
                pallet TEXT NOT NULL,
                variant TEXT NOT NULL,
                fields_json TEXT NOT NULL,
                indexed_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (block_number, event_index)
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    fn db_error(block: u64, source: sqlx::Error) -> IndexerError {
        IndexerError::HandlerFailed {
            handler: HANDLER_NAME.into(),
            block,
            source: Box::new(source),
        }
    }

    fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl Handler<SubstrateConfig> for SQLiteEventStore {
    async fn handle_block(
        &self,
        ctx: &Context<SubstrateConfig>,
        events: &[ChainEvent<SubstrateConfig>],
    ) -> Result<(), IndexerError> {
        sqlx::query(
            "INSERT INTO indexed_blocks (block_number, block_hash, event_count)
             VALUES (?, ?, ?)
             ON CONFLICT(block_number) DO UPDATE SET
               block_hash = excluded.block_hash,
               event_count = excluded.event_count,
               indexed_at = datetime('now')",
        )
        .bind(ctx.block_number as i64)
        .bind(format!("{:?}", ctx.block_hash))
        .bind(events.len() as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| Self::db_error(ctx.block_number, e))?;

        Ok(())
    }

    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        let fields_json = match event.field_values() {
            Ok(composite) => {
                let value: Value<u32> = Value {
                    value: ValueDef::Composite(composite),
                    context: 0,
                };
                serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string())
            }
            Err(_) => "null".to_string(),
        };

        sqlx::query(
            "INSERT INTO indexed_events (block_number, event_index, pallet, variant, fields_json)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(block_number, event_index) DO UPDATE SET
               pallet = excluded.pallet,
               variant = excluded.variant,
               fields_json = excluded.fields_json,
               indexed_at = datetime('now')",
        )
        .bind(ctx.block_number as i64)
        .bind(event.index as i64)
        .bind(event.pallet_name())
        .bind(event.variant_name())
        .bind(fields_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Self::db_error(ctx.block_number, e))?;

        Ok(())
    }
}

#[cfg(feature = "sqlite")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    let node_url = WebSocketUrl::parse(
        "wss://gateway-dev.flamewire.io/public/rpc/bittensor",
    )?;
    let db_url = "sqlite://./database/indexer.db";
    let event_store = SQLiteEventStore::new(db_url).await?;
    let verify_pool = event_store.pool();

    info!(
        db_url = db_url,
        "Starting indexer with SQLite storage for checkpoint + indexed data"
    );

    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(node_url)
        .with_sqlite(db_url)
        .start_from_block(0)
        .end_at_block(4)
        .add_handler(event_store)
        .build()
        .await?;

    indexer.run().await?;

    let block_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM indexed_blocks")
        .fetch_one(&verify_pool)
        .await?;
    let event_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM indexed_events")
        .fetch_one(&verify_pool)
        .await?;
    let checkpoint: Option<i64> =
        sqlx::query_scalar("SELECT last_block FROM indexer_checkpoint WHERE id = ?")
            .bind("bittensor")
            .fetch_optional(&verify_pool)
            .await?;

    info!(
        block_rows = block_rows,
        event_rows = event_rows,
        checkpoint = checkpoint.unwrap_or(-1),
        "Indexer run complete; rows persisted in SQLite"
    );

    Ok(())
}
