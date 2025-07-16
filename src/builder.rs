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

use std::marker::PhantomData;

use subxt::Config;
use subxt::OnlineClient;

use crate::config::IndexerConfig;
use crate::error::IndexerError;
use crate::handler::Handler;
use crate::indexer::Indexer;
use crate::storage::init::init_store;
use crate::types::BlockNumber;
use crate::validated_types::WebSocketUrl;

/// Convenient builder for creating an [`Indexer`].
pub struct IndexerBuilder<C: Config> {
    node_url: Option<WebSocketUrl>,
    database_url: Option<String>,
    start_block: Option<BlockNumber>,
    end_block: Option<BlockNumber>,
    max_blocks_per_minute: Option<u32>,
    handlers: Vec<Box<dyn Handler<C>>>,
    _marker: PhantomData<C>,
}

impl<C> Default for IndexerBuilder<C>
where
    C: Config + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> IndexerBuilder<C>
where
    C: Config + Send + Sync + 'static,
{
    /// Create a new builder with default options.
    pub fn new() -> Self {
        Self {
            node_url: None,
            database_url: None,
            start_block: None,
            end_block: None,
            max_blocks_per_minute: None,
            handlers: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Connect to the given websocket URL.
    pub fn connect(mut self, url: WebSocketUrl) -> Self {
        self.node_url = Some(url);
        self
    }

    /// Use a PostgreSQL store.
    pub fn with_postgres(mut self, url: impl Into<String>) -> Self {
        self.database_url = Some(url.into());
        self
    }

    /// Use a SQLite store.
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

    /// Set a maximum number of blocks to process per second.
    pub fn max_blocks_per_minute(mut self, value: u32) -> Self {
        self.max_blocks_per_minute = Some(value);
        self
    }

    /// Add a handler to the indexer.
    pub fn add_handler(mut self, handler: impl Handler<C> + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// Add a [`HandlerGroup`] to the indexer.
    pub fn add_handler_group(mut self, group: crate::handler_group::HandlerGroup<C>) -> Self {
        self.handlers.push(Box::new(group));
        self
    }

    /// Build the indexer.
    pub async fn build(self) -> Result<Indexer<C>, IndexerError> {
        let node_url = self
            .node_url
            .ok_or_else(|| IndexerError::invalid_config("node_url", "missing"))?;

        let client = OnlineClient::<C>::from_insecure_url(node_url.as_str()).await?;
        let store = init_store(self.database_url.clone()).await?;

        let mut cfg_builder = IndexerConfig::builder().node_url(node_url.as_str());
        if let Some(ref db) = self.database_url {
            cfg_builder = cfg_builder.with_postgres(db);
        }
        if let Some(block) = self.start_block {
            cfg_builder = cfg_builder.start_from_block(block);
        }
        if let Some(block) = self.end_block {
            cfg_builder = cfg_builder.end_at_block(block);
        }
        let config = cfg_builder.build()?;

        let mut indexer = Indexer::new(client, store, config).await?;
        indexer.max_blocks_per_minute = self.max_blocks_per_minute;
        for h in self.handlers {
            indexer.add_dyn_handler(h)?;
        }

        Ok(indexer)
    }
}
