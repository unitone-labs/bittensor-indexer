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

/// Simple data structure extracted from Balances `Transfer` events
#[derive(Debug, Decode, DecodeAsType)]
struct TransferEvent {
    from: AccountId32,
    to: AccountId32,
    amount: u128,
}

impl StaticEvent for TransferEvent {
    const PALLET: &'static str = "Balances";
    const EVENT: &'static str = "Transfer";
}

/// Extract `TransferEvent` from the chain and store it in pipeline data
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
            // Pass data to next handlers in the pipeline
            ctx.set_pipeline_data("transfer", transfer);
        }
        Ok(())
    }
}

/// Print out transfer details stored in the pipeline
struct TransferPrinter;

#[async_trait::async_trait]
impl Handler<SubstrateConfig> for TransferPrinter {
    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context,
    ) -> Result<(), IndexerError> {
        if let Some(transfer) = ctx.get_pipeline_data::<TransferEvent>("transfer") {
            println!(
                "Block {} -> {} sent {} units to {}",
                ctx.block_number, transfer.from, transfer.amount, transfer.to
            );
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Build a sequential pipeline: Extract -> Transform -> Save
    let pipeline = HandlerGroup::new()
        .add(TransferExtractor)
        .pipe_to(TransferPrinter);

    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(WebSocketUrl::parse(
            "wss://archive.chain.opentensor.ai:443",
        )?)
        .start_from_block(1017)
        .end_at_block(1033)
        .add_handler_group(pipeline)
        .build()
        .await?;

    indexer.run().await?;
    Ok(())
}
