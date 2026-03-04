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

// Regression tests for metadata change handling (v0.2.3).
// The Indexer caches fetched metadata by spec_version in a HashMap to avoid
// redundant RPC calls. The cache is pub(crate), so not directly testable here.
#[path = "../common/mod.rs"]
mod common;
use common::*;

use flamewire_bittensor_indexer::error::IndexerError;
use flamewire_bittensor_indexer::handler::{Context, EventFilter, Handler};
use flamewire_bittensor_indexer::retry::is_retryable_error;
use flamewire_bittensor_indexer::types::ChainEvent;

use parity_scale_codec::Encode;
use std::sync::Arc;
use subxt::config::substrate::SubstrateConfig;
use subxt::events::Phase;
use subxt::utils::H256;
use subxt::Error as SubxtError;

#[test]
fn metadata_update_failed_contains_source() {
    let err = IndexerError::MetadataUpdateFailed {
        source: Box::new(SubxtError::Other("rpc timeout".into())),
    };
    let msg = format!("{err}");
    assert!(msg.contains("Metadata update failed"), "got: {msg}");
    assert!(msg.contains("rpc timeout"), "got: {msg}");
}

#[test]
fn metadata_update_failed_is_debug_printable() {
    let err = IndexerError::MetadataUpdateFailed {
        source: Box::new(SubxtError::Other("test".into())),
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("MetadataUpdateFailed"), "got: {debug}");
}

#[test]
fn event_decoding_failed_contains_block_number() {
    let err = IndexerError::EventDecodingFailed {
        pallet: "System".into(),
        event: "ExtrinsicSuccess".into(),
        block: 12345,
        source: Box::new(SubxtError::Other("bad bytes".into())),
    };
    let msg = format!("{err}");
    assert!(msg.contains("12345"), "block number missing: {msg}");
    assert!(msg.contains("System"), "pallet missing: {msg}");
    assert!(msg.contains("ExtrinsicSuccess"), "event missing: {msg}");
}

#[test]
fn event_decoding_failed_unknown_pallet_and_event() {
    let err = IndexerError::EventDecodingFailed {
        pallet: "<unknown>".into(),
        event: "<unknown>".into(),
        block: 99,
        source: Box::new(SubxtError::Other("corrupt".into())),
    };
    let msg = format!("{err}");
    assert!(msg.contains("<unknown>.<unknown>"), "got: {msg}");
    assert!(msg.contains("99"), "block missing: {msg}");
}

#[test]
fn corrupted_events_fail_decode_iter() {
    let metadata = test_metadata::<TestEvent>();
    let mut raw = parity_scale_codec::Compact(2u32).encode();
    EventRecord::new(Phase::Initialization, TestEvent::A(1)).encode_to(&mut raw);
    EventRecord::new(Phase::ApplyExtrinsic(0), TestEvent::B(true)).encode_to(&mut raw);
    raw.pop(); // corrupt last byte
    let events = subxt::events::Events::<SubstrateConfig>::decode_from(raw, metadata);
    let err_count = events.iter().filter(|r| r.is_err()).count();
    assert!(err_count > 0, "expected decode error from corrupted bytes");
}

#[test]
fn event_decoding_failed_matches_variant() {
    let err = IndexerError::EventDecodingFailed {
        pallet: "Balances".into(),
        event: "Transfer".into(),
        block: 500,
        source: Box::new(SubxtError::Other("mismatch".into())),
    };
    assert!(matches!(
        err,
        IndexerError::EventDecodingFailed { block: 500, .. }
    ));
}

#[tokio::test]
async fn handler_still_called_for_valid_events() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![
            EventRecord::new(Phase::Initialization, TestEvent::A(42)),
            EventRecord::new(Phase::ApplyExtrinsic(0), TestEvent::B(true)),
        ],
    );
    let handler = Arc::new(MockHandler::new(EventFilter::all()));
    let ctx = Context::<SubstrateConfig>::new(100, H256::zero());
    let chain_events: Vec<ChainEvent<SubstrateConfig>> = evs
        .iter()
        .enumerate()
        .map(|(i, e)| ChainEvent::new(e.unwrap(), i as u32))
        .collect();

    handler.handle_block(&ctx, &chain_events).await.unwrap();
    for ce in &chain_events {
        handler.handle_event(ce, &ctx).await.unwrap();
    }

    let recorded = handler.events.lock().unwrap();
    assert_eq!(recorded.len(), 3, "expected block + 2 events: {recorded:?}");
    assert!(recorded[0].starts_with("block:"), "first should be block");
}

#[test]
fn metadata_error_wraps_subxt_error() {
    use std::error::Error;
    let err = IndexerError::MetadataUpdateFailed {
        source: Box::new(SubxtError::Other("network unreachable".into())),
    };
    let source_msg = format!("{}", err.source().unwrap());
    assert!(source_msg.contains("network unreachable"), "got: {source_msg}");
}

#[tokio::test]
async fn error_handler_receives_block_context() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let mut handler = MockHandler::new(EventFilter::all());
    handler.fail = true;
    let handler = Arc::new(handler);
    let ctx = Context::<SubstrateConfig>::new(42, H256::zero());
    let chain_events: Vec<ChainEvent<SubstrateConfig>> = evs
        .iter()
        .enumerate()
        .map(|(i, e)| ChainEvent::new(e.unwrap(), i as u32))
        .collect();

    for ce in &chain_events {
        let res = handler.handle_event(ce, &ctx).await;
        assert!(res.is_err());
        handler.handle_error(res.as_ref().unwrap_err(), &ctx).await;
    }

    let errors = handler.errors.lock().unwrap();
    assert!(!errors.is_empty(), "error handler should have been called");
    assert!(errors[0].contains("block 42"), "error should contain block: {}", errors[0]);
}

#[test]
fn event_decoding_failed_is_not_retryable() {
    let err = IndexerError::EventDecodingFailed {
        pallet: "System".into(),
        event: "ExtrinsicSuccess".into(),
        block: 1,
        source: Box::new(SubxtError::Other("decode".into())),
    };
    assert!(!is_retryable_error(&err), "decode errors should not be retried");
}

#[test]
fn metadata_update_failed_is_retryable() {
    let err = IndexerError::MetadataUpdateFailed {
        source: Box::new(SubxtError::Other("rpc timeout".into())),
    };
    assert!(is_retryable_error(&err), "metadata failures should be retried");
}
#[test]
fn metadata_update_failed_to_string_includes_context() {
    let err = IndexerError::MetadataUpdateFailed {
        source: Box::new(SubxtError::Other("connection reset".into())),
    };
    let s = err.to_string();
    assert!(s.contains("Metadata update failed"), "got: {s}");
    assert!(s.contains("connection reset"), "source missing: {s}");
}
