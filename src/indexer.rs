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
use crate::handler::{Context, Handler};
use crate::retry::{retry_with_backoff, CircuitBreaker, RetryConfig};
use crate::storage::CheckpointStore;
use crate::types::{BlockNumber, ChainEvent};
use std::sync::Arc;
use subxt::backend::BackendExt;
use subxt::config::HashFor;
use subxt::config::Header;
use subxt::events::Events;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    client::RuntimeVersion,
    Config, OnlineClient,
};
use tracing::warn;

pub struct Indexer<C: Config> {
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    client: OnlineClient<C>,
    handlers: Vec<Arc<dyn Handler<C>>>,
    store: Box<dyn CheckpointStore>,
    config: IndexerConfig,
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
            circuit_breaker: CircuitBreaker::new(3, std::time::Duration::from_secs(60)),
            client,
            handlers: Vec::new(),
            store,
            config,
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

    async fn with_circuit_breaker<F, Fut, T>(&self, op: F) -> Result<T, IndexerError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, IndexerError>>,
    {
        if self.circuit_breaker.is_open() {
            return Err(IndexerError::ConnectionFailed {
                url: self.config.node_url.clone(),
                source: Box::new(subxt::Error::Other("circuit open".into())),
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

    async fn update_metadata(
        &self,
        rpc: &LegacyRpcMethods<C>,
        hash: HashFor<C>,
    ) -> Result<(), IndexerError> {
        let version = self
            .with_circuit_breaker(|| async {
                rpc.state_get_runtime_version(Some(hash))
                    .await
                    .map_err(|e| IndexerError::MetadataUpdateFailed {
                        source: Box::new(subxt::Error::from(e)),
                    })
            })
            .await?;

        let current = self.client.runtime_version();
        if version.spec_version != current.spec_version {
            use subxt::metadata::types::SUPPORTED_METADATA_VERSIONS;
            let backend = self.client.backend();
            let mut metadata = None;
            for v in SUPPORTED_METADATA_VERSIONS {
                match backend.metadata_at_version(v, hash).await {
                    Ok(m) => {
                        metadata = Some(m);
                        break;
                    }
                    Err(_) => continue,
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
            self.client.set_metadata(metadata);
            self.client.set_runtime_version(RuntimeVersion {
                spec_version: version.spec_version,
                transaction_version: version.transaction_version,
            });
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), IndexerError> {
        let rpc_client = self
            .with_circuit_breaker(|| async {
                RpcClient::from_insecure_url(&self.config.node_url)
                    .await
                    .map_err(|e| IndexerError::ConnectionFailed {
                        url: self.config.node_url.clone(),
                        source: Box::new(subxt::Error::from(e)),
                    })
            })
            .await?;
        let rpc = LegacyRpcMethods::<C>::new(rpc_client);

        let mut current_block = match self.config.start_block {
            Some(n) => n,
            None => self
                .with_circuit_breaker(|| async { self.store.load_checkpoint().await })
                .await?
                .unwrap_or(0),
        };
        let end_block = self.config.end_block;

        let finalized_hash = self
            .with_circuit_breaker(|| async {
                rpc.chain_get_finalized_head()
                    .await
                    .map_err(|e| IndexerError::from(subxt::Error::from(e)))
            })
            .await?;
        let finalized_header = self
            .with_circuit_breaker(|| async {
                rpc.chain_get_header(Some(finalized_hash))
                    .await
                    .map_err(|e| IndexerError::from(subxt::Error::from(e)))
            })
            .await?
            .ok_or(IndexerError::BlockNotFound { block: 0 })?;
        let latest_number = finalized_header.number().into();

        while current_block <= latest_number {
            if let Some(end) = end_block {
                if current_block > end {
                    return Ok(());
                }
            }
            let hash = self
                .with_circuit_breaker(|| async {
                    rpc.chain_get_block_hash(Some(current_block.into()))
                        .await
                        .map_err(|e| IndexerError::from(subxt::Error::from(e)))
                })
                .await?
                .ok_or(IndexerError::BlockNotFound {
                    block: current_block,
                })?;
            self.update_metadata(&rpc, hash).await?;
            let block = self.client.blocks().at(hash).await?;
            let events = block.events().await?;
            self.process_events(current_block, &events).await?;
            self.with_circuit_breaker(|| async {
                self.store.store_checkpoint(current_block).await
            })
            .await?;
            current_block += 1;
        }

        let updater = self.client.updater();
        tokio::spawn(async move {
            if let Err(e) = updater.perform_runtime_updates().await {
                warn!(target: "indexer", "runtime updater exited: {:?}", e);
            }
        });

        let mut sub = self.client.blocks().subscribe_finalized().await?;
        while let Some(block) = sub.next().await {
            let block = block?;
            let number = block.header().number().into();

            if number < current_block {
                continue;
            }

            self.update_metadata(&rpc, block.hash()).await?;
            let events = block.events().await?;
            self.process_events(number, &events).await?;
            self.with_circuit_breaker(|| async { self.store.store_checkpoint(number).await })
                .await?;

            current_block = number + 1;

            if let Some(end) = end_block {
                if number >= end {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_events(
        &self,
        block_number: BlockNumber,
        events: &Events<C>,
    ) -> Result<(), IndexerError> {
        let ctx = Context::new(block_number);

        for handler in &self.handlers {
            if let Err(e) = handler.handle_block(&ctx, events).await {
                handler.handle_error(&e, &ctx).await;
            }
        }

        for evt_result in events.iter() {
            let evt = match evt_result {
                Ok(evt) => evt,
                Err(e) => {
                    return Err(IndexerError::EventDecodingFailed {
                        pallet: "<unknown>".into(),
                        event: "<unknown>".into(),
                        block: block_number,
                        source: Box::new(e),
                    });
                }
            };
            let pallet = evt.pallet_name().to_string();
            let variant = evt.variant_name().to_string();
            let chain_event = ChainEvent::new(evt);

            for handler in &self.handlers {
                let filter = handler.event_filter();
                if filter.matches(&pallet, &variant) {
                    if let Err(e) = handler.handle_event(&chain_event, &ctx).await {
                        handler.handle_error(&e, &ctx).await;
                    }
                }
            }
        }

        Ok(())
    }
}
