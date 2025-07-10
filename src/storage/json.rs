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
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct JsonCheckpoint {
    last_block: u64,
}

pub struct JsonStore {
    path: PathBuf,
}

impl JsonStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path: PathBuf = path.into();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self { path }
    }
}

#[async_trait]
impl CheckpointStore for JsonStore {
    async fn load_checkpoint(&self) -> Result<Option<u64>, IndexerError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&self.path).map_err(|e| IndexerError::CheckpointError {
            operation: "load_checkpoint".into(),
            backend: "json".into(),
            source: Box::new(e),
        })?;
        let checkpoint: JsonCheckpoint = serde_json::from_str(&data)?;
        Ok(Some(checkpoint.last_block))
    }

    async fn store_checkpoint(&self, block: u64) -> Result<(), IndexerError> {
        let checkpoint = JsonCheckpoint { last_block: block };
        let json = serde_json::to_string_pretty(&checkpoint)?;
        let mut file = fs::File::create(&self.path).map_err(|e| IndexerError::CheckpointError {
            operation: "store_checkpoint".into(),
            backend: "json".into(),
            source: Box::new(e),
        })?;
        file.write_all(json.as_bytes())
            .map_err(|e| IndexerError::CheckpointError {
                operation: "store_checkpoint".into(),
                backend: "json".into(),
                source: Box::new(e),
            })?;
        Ok(())
    }
}
