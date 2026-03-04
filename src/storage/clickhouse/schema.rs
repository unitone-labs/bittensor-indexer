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

use clickhouse::Client;
use tracing::{debug, info};

const CREATE_BLOCKS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS blocks (
        block_number UInt64,
        block_hash String,
        extrinsic_count UInt32,
        event_count UInt32
    ) ENGINE = ReplacingMergeTree() ORDER BY block_number
";

const CREATE_EVENTS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS events (
        block_number UInt64,
        event_index UInt32,
        pallet LowCardinality(String),
        variant LowCardinality(String),
        fields String
    ) ENGINE = MergeTree() ORDER BY (block_number, event_index)
";

const CREATE_EVENTS_PER_PALLET_HOURLY: &str = "
    CREATE TABLE IF NOT EXISTS events_per_pallet_hourly (
        hour DateTime,
        pallet LowCardinality(String),
        count UInt64
    ) ENGINE = SummingMergeTree() ORDER BY (hour, pallet)
";

const CREATE_MV_EVENTS_PER_PALLET_HOURLY: &str = "
    CREATE MATERIALIZED VIEW IF NOT EXISTS mv_events_per_pallet_hourly
    TO events_per_pallet_hourly AS
    SELECT
        toStartOfHour(now()) AS hour,
        pallet,
        count() AS count
    FROM events
    GROUP BY hour, pallet
";

/// Run all ClickHouse DDL migrations.
pub async fn run_migrations(client: &Client) -> Result<(), IndexerError> {
    let migrations = [
        ("blocks", CREATE_BLOCKS_TABLE),
        ("events", CREATE_EVENTS_TABLE),
        ("events_per_pallet_hourly", CREATE_EVENTS_PER_PALLET_HOURLY),
        ("mv_events_per_pallet_hourly", CREATE_MV_EVENTS_PER_PALLET_HOURLY),
    ];

    for (name, ddl) in &migrations {
        debug!(table = %name, "Running ClickHouse migration");
        client.query(ddl).execute().await.map_err(|e| {
            IndexerError::ClickHouseError {
                operation: format!("migrate_{}", name),
                source: Box::new(e),
            }
        })?;
    }

    info!("ClickHouse migrations complete");
    Ok(())
}
