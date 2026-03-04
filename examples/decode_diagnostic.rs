use flamewire_bittensor_indexer::prelude::{
    async_trait, BittensorDecoder, ChainEvent, Context, DecodedEvent, Handler,
    IndexerBuilder, IndexerError, SubstrateConfig, WebSocketUrl,
};

use serde_json::Value as JsonValue;
use tracing::{info, Level};

use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Instant;

struct EventTypeStats {
    count: u64,
    null_fields: u64,
    sample_keys: Option<String>,
}

struct DiagnosticState {
    blocks_processed: u64,
    total_events: u64,
    event_types: BTreeMap<String, EventTypeStats>,
    block_errors: Vec<(u64, String)>,
    start_time: Instant,
}

impl DiagnosticState {
    fn new() -> Self {
        Self {
            blocks_processed: 0,
            total_events: 0,
            event_types: BTreeMap::new(),
            block_errors: Vec::new(),
            start_time: Instant::now(),
        }
    }
}

struct DiagnosticHandler {
    state: Arc<Mutex<DiagnosticState>>,
}

fn extract_keys(val: &JsonValue) -> String {
    match val {
        JsonValue::Object(map) => {
            let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            format!("{{{}}}", keys.join(", "))
        }
        JsonValue::Null => "null".into(),
        _ => format!("{}", val),
    }
}

#[async_trait]
impl Handler<SubstrateConfig> for DiagnosticHandler {
    async fn handle_block(
        &self,
        ctx: &Context<SubstrateConfig>,
        _events: &[ChainEvent<SubstrateConfig>],
    ) -> Result<(), IndexerError> {
        let decoded_events: Vec<DecodedEvent> = ctx
            .peek_pipeline_data::<Vec<DecodedEvent>>("decoded_events")
            .unwrap_or_default();

        let mut state = self.state.lock().unwrap();
        state.blocks_processed += 1;
        state.total_events += decoded_events.len() as u64;

        for evt in &decoded_events {
            let key = format!("{}::{}", evt.pallet, evt.variant);
            let is_null = evt.fields == JsonValue::Null;
            let entry = state.event_types.entry(key).or_insert(EventTypeStats {
                count: 0,
                null_fields: 0,
                sample_keys: None,
            });
            entry.count += 1;
            if is_null {
                entry.null_fields += 1;
            }
            if entry.sample_keys.is_none() {
                entry.sample_keys = Some(extract_keys(&evt.fields));
            }
        }

        if state.blocks_processed % 100 == 0 {
            let bps = state.blocks_processed as f64
                / state.start_time.elapsed().as_secs_f64();
            info!(
                blocks = state.blocks_processed,
                events = state.total_events,
                bps = format!("{:.1}", bps),
                "Progress"
            );
        }
        Ok(())
    }

    async fn handle_error(
        &self,
        error: &IndexerError,
        ctx: &Context<SubstrateConfig>,
    ) {
        let mut state = self.state.lock().unwrap();
        state.block_errors.push((ctx.block_number, error.to_string()));
    }
}

fn print_report(state: &DiagnosticState) {
    let elapsed = state.start_time.elapsed().as_secs_f64();
    let bps = state.blocks_processed as f64 / elapsed;
    let total_null: u64 = state.event_types.values().map(|s| s.null_fields).sum();
    let null_pct = if state.total_events > 0 {
        total_null as f64 / state.total_events as f64 * 100.0
    } else {
        0.0
    };
    let unique_pallets: std::collections::BTreeSet<&str> = state
        .event_types
        .keys()
        .filter_map(|k| k.split("::").next())
        .collect();

    println!("\n=== Bittensor Decode Diagnostic: Blocks 1-1000 ===\n");
    println!("Blocks processed:    {}", state.blocks_processed);
    println!("Total events:        {}", state.total_events);
    println!("Unique pallets:      {}", unique_pallets.len());
    println!("Unique event types:  {}", state.event_types.len());
    println!("Throughput:          {:.1} blocks/sec", bps);
    println!(
        "Null-field events:   {} / {} ({:.2}%)",
        total_null, state.total_events, null_pct
    );

    println!("\n--- Event Type Breakdown ---");
    println!(
        "{:<24} {:<28} {:>6} {:>8}   Sample Fields",
        "Pallet", "Variant", "Count", "Nulls"
    );
    for (key, stats) in &state.event_types {
        let parts: Vec<&str> = key.splitn(2, "::").collect();
        let (pallet, variant) = (parts[0], parts.get(1).unwrap_or(&"?"));
        let sample = stats.sample_keys.as_deref().unwrap_or("?");
        println!(
            "{:<24} {:<28} {:>6} {:>8}   {}",
            pallet, variant, stats.count, stats.null_fields, sample
        );
    }

    if state.block_errors.is_empty() {
        println!("\n--- Blocks With Decode Errors ---\n(none)");
    } else {
        println!("\n--- Blocks With Decode Errors ---");
        for (block, err) in &state.block_errors {
            println!("Block {}: {}", block, err);
        }
    }
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    let url = "wss://gateway-dev.flamewire.io/public/rpc/bittensor";
    let ws_url = WebSocketUrl::parse(url)?;

    let state = Arc::new(Mutex::new(DiagnosticState::new()));
    let handler = DiagnosticHandler {
        state: Arc::clone(&state),
    };

    info!(url = url, blocks = "1-1000", "Starting diagnostic run");

    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(ws_url)
        .start_from_block(1)
        .end_at_block(1000)
        .max_blocks_per_minute(600)
        .add_handler(BittensorDecoder)
        .add_handler(handler)
        .build()
        .await?;

    indexer.run().await?;

    let state = state.lock().unwrap();
    print_report(&state);

    Ok(())
}
