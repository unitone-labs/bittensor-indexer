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
use crate::indexer::Indexer;

use subxt::backend::{legacy::LegacyRpcMethods, rpc::RpcClient};
use subxt::config::Header;
use subxt::{Config, Error as SubxtError};
use tracing::warn;

impl<C> Indexer<C>
where
    C: Config + Send + Sync + 'static,
{
    pub async fn run(&mut self) -> Result<(), IndexerError> {
        let rpc_client = self
            .with_circuit_breaker(|| async {
                RpcClient::from_insecure_url(&self.config.node_url)
                    .await
                    .map_err(|e| IndexerError::ConnectionFailed {
                        url: self.config.node_url.clone(),
                        source: Box::new(SubxtError::from(e)),
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
                    .map_err(|e| IndexerError::from(SubxtError::from(e)))
            })
            .await?;
        let finalized_header = self
            .with_circuit_breaker(|| async {
                rpc.chain_get_header(Some(finalized_hash))
                    .await
                    .map_err(|e| IndexerError::from(SubxtError::from(e)))
            })
            .await?
            .ok_or(IndexerError::BlockNotFound { block: 0 })?;
        let latest_number = finalized_header.number().into();

        while current_block <= latest_number {
            if end_block.is_some_and(|end| current_block > end) {
                return Ok(());
            }
            let hash = self
                .with_circuit_breaker(|| async {
                    rpc.chain_get_block_hash(Some(current_block.into()))
                        .await
                        .map_err(|e| IndexerError::from(SubxtError::from(e)))
                })
                .await?
                .ok_or(IndexerError::BlockNotFound {
                    block: current_block,
                })?;

            self.process_block(&rpc, current_block, hash).await?;
            current_block += 1;
        }

        // update_metadata() handles historical blocks per-block hash.
        // perform_runtime_updates() handles live blocks reactively.
        // Both are safe: OnlineClient uses internal locking, duplicate sets are idempotent.
        let updater = self.client.updater();
        tokio::spawn(async move {
            if let Err(e) = updater.perform_runtime_updates().await {
                warn!(error = %e, "Runtime updater exited");
            }
        });

        let mut sub = self.client.blocks().subscribe_finalized().await?;
        while let Some(block) = sub.next().await {
            let block = block?;
            let number = block.header().number().into();

            if number < current_block {
                continue;
            }

            self.process_block(&rpc, number, block.hash()).await?;
            current_block = number + 1;

            if end_block.is_some_and(|end| number >= end) {
                break;
            }
        }

        Ok(())
    }
}
