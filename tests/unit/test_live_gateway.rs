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

// Regression tests for v0.2.1 refactoring.
#[path = "../common/mod.rs"]
mod common;
use common::*;

use flamewire_bittensor_indexer::decoder::BittensorDecoder;
use flamewire_bittensor_indexer::error::IndexerError;
use flamewire_bittensor_indexer::handler::{Context, EventFilter, Handler};
use flamewire_bittensor_indexer::handler_group::HandlerGroup;
use flamewire_bittensor_indexer::types::decoded::{DecodedBlock, DecodedEvent};
use flamewire_bittensor_indexer::types::ChainEvent;
use flamewire_bittensor_indexer::validated_types::WebSocketUrl;

use subxt::config::substrate::SubstrateConfig;
use subxt::events::Phase;
use subxt::utils::H256;

// -- WebSocketUrl: gateway-style URLs parse correctly --

#[test]
fn wss_gateway_url_parses() {
    let url = WebSocketUrl::parse(
        "wss://gateway-dev.flamewire.io/public/rpc/bittensor/test",
    );
    assert!(url.is_ok());
    assert!(url.unwrap().as_str().starts_with("wss://"));
}

#[test]
fn wss_url_preserves_path() {
    let input = "wss://gateway-dev.flamewire.io/public/rpc/bittensor/fw_test";
    let url = WebSocketUrl::parse(input).unwrap();
    assert!(url.as_str().contains("/public/rpc/bittensor/fw_test"));
}

#[test]
fn non_ws_schemes_rejected() {
    assert!(WebSocketUrl::parse("http://node").is_err());
    assert!(WebSocketUrl::parse("https://node").is_err());
    assert!(WebSocketUrl::parse("").is_err());
    assert!(WebSocketUrl::parse("not-a-url").is_err());
}

// -- serde_json error variant (regression: was gated behind json-storage feature) --

#[test]
fn serde_json_error_converts_without_feature_gate() {
    let json_err = serde_json::from_str::<serde_json::Value>("bad").unwrap_err();
    let err: IndexerError = json_err.into();
    assert!(format!("{err}").contains("Serde JSON error"));
}

// -- ConditionalHandler extraction: still works after moving to its own module --

#[tokio::test]
async fn conditional_handler_filters_after_extraction() {
    let metadata = test_metadata::<TestEvent>();
    let ev_a = events(
        metadata.clone(),
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let ev_b = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::B(true))],
    );

    let cond_handler = MockHandler::new(EventFilter::all());
    let cond_events = cond_handler.events.clone();
    let uncond_handler = MockHandler::new(EventFilter::all());
    let uncond_events = uncond_handler.events.clone();

    let group = HandlerGroup::new()
        .add_conditional(
            cond_handler,
            |e: &ChainEvent<SubstrateConfig>| e.variant_name() == "A",
        )
        .add(uncond_handler);

    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());

    // Event A reaches both handlers (conditional predicate passes)
    let ce_a = ev_a.iter().next().unwrap().unwrap();
    group
        .handle_event(&ChainEvent::new(ce_a, 0), &ctx)
        .await
        .unwrap();

    let cond_count = cond_events.lock().unwrap().len();
    let uncond_count_after_a = uncond_events.lock().unwrap().len();
    assert!(cond_count > 0, "conditional handler should fire for A");
    assert!(uncond_count_after_a > 0, "unconditional should fire for A");

    // Event B skips conditional handler, reaches unconditional
    let ce_b = ev_b.iter().next().unwrap().unwrap();
    group
        .handle_event(&ChainEvent::new(ce_b, 0), &ctx)
        .await
        .unwrap();

    let cond_after_b = cond_events.lock().unwrap().len();
    let uncond_after_b = uncond_events.lock().unwrap().len();
    assert_eq!(cond_after_b, cond_count, "conditional should skip B");
    assert!(uncond_after_b > uncond_count_after_a, "unconditional fires for B");
}

// -- BittensorDecoder pipeline: block + event decode after refactoring --

fn decoder_ctx(block: u64) -> Context<SubstrateConfig> {
    Context::<SubstrateConfig>::new(block, H256::zero())
}

fn two_test_events() -> Vec<ChainEvent<SubstrateConfig>> {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![
            EventRecord::new(Phase::Initialization, TestEvent::A(42)),
            EventRecord::new(Phase::ApplyExtrinsic(0), TestEvent::B(true)),
        ],
    );
    evs.iter()
        .enumerate()
        .map(|(i, e)| ChainEvent::new(e.unwrap(), i as u32))
        .collect()
}

#[tokio::test]
async fn decoder_pipeline_block_and_events() {
    let decoder = BittensorDecoder;
    let ctx = decoder_ctx(100);
    let chain_events = two_test_events();

    decoder.handle_block(&ctx, &chain_events).await.unwrap();
    for ce in &chain_events {
        decoder.handle_event(ce, &ctx).await.unwrap();
    }

    let block: DecodedBlock = ctx
        .get_pipeline_data("decoded_block")
        .expect("decoded_block missing");
    assert_eq!(block.block_number, 100);
    assert_eq!(block.event_count, 2);

    let decoded: Vec<DecodedEvent> = ctx
        .get_pipeline_data("decoded_events")
        .expect("decoded_events missing");
    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[0].pallet, "Test");
    assert_eq!(decoded[0].variant, "A");
    assert_eq!(decoded[1].variant, "B");
}

#[tokio::test]
async fn decoded_events_roundtrip_through_json() {
    let decoder = BittensorDecoder;
    let ctx = decoder_ctx(1);
    for ce in &two_test_events() {
        decoder.handle_event(ce, &ctx).await.unwrap();
    }
    let decoded: Vec<DecodedEvent> = ctx.get_pipeline_data("decoded_events").unwrap();
    for event in &decoded {
        let json_str = serde_json::to_string(event).unwrap();
        let restored: DecodedEvent = serde_json::from_str(&json_str).unwrap();
        assert_eq!(restored.pallet, event.pallet);
        assert_eq!(restored.variant, event.variant);
        assert_eq!(restored.index, event.index);
    }
}

// -- Config: gateway-style wss URL accepted --

#[test]
fn config_accepts_gateway_wss_url() {
    use flamewire_bittensor_indexer::config::IndexerConfig;
    let cfg = IndexerConfig::builder()
        .node_url("wss://gateway-dev.flamewire.io/public/rpc/bittensor/fw_test")
        .start_from_block(0)
        .end_at_block(2)
        .build()
        .unwrap();
    assert_eq!(cfg.start_block, Some(0));
    assert_eq!(cfg.end_block, Some(2));
}
