use flamewire_bittensor_indexer::prelude::{
    async_trait, BittensorDecoder, ChainEvent, Context, Handler,
    IndexerBuilder, IndexerError, SubstrateConfig, WebSocketUrl,
};
use tracing::{debug, info, Level};

use std::error::Error;

struct EventLogger;

#[async_trait]
impl Handler<SubstrateConfig> for EventLogger {
    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        info!(
            block = ctx.block_number,
            pallet = event.pallet_name(),
            variant = event.variant_name(),
            index = event.index,
            "Live event"
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .with_level(true)
        .compact()
        .init();

    let url = "wss://gateway-dev.flamewire.io/public/rpc/bittensor";
    info!(url = url, "Connecting to live gateway");

    let ws_url = WebSocketUrl::parse(url)?;

    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(ws_url)
        .start_from_block(0)
        .end_at_block(2)
        .add_handler(BittensorDecoder)
        .add_handler(EventLogger)
        .build()
        .await?;

    debug!("Indexer built, starting run");
    indexer.run().await?;

    info!("Live test completed");
    Ok(())
}
