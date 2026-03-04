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

use sqlx::PgPool;

const CREATE_BLOCKS: &str = "
CREATE TABLE IF NOT EXISTS blocks (
    block_number BIGINT PRIMARY KEY,
    block_hash TEXT UNIQUE NOT NULL,
    extrinsic_count INT NOT NULL,
    event_count INT NOT NULL,
    spec_version INT,
    created_at TIMESTAMPTZ DEFAULT now()
)";

const CREATE_EXTRINSICS: &str = "
CREATE TABLE IF NOT EXISTS extrinsics (
    block_number BIGINT REFERENCES blocks(block_number),
    index INT NOT NULL,
    pallet TEXT NOT NULL,
    call TEXT NOT NULL,
    signer TEXT,
    args JSONB,
    PRIMARY KEY (block_number, index)
)";

const CREATE_EVENTS: &str = "
CREATE TABLE IF NOT EXISTS events (
    block_number BIGINT REFERENCES blocks(block_number),
    index INT NOT NULL,
    pallet TEXT NOT NULL,
    variant TEXT NOT NULL,
    fields JSONB,
    PRIMARY KEY (block_number, index)
)";

const CREATE_ACCOUNTS: &str = "
CREATE TABLE IF NOT EXISTS accounts (
    address TEXT PRIMARY KEY,
    coldkey TEXT,
    hotkey TEXT,
    free_balance NUMERIC NOT NULL,
    staked_balance NUMERIC NOT NULL,
    first_seen_block BIGINT NOT NULL,
    last_seen_block BIGINT NOT NULL
)";

const CREATE_SUBNETS: &str = "
CREATE TABLE IF NOT EXISTS subnets (
    netuid INT PRIMARY KEY,
    owner TEXT NOT NULL,
    tempo INT NOT NULL,
    emission_value BIGINT NOT NULL,
    max_allowed_uids INT NOT NULL,
    registered_at_block BIGINT NOT NULL,
    last_updated_block BIGINT NOT NULL
)";

const CREATE_NEURONS: &str = "
CREATE TABLE IF NOT EXISTS neurons (
    netuid INT NOT NULL,
    uid INT NOT NULL,
    hotkey TEXT NOT NULL,
    coldkey TEXT NOT NULL,
    stake NUMERIC NOT NULL,
    trust BIGINT NOT NULL,
    consensus BIGINT NOT NULL,
    incentive BIGINT NOT NULL,
    dividends BIGINT NOT NULL,
    last_updated_block BIGINT NOT NULL,
    PRIMARY KEY (netuid, uid)
)";

const CREATE_STAKE_POSITIONS: &str = "
CREATE TABLE IF NOT EXISTS stake_positions (
    coldkey TEXT NOT NULL,
    hotkey TEXT NOT NULL,
    netuid INT NOT NULL,
    amount NUMERIC NOT NULL,
    last_updated_block BIGINT NOT NULL,
    PRIMARY KEY (coldkey, hotkey, netuid)
)";

const CREATE_IDX_EXTRINSICS_PALLET: &str =
    "CREATE INDEX IF NOT EXISTS idx_extrinsics_pallet ON extrinsics (pallet)";

const CREATE_IDX_EXTRINSICS_SIGNER: &str =
    "CREATE INDEX IF NOT EXISTS idx_extrinsics_signer ON extrinsics (signer)";

const CREATE_IDX_EVENTS_PALLET_VARIANT: &str =
    "CREATE INDEX IF NOT EXISTS idx_events_pallet_variant ON events (pallet, variant)";

const CREATE_IDX_NEURONS_HOTKEY: &str =
    "CREATE INDEX IF NOT EXISTS idx_neurons_hotkey ON neurons (hotkey)";

/// Runs all DDL statements to create tables and indexes.
pub async fn run_migrations(pool: &PgPool) -> Result<(), IndexerError> {
    let statements = [
        CREATE_BLOCKS,
        CREATE_EXTRINSICS,
        CREATE_EVENTS,
        CREATE_ACCOUNTS,
        CREATE_SUBNETS,
        CREATE_NEURONS,
        CREATE_STAKE_POSITIONS,
        CREATE_IDX_EXTRINSICS_PALLET,
        CREATE_IDX_EXTRINSICS_SIGNER,
        CREATE_IDX_EVENTS_PALLET_VARIANT,
        CREATE_IDX_NEURONS_HOTKEY,
    ];

    for stmt in statements {
        sqlx::query(stmt).execute(pool).await?;
    }

    Ok(())
}
