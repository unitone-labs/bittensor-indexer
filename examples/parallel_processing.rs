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

use flamewire_bittensor_indexer::prelude::{
    async_trait, AccountId32, ChainEvent, Context, Decode, DecodeAsType, EventFilter, Handler,
    HandlerGroup, IndexerBuilder, IndexerError, StaticEvent, SubstrateConfig, WebSocketUrl,
};

#[allow(dead_code)]
#[derive(Debug, Decode, DecodeAsType, Clone)]
struct TransferEvent {
    from: AccountId32,
    to: AccountId32,
    amount: u128,
}

impl StaticEvent for TransferEvent {
    const PALLET: &'static str = "Balances";
    const EVENT: &'static str = "Transfer";
}

struct TransferExtractor;

#[async_trait]
impl Handler<SubstrateConfig> for TransferExtractor {
    fn event_filter(&self) -> EventFilter {
        EventFilter::event("Balances", "Transfer")
    }

    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context,
    ) -> Result<(), IndexerError> {
        if let Some(transfer) = event.as_event::<TransferEvent>()? {
            ctx.set_pipeline_data("transfer", transfer);
        }
        Ok(())
    }
}

struct DatabaseSaver;

#[async_trait]
impl Handler<SubstrateConfig> for DatabaseSaver {
    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context,
    ) -> Result<(), IndexerError> {
        if ctx
            .peek_pipeline_data::<TransferEvent>("transfer")
            .is_some()
        {
            println!("Saving transfer to database");
        }
        Ok(())
    }
}

struct BackupSaver;

#[async_trait]
impl Handler<SubstrateConfig> for BackupSaver {
    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context,
    ) -> Result<(), IndexerError> {
        if ctx
            .peek_pipeline_data::<TransferEvent>("transfer")
            .is_some()
        {
            println!("Writing transfer to backup");
        }
        Ok(())
    }
}

struct MetricsCollector;

#[async_trait]
impl Handler<SubstrateConfig> for MetricsCollector {
    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context,
    ) -> Result<(), IndexerError> {
        if ctx
            .peek_pipeline_data::<TransferEvent>("transfer")
            .is_some()
        {
            println!("Updating metrics");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Extract transfers then fan out work in parallel
    let handlers = HandlerGroup::new().add(TransferExtractor).pipe_to(
        HandlerGroup::parallel()
            .add(DatabaseSaver)
            .add(BackupSaver)
            .add(MetricsCollector),
    );

    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(WebSocketUrl::parse(
            "wss://archive.chain.opentensor.ai:443",
        )?)
        .start_from_block(1017)
        .end_at_block(1033)
        .add_handler_group(handlers)
        .build()
        .await?;

    indexer.run().await?;
    Ok(())
}
