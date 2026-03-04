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

use crate::config::IndexerConfig;
use crate::error::IndexerError;
use crate::handler::Handler;
use crate::retry::{retry_with_backoff, CircuitBreaker, RetryConfig};
use crate::storage::CheckpointStore;

use subxt::backend::legacy::LegacyRpcMethods;
use subxt::backend::BackendExt;
use subxt::client::RuntimeVersion;
use subxt::config::HashFor;
use subxt::metadata::types::SUPPORTED_METADATA_VERSIONS;
use subxt::{Config, Error as SubxtError, Metadata, OnlineClient};
use tracing::{debug, info, warn};

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

pub struct Indexer<C: Config> {
    pub(crate) retry_config: RetryConfig,
    pub(crate) circuit_breaker: CircuitBreaker,
    pub(crate) client: OnlineClient<C>,
    pub(crate) handlers: Vec<Arc<dyn Handler<C>>>,
    pub(crate) store: Box<dyn CheckpointStore>,
    pub(crate) config: IndexerConfig,
    pub(crate) max_blocks_per_minute: Option<u32>,
    pub(crate) metadata_cache: HashMap<u32, Metadata>,
}

impl<C> Indexer<C>
where
    C: Config + Send + Sync + 'static,
{
    pub async fn new(
        client: OnlineClient<C>,
        store: Box<dyn CheckpointStore>,
        config: IndexerConfig,
    ) -> Result<Self, IndexerError> {
        Ok(Self {
            retry_config: RetryConfig::default(),
            circuit_breaker: CircuitBreaker::new(3, Duration::from_secs(60)),
            client,
            handlers: Vec::new(),
            store,
            config,
            max_blocks_per_minute: None,
            metadata_cache: HashMap::new(),
        })
    }

    pub fn add_handler(&mut self, handler: impl Handler<C> + 'static) -> Result<(), IndexerError> {
        self.handlers.push(Arc::new(handler));
        Ok(())
    }

    pub fn add_handler_group(
        &mut self,
        group: crate::handler_group::HandlerGroup<C>,
    ) -> Result<(), IndexerError> {
        self.handlers.push(Arc::new(group));
        Ok(())
    }

    pub fn add_dyn_handler(&mut self, handler: Box<dyn Handler<C>>) -> Result<(), IndexerError> {
        self.handlers.push(Arc::from(handler));
        Ok(())
    }

    pub(crate) async fn with_circuit_breaker<F, Fut, T>(&self, op: F) -> Result<T, IndexerError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, IndexerError>>,
    {
        if self.circuit_breaker.is_open() {
            return Err(IndexerError::ConnectionFailed {
                url: self.config.node_url.clone(),
                source: Box::new(SubxtError::Other("circuit open".into())),
            });
        }
        let inner = op;
        let res = retry_with_backoff(inner, &self.retry_config, &self.circuit_breaker).await;
        match &res {
            Ok(_) => self.circuit_breaker.record_success(),
            Err(e) => {
                if crate::retry::is_retryable_error(e) {
                    self.circuit_breaker.record_failure();
                }
            }
        }
        res
    }

    pub(crate) async fn update_metadata(
        &mut self,
        rpc: &LegacyRpcMethods<C>,
        hash: HashFor<C>,
    ) -> Result<(), IndexerError> {
        let version = self
            .with_circuit_breaker(|| async {
                rpc.state_get_runtime_version(Some(hash))
                    .await
                    .map_err(|e| IndexerError::MetadataUpdateFailed {
                        source: Box::new(SubxtError::from(e)),
                    })
            })
            .await?;

        let current = self.client.runtime_version();
        if version.spec_version == current.spec_version {
            return Ok(());
        }
        if let Some(cached) = self.metadata_cache.get(&version.spec_version) {
            info!(
                old_spec = current.spec_version,
                new_spec = version.spec_version,
                "Runtime version changed (cached)"
            );
            self.client.set_metadata(cached.clone());
            self.client.set_runtime_version(RuntimeVersion {
                spec_version: version.spec_version,
                transaction_version: version.transaction_version,
            });
            return Ok(());
        }
        info!(
            old_spec = current.spec_version,
            new_spec = version.spec_version,
            block_hash = ?hash,
            "Runtime version changed"
        );
        let backend = self.client.backend();
        let mut metadata = None;
        for v in SUPPORTED_METADATA_VERSIONS {
            match backend.metadata_at_version(v, hash).await {
                Ok(m) => {
                    metadata = Some(m);
                    break;
                }
                Err(e) if e.to_string().contains("is not found") => {
                    debug!(
                        block_hash = ?hash,
                        "metadata_at_version not supported, using legacy"
                    );
                    break;
                }
                Err(e) => {
                    warn!(version = v, error = %e, block_hash = ?hash, "Metadata version fetch failed");
                    continue;
                }
            }
        }
        let metadata = match metadata {
            Some(m) => m,
            None => {
                self.with_circuit_breaker(|| async {
                    backend.legacy_metadata(hash).await.map_err(|e| {
                        IndexerError::MetadataUpdateFailed {
                            source: Box::new(e),
                        }
                    })
                })
                .await?
            }
        };
        self.metadata_cache.insert(version.spec_version, metadata.clone());
        self.client.set_metadata(metadata);
        self.client.set_runtime_version(RuntimeVersion {
            spec_version: version.spec_version,
            transaction_version: version.transaction_version,
        });
        Ok(())
    }
}
