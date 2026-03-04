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
use crate::types::decoded::{DecodedBlock, DecodedEvent, DecodedExtrinsic};
use crate::types::entity::{Account, Neuron, StakePosition, Subnet};

use sqlx::PgConnection;

/// Insert or update a block record.
pub async fn upsert_block(
    conn: &mut PgConnection,
    block: &DecodedBlock,
) -> Result<(), IndexerError> {
    sqlx::query(
        "INSERT INTO blocks (block_number, block_hash, extrinsic_count, event_count) \
         VALUES ($1, $2, $3, $4) ON CONFLICT (block_number) DO UPDATE SET \
         block_hash = EXCLUDED.block_hash, extrinsic_count = EXCLUDED.extrinsic_count, \
         event_count = EXCLUDED.event_count",
    )
    .bind(block.block_number as i64)
    .bind(&block.block_hash)
    .bind(block.extrinsic_count as i32)
    .bind(block.event_count as i32)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Insert extrinsics in batch for a given block.
pub async fn batch_insert_extrinsics(
    conn: &mut PgConnection,
    block_number: u64,
    extrinsics: &[DecodedExtrinsic],
) -> Result<(), IndexerError> {
    for ext in extrinsics {
        sqlx::query(
            "INSERT INTO extrinsics (block_number, index, pallet, call, signer, args) \
             VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (block_number, index) DO UPDATE SET \
             pallet = EXCLUDED.pallet, call = EXCLUDED.call, \
             signer = EXCLUDED.signer, args = EXCLUDED.args",
        )
        .bind(block_number as i64)
        .bind(ext.index as i32)
        .bind(&ext.pallet)
        .bind(&ext.call)
        .bind(&ext.signer)
        .bind(&ext.args)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Insert events in batch for a given block.
pub async fn batch_insert_events(
    conn: &mut PgConnection,
    block_number: u64,
    events: &[DecodedEvent],
) -> Result<(), IndexerError> {
    for event in events {
        sqlx::query(
            "INSERT INTO events (block_number, index, pallet, variant, fields) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (block_number, index) DO UPDATE SET \
             pallet = EXCLUDED.pallet, variant = EXCLUDED.variant, fields = EXCLUDED.fields",
        )
        .bind(block_number as i64)
        .bind(event.index as i32)
        .bind(&event.pallet)
        .bind(&event.variant)
        .bind(&event.fields)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Insert or update an account record.
pub async fn upsert_account(
    conn: &mut PgConnection,
    account: &Account,
) -> Result<(), IndexerError> {
    sqlx::query(
        "INSERT INTO accounts (address, coldkey, hotkey, free_balance, staked_balance, \
         first_seen_block, last_seen_block) VALUES ($1, $2, $3, $4::NUMERIC, $5::NUMERIC, $6, $7) \
         ON CONFLICT (address) DO UPDATE SET \
         coldkey = COALESCE(EXCLUDED.coldkey, accounts.coldkey), \
         hotkey = COALESCE(EXCLUDED.hotkey, accounts.hotkey), \
         free_balance = EXCLUDED.free_balance, staked_balance = EXCLUDED.staked_balance, \
         last_seen_block = EXCLUDED.last_seen_block",
    )
    .bind(&account.address)
    .bind(&account.coldkey)
    .bind(&account.hotkey)
    .bind(account.free_balance.to_string())
    .bind(account.staked_balance.to_string())
    .bind(account.first_seen_block as i64)
    .bind(account.last_seen_block as i64)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Insert or update a neuron record.
pub async fn upsert_neuron(
    conn: &mut PgConnection,
    neuron: &Neuron,
) -> Result<(), IndexerError> {
    sqlx::query(
        "INSERT INTO neurons (netuid, uid, hotkey, coldkey, stake, trust, \
         consensus, incentive, dividends, last_updated_block) \
         VALUES ($1, $2, $3, $4, $5::NUMERIC, $6, $7, $8, $9, $10) \
         ON CONFLICT (netuid, uid) DO UPDATE SET \
         hotkey = EXCLUDED.hotkey, coldkey = EXCLUDED.coldkey, stake = EXCLUDED.stake, \
         trust = EXCLUDED.trust, consensus = EXCLUDED.consensus, \
         incentive = EXCLUDED.incentive, dividends = EXCLUDED.dividends, \
         last_updated_block = EXCLUDED.last_updated_block",
    )
    .bind(neuron.netuid as i32)
    .bind(neuron.uid as i32)
    .bind(&neuron.hotkey)
    .bind(&neuron.coldkey)
    .bind(neuron.stake.to_string())
    .bind(neuron.trust as i64)
    .bind(neuron.consensus as i64)
    .bind(neuron.incentive as i64)
    .bind(neuron.dividends as i64)
    .bind(neuron.last_updated_block as i64)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Insert or update a subnet record.
pub async fn upsert_subnet(
    conn: &mut PgConnection,
    subnet: &Subnet,
) -> Result<(), IndexerError> {
    sqlx::query(
        "INSERT INTO subnets (netuid, owner, tempo, emission_value, \
         max_allowed_uids, registered_at_block, last_updated_block) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (netuid) DO UPDATE SET \
         owner = EXCLUDED.owner, tempo = EXCLUDED.tempo, \
         emission_value = EXCLUDED.emission_value, \
         max_allowed_uids = EXCLUDED.max_allowed_uids, \
         last_updated_block = EXCLUDED.last_updated_block",
    )
    .bind(subnet.netuid as i32)
    .bind(&subnet.owner)
    .bind(subnet.tempo as i32)
    .bind(subnet.emission_value as i64)
    .bind(subnet.max_allowed_uids as i32)
    .bind(subnet.registered_at_block as i64)
    .bind(subnet.last_updated_block as i64)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Insert or update a stake position record.
pub async fn upsert_stake(
    conn: &mut PgConnection,
    stake: &StakePosition,
) -> Result<(), IndexerError> {
    sqlx::query(
        "INSERT INTO stake_positions (coldkey, hotkey, netuid, amount, last_updated_block) \
         VALUES ($1, $2, $3, $4::NUMERIC, $5) ON CONFLICT (coldkey, hotkey, netuid) DO UPDATE SET \
         amount = EXCLUDED.amount, last_updated_block = EXCLUDED.last_updated_block",
    )
    .bind(&stake.coldkey)
    .bind(&stake.hotkey)
    .bind(stake.netuid as i32)
    .bind(stake.amount.to_string())
    .bind(stake.last_updated_block as i64)
    .execute(&mut *conn)
    .await?;
    Ok(())
}
