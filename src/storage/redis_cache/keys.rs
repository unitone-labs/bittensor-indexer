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

// Key patterns
pub const LATEST_BLOCK: &str = "bittensor:latest_block";

pub fn block_key(number: u64) -> String {
    format!("bittensor:block:{}", number)
}

pub fn events_key(number: u64) -> String {
    format!("bittensor:events:{}", number)
}

pub fn account_key(address: &str) -> String {
    format!("bittensor:account:{}", address)
}

pub fn subnet_key(netuid: u16) -> String {
    format!("bittensor:subnet:{}", netuid)
}

// TTL constants in seconds
pub const LATEST_BLOCK_TTL: u64 = 12;
pub const BLOCK_TTL: u64 = 300;
pub const EVENTS_TTL: u64 = 120;
pub const ACCOUNT_TTL: u64 = 30;
pub const SUBNET_TTL: u64 = 60;
