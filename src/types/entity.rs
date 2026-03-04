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

use serde::{Deserialize, Serialize};

/// Bittensor account state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub coldkey: Option<String>,
    pub hotkey: Option<String>,
    pub free_balance: u128,
    pub staked_balance: u128,
    pub first_seen_block: u64,
    pub last_seen_block: u64,
}

/// Bittensor subnet state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subnet {
    pub netuid: u16,
    pub owner: String,
    pub tempo: u16,
    pub emission_value: u64,
    pub max_allowed_uids: u16,
    pub registered_at_block: u64,
    pub last_updated_block: u64,
}

/// Bittensor neuron state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub netuid: u16,
    pub uid: u16,
    pub hotkey: String,
    pub coldkey: String,
    pub stake: u128,
    pub trust: u64,
    pub consensus: u64,
    pub incentive: u64,
    pub dividends: u64,
    pub last_updated_block: u64,
}

/// Staking position for a coldkey-hotkey pair on a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakePosition {
    pub coldkey: String,
    pub hotkey: String,
    pub netuid: u16,
    pub amount: u128,
    pub last_updated_block: u64,
}
