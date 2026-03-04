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
use crate::handler::Context;
use crate::indexer::Indexer;
use crate::types::{BlockNumber, ChainEvent};

use subxt::backend::legacy::LegacyRpcMethods;
use subxt::config::HashFor;
use subxt::events::Events;
use subxt::Config;
use tracing::{debug, warn};

use std::time::{Duration, Instant};

impl<C> Indexer<C>
where
    C: Config + Send + Sync + 'static,
{
    pub(crate) async fn process_block(
        &mut self,
        rpc: &LegacyRpcMethods<C>,
        number: BlockNumber,
        hash: HashFor<C>,
    ) -> Result<(), IndexerError> {
        let block_start = Instant::now();
        self.update_metadata(rpc, hash).await?;
        let block = self.client.blocks().at(hash).await?;
        let events = block.events().await?;
        self.process_events(number, hash, &events).await?;
        self.with_circuit_breaker(|| async { self.store.store_checkpoint(number).await })
            .await?;

        let elapsed = block_start.elapsed();
        debug!(block = number, elapsed_ms = elapsed.as_millis() as u64, "Block processed");

        if let Some(min_dur) = self
            .max_blocks_per_minute
            .map(|bpm| Duration::from_secs_f64(60.0 / bpm as f64))
        {
            if elapsed < min_dur {
                let to_wait = min_dur - elapsed;
                debug!(wait_ms = to_wait.as_millis() as u64, "Throttling");
                tokio::time::sleep(to_wait).await;
            }
        }

        Ok(())
    }

    async fn process_events(
        &self,
        block_number: BlockNumber,
        block_hash: HashFor<C>,
        events: &Events<C>,
    ) -> Result<(), IndexerError> {
        let ctx = Context::new(block_number, block_hash);

        let mut decoded = Vec::new();
        for (index, evt_result) in events.iter().enumerate() {
            let evt = match evt_result {
                Ok(evt) => evt,
                Err(e) => {
                    warn!(block = block_number, block_hash = ?block_hash, event_index = index, "Event decode failed");
                    return Err(IndexerError::EventDecodingFailed {
                        pallet: "<unknown>".into(),
                        event: "<unknown>".into(),
                        block: block_number,
                        source: Box::new(e),
                    });
                }
            };
            decoded.push(ChainEvent::new(evt, index as u32));
        }

        for handler in &self.handlers {
            if let Err(e) = handler.handle_block(&ctx, &decoded).await {
                handler.handle_error(&e, &ctx).await;
            }
        }

        for chain_event in &decoded {
            let pallet = chain_event.pallet_name().to_string();
            let variant = chain_event.variant_name().to_string();

            for handler in &self.handlers {
                let filter = handler.event_filter();
                if filter.matches(&pallet, &variant) {
                    if let Err(e) = handler.handle_event(chain_event, &ctx).await {
                        handler.handle_error(&e, &ctx).await;
                    }
                }
            }
        }

        Ok(())
    }
}
