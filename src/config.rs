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
use crate::types::BlockNumber;

/// Configuration for the [`Indexer`](crate::indexer::Indexer).
pub struct IndexerConfig {
    pub node_url: String,
    pub database_url: Option<String>,
    pub start_block: Option<BlockNumber>,
    pub end_block: Option<BlockNumber>,
}

impl IndexerConfig {
    /// Create a new [`IndexerConfigBuilder`].
    pub fn builder() -> IndexerConfigBuilder {
        IndexerConfigBuilder::new()
    }

    /// Validate this configuration.
    pub fn validate(&self) -> Result<(), IndexerError> {
        if self.node_url.trim().is_empty() {
            return Err(IndexerError::invalid_config("node_url", "cannot be empty"));
        }

        if !self.node_url.starts_with("ws://") && !self.node_url.starts_with("wss://") {
            return Err(IndexerError::invalid_config(
                "node_url",
                "must start with ws:// or wss://",
            ));
        }

        if let Some(db) = &self.database_url {
            if db.trim().is_empty() {
                return Err(IndexerError::invalid_config(
                    "database_url",
                    "cannot be empty",
                ));
            }
        }

        if let (Some(start), Some(end)) = (self.start_block, self.end_block) {
            if end < start {
                return Err(IndexerError::invalid_config(
                    "end_block",
                    "must be greater than or equal to start_block",
                ));
            }
        }

        Ok(())
    }
}

/// Builder pattern for [`IndexerConfig`].
pub struct IndexerConfigBuilder {
    node_url: String,
    database_url: Option<String>,
    start_block: Option<BlockNumber>,
    end_block: Option<BlockNumber>,
}

impl Default for IndexerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexerConfigBuilder {
    /// Create a new builder with default values.
    pub fn new() -> Self {
        Self {
            node_url: String::new(),
            database_url: None,
            start_block: None,
            end_block: None,
        }
    }

    /// Set the node URL.
    pub fn node_url(mut self, url: impl Into<String>) -> Self {
        self.node_url = url.into();
        self
    }

    /// Configure a PostgreSQL backend.
    pub fn with_postgres(mut self, url: impl Into<String>) -> Self {
        self.database_url = Some(url.into());
        self
    }

    /// Configure a SQLite backend.
    pub fn with_sqlite(mut self, url: impl Into<String>) -> Self {
        self.database_url = Some(url.into());
        self
    }

    /// Start indexing from the specified block.
    pub fn start_from_block(mut self, block: BlockNumber) -> Self {
        self.start_block = Some(block);
        self
    }

    /// End indexing at the specified block.
    pub fn end_at_block(mut self, block: BlockNumber) -> Self {
        self.end_block = Some(block);
        self
    }

    /// Build the configuration and validate it.
    pub fn build(self) -> Result<IndexerConfig, IndexerError> {
        let config = IndexerConfig {
            node_url: self.node_url,
            database_url: self.database_url,
            start_block: self.start_block,
            end_block: self.end_block,
        };
        config.validate()?;
        Ok(config)
    }
}
