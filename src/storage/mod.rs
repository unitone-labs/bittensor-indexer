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
use async_trait::async_trait;

pub mod init;

#[cfg(feature = "json-storage")]
pub mod json;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[async_trait]
pub trait CheckpointStore: Send + Sync {
    async fn load_checkpoint(&self) -> Result<Option<u64>, IndexerError>;
    async fn store_checkpoint(&self, block: u64) -> Result<(), IndexerError>;
}
