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

#[path = "../common/mod.rs"]
mod common;
use common::*;

use flamewire_bittensor_indexer::decoder::BittensorDecoder;
use flamewire_bittensor_indexer::handler::{Context, Handler};
use flamewire_bittensor_indexer::types::decoded::{DecodedBlock, DecodedEvent};
use flamewire_bittensor_indexer::types::ChainEvent;

use subxt::config::substrate::SubstrateConfig;
use subxt::events::Phase;
use subxt::utils::H256;

fn make_context(block_number: u64) -> Context<SubstrateConfig> {
    Context::<SubstrateConfig>::new(block_number, H256::zero())
}

fn make_chain_events() -> Vec<ChainEvent<SubstrateConfig>> {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    evs.iter()
        .enumerate()
        .map(|(i, e)| ChainEvent::new(e.unwrap(), i as u32))
        .collect()
}

#[test]
fn event_filter_returns_all() {
    let decoder = BittensorDecoder;
    let filter = Handler::<SubstrateConfig>::event_filter(&decoder);
    assert!(filter.matches("AnyPallet", "AnyEvent"));
    assert!(filter.matches("System", "ExtrinsicSuccess"));
}

#[tokio::test]
async fn handle_block_stores_decoded_block() {
    let decoder = BittensorDecoder;
    let ctx = make_context(42);
    let chain_events = make_chain_events();

    decoder.handle_block(&ctx, &chain_events).await.unwrap();

    let block: DecodedBlock = ctx
        .get_pipeline_data("decoded_block")
        .expect("decoded_block missing from pipeline");
    assert_eq!(block.block_number, 42);
    assert_eq!(block.event_count, chain_events.len() as u32);
    assert_eq!(block.extrinsic_count, 0);
}

#[tokio::test]
async fn handle_block_formats_block_hash() {
    let decoder = BittensorDecoder;
    let ctx = make_context(1);
    let chain_events = make_chain_events();

    decoder.handle_block(&ctx, &chain_events).await.unwrap();

    let block: DecodedBlock = ctx.get_pipeline_data("decoded_block").unwrap();
    // H256::zero() should format as a hex string containing zeros
    assert!(block.block_hash.contains('0'));
}

#[tokio::test]
async fn handle_block_empty_events() {
    let decoder = BittensorDecoder;
    let ctx = make_context(10);
    let empty: Vec<ChainEvent<SubstrateConfig>> = vec![];

    decoder.handle_block(&ctx, &empty).await.unwrap();

    let block: DecodedBlock = ctx.get_pipeline_data("decoded_block").unwrap();
    assert_eq!(block.event_count, 0);
}

#[tokio::test]
async fn handle_event_stores_decoded_event() {
    let decoder = BittensorDecoder;
    let ctx = make_context(5);
    let chain_events = make_chain_events();

    for ce in &chain_events {
        decoder.handle_event(ce, &ctx).await.unwrap();
    }

    let decoded: Vec<DecodedEvent> = ctx
        .get_pipeline_data("decoded_events")
        .expect("decoded_events missing from pipeline");
    assert_eq!(decoded.len(), chain_events.len());
    assert_eq!(decoded[0].pallet, "Test");
    assert_eq!(decoded[0].variant, "A");
    assert_eq!(decoded[0].index, 0);
}

#[tokio::test]
async fn handle_event_accumulates_multiple() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![
            EventRecord::new(Phase::Initialization, TestEvent::A(1)),
            EventRecord::new(Phase::ApplyExtrinsic(0), TestEvent::B(true)),
        ],
    );
    let chain_events: Vec<ChainEvent<SubstrateConfig>> = evs
        .iter()
        .enumerate()
        .map(|(i, e)| ChainEvent::new(e.unwrap(), i as u32))
        .collect();

    let decoder = BittensorDecoder;
    let ctx = make_context(7);

    for ce in &chain_events {
        decoder.handle_event(ce, &ctx).await.unwrap();
    }

    let decoded: Vec<DecodedEvent> = ctx.get_pipeline_data("decoded_events").unwrap();
    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[0].variant, "A");
    assert_eq!(decoded[1].variant, "B");
}

#[tokio::test]
async fn handle_event_fields_not_null() {
    let decoder = BittensorDecoder;
    let ctx = make_context(1);
    let chain_events = make_chain_events();

    for ce in &chain_events {
        decoder.handle_event(ce, &ctx).await.unwrap();
    }

    let decoded: Vec<DecodedEvent> = ctx.get_pipeline_data("decoded_events").unwrap();
    // field_values() succeeds for TestEvent, so fields should not be null
    assert!(!decoded[0].fields.is_null());
}

#[tokio::test]
async fn full_pipeline_block_then_events() {
    let decoder = BittensorDecoder;
    let ctx = make_context(99);
    let chain_events = make_chain_events();

    decoder.handle_block(&ctx, &chain_events).await.unwrap();
    for ce in &chain_events {
        decoder.handle_event(ce, &ctx).await.unwrap();
    }

    let block: DecodedBlock = ctx.get_pipeline_data("decoded_block").unwrap();
    let events: Vec<DecodedEvent> =
        ctx.get_pipeline_data("decoded_events").unwrap();
    assert_eq!(block.block_number, 99);
    assert_eq!(events.len(), chain_events.len());
}
