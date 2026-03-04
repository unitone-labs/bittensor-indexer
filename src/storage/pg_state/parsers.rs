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

use crate::types::entity::{Account, Neuron, StakePosition, Subnet};

use serde_json::Value;

/// Parse a neuron from NeuronRegistered event fields.
pub fn parse_neuron(fields: &Value, block: u64) -> Option<Neuron> {
    Some(Neuron {
        netuid: fields.get("netuid")?.as_u64()? as u16,
        uid: fields.get("uid")?.as_u64()? as u16,
        hotkey: fields.get("hotkey")?.as_str()?.to_string(),
        coldkey: fields
            .get("coldkey")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        stake: 0,
        trust: 0,
        consensus: 0,
        incentive: 0,
        dividends: 0,
        last_updated_block: block,
    })
}

/// Parse a stake position from StakeAdded/StakeRemoved event fields.
pub fn parse_stake(fields: &Value, block: u64) -> Option<StakePosition> {
    Some(StakePosition {
        coldkey: fields.get("coldkey")?.as_str()?.to_string(),
        hotkey: fields.get("hotkey")?.as_str()?.to_string(),
        netuid: fields
            .get("netuid")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16,
        amount: fields
            .get("amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u128,
        last_updated_block: block,
    })
}

/// Parse a subnet from NetworkAdded event fields.
pub fn parse_subnet(fields: &Value, block: u64) -> Option<Subnet> {
    Some(Subnet {
        netuid: fields.get("netuid")?.as_u64()? as u16,
        owner: fields
            .get("owner")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        tempo: 0,
        emission_value: 0,
        max_allowed_uids: 0,
        registered_at_block: block,
        last_updated_block: block,
    })
}

/// Parse an account from Balances Transfer/Deposit/Withdraw event fields.
pub fn parse_account(fields: &Value, block: u64) -> Option<Account> {
    let address = fields
        .get("who")
        .or_else(|| fields.get("account"))
        .and_then(|v| v.as_str())?
        .to_string();

    let amount = fields
        .get("amount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u128;

    Some(Account {
        address,
        coldkey: None,
        hotkey: None,
        free_balance: amount,
        staked_balance: 0,
        first_seen_block: block,
        last_seen_block: block,
    })
}
