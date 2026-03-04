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
use serde_json::Value;

/// Block-level decoded data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedBlock {
    pub block_number: u64,
    pub block_hash: String,
    pub extrinsic_count: u32,
    pub event_count: u32,
}

/// A decoded extrinsic from a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedExtrinsic {
    pub index: u32,
    pub pallet: String,
    pub call: String,
    pub signer: Option<String>,
    pub args: Value,
}

/// A decoded event from a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedEvent {
    pub index: u32,
    pub pallet: String,
    pub variant: String,
    pub fields: Value,
}
