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
    async_trait, ChainEvent, Context, Handler, IndexerBuilder, IndexerError, SubstrateConfig,
    WebSocketUrl,
};
use tracing::info;

struct PrintHandler;

#[async_trait]
impl Handler<SubstrateConfig> for PrintHandler {
    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        info!(
            block = ctx.block_number,
            pallet = event.pallet_name(),
            event = event.variant_name(),
            "Event"
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_level(true)
        .compact()
        .init();

    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(WebSocketUrl::parse(
            "wss://archive.chain.opentensor.ai:443",
        )?)
        .start_from_block(1017)
        .end_at_block(1133)
        .max_blocks_per_minute(12) // Optional throttling
        .add_handler(PrintHandler)
        .build()
        .await?;

    indexer.run().await?;
    Ok(())
}
