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

#![allow(clippy::duplicate_mod)]
#[path = "../common/mod.rs"]
mod common;
use common::*;
use flamewire_bittensor_indexer::retry::{retry_with_backoff, CircuitBreaker, RetryConfig};
use flamewire_bittensor_indexer::{
    ChainEvent, CheckpointStore, Context, EventFilter, Handler, IndexerConfig, IndexerError,
};
use parity_scale_codec::Encode;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use subxt::config::substrate::SubstrateConfig;
use subxt::events::Phase;
use subxt::utils::H256;
use subxt::Error as SubxtError;

#[tokio::test]
async fn retry_recovers_from_connection_drop() {
    let cfg = RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(2),
        backoff_multiplier: 1.0,
    };
    let cb = CircuitBreaker::new(3, Duration::from_secs(60));
    let attempts = Arc::new(AtomicUsize::new(0));
    let cnt = attempts.clone();

    let res = retry_with_backoff(
        || {
            let cnt = cnt.clone();
            async move {
                let n = cnt.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err(IndexerError::ConnectionFailed {
                        url: "wss://node".into(),
                        source: Box::new(SubxtError::Other("drop".into())),
                    })
                } else {
                    Ok(n)
                }
            }
        },
        &cfg,
        &cb,
    )
    .await
    .unwrap();

    assert!(res >= 2);
    assert!(attempts.load(Ordering::SeqCst) >= 3);
}

#[tokio::test]
async fn circuit_breaker_opens_after_failures() {
    let cfg = RetryConfig {
        max_retries: 1,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(1),
        backoff_multiplier: 1.0,
    };
    let cb = CircuitBreaker::new(2, Duration::from_secs(60));

    let failing_op = || async {
        Err::<(), _>(IndexerError::ConnectionFailed {
            url: "ws://node".into(),
            source: Box::new(SubxtError::Other("timeout".into())),
        })
    };

    assert!(retry_with_backoff(failing_op, &cfg, &cb).await.is_err());
    cb.record_failure();
    assert!(retry_with_backoff(failing_op, &cfg, &cb).await.is_err());
    cb.record_failure();

    let res = retry_with_backoff(failing_op, &cfg, &cb).await;
    match res {
        Err(IndexerError::Subxt(ref err)) => {
            if let SubxtError::Other(msg) = err.as_ref() {
                assert_eq!(msg, "circuit open");
            } else {
                panic!("wrong error: {res:?}");
            }
        }
        _ => panic!("wrong error: {res:?}"),
    }
}

#[tokio::test]
async fn database_checkpoint_store_failure() {
    let mut store = MockCheckpointStore::new();
    store.fail_store = true;
    let res = store.store_checkpoint(10).await;
    assert!(matches!(res, Err(IndexerError::CheckpointError { .. })));
}

#[tokio::test]
async fn database_checkpoint_load_failure() {
    let mut store = MockCheckpointStore::new();
    store.fail_load = true;
    let res = store.load_checkpoint().await;
    assert!(matches!(res, Err(IndexerError::CheckpointError { .. })));
}

#[tokio::test]
async fn corrupted_event_bytes_fail_to_decode() {
    let metadata = test_metadata::<TestEvent>();
    let mut raw = parity_scale_codec::Compact(1u32).encode();
    EventRecord::new(Phase::Initialization, TestEvent::A(1)).encode_to(&mut raw);
    raw.pop(); // corrupt
    let events = subxt::events::Events::<SubstrateConfig>::decode_from(raw, metadata);
    let res = events.iter().next().unwrap();
    assert!(res.is_err());
}

#[tokio::test]
async fn retry_gives_up_after_max_retries() {
    let cfg = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(2),
        backoff_multiplier: 1.0,
    };
    let cb = CircuitBreaker::new(3, Duration::from_secs(60));
    let attempts = Arc::new(AtomicUsize::new(0));
    let cnt = attempts.clone();

    let res = retry_with_backoff::<_, _, ()>(
        || {
            let cnt = cnt.clone();
            async move {
                cnt.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(5)).await;
                Err(IndexerError::HandlerFailed {
                    handler: "h".into(),
                    block: 0,
                    source: Box::new(std::io::Error::other("slow")),
                })
            }
        },
        &cfg,
        &cb,
    )
    .await;
    assert!(res.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn invalid_node_url_configuration_error() {
    let res = IndexerConfig::builder().node_url("ftp://node").build();
    assert!(res.is_err());
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn wrong_database_credentials() {
    use flamewire_bittensor_indexer::storage::postgres::PostgreSQLStore;
    let err = PostgreSQLStore::new("postgres://invalid:invalid@localhost:1/db").await;
    assert!(matches!(err, Err(IndexerError::CheckpointError { .. })));
}

#[tokio::test]
async fn conflicting_handlers_continue_processing() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let handler_ok = Arc::new(MockHandler::new(EventFilter::all()));
    let mut handler_fail = MockHandler::new(EventFilter::all());
    handler_fail.fail = true;
    let handler_fail = Arc::new(handler_fail);
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());

    let chain_events: Vec<ChainEvent<SubstrateConfig>> = evs
        .iter()
        .enumerate()
        .map(|(i, e)| ChainEvent::new(e.unwrap(), i as u32))
        .collect();
    handler_ok.handle_block(&ctx, &chain_events).await.unwrap();
    handler_fail
        .handle_block(&ctx, &chain_events)
        .await
        .unwrap();

    for ce in &chain_events {
        handler_ok.handle_event(ce, &ctx).await.unwrap();
        let res = handler_fail.handle_event(ce, &ctx).await;
        assert!(res.is_err());
        handler_fail
            .handle_error(res.as_ref().err().unwrap(), &ctx)
            .await;
    }

    assert!(!handler_fail.errors.lock().unwrap().is_empty());
    assert!(!handler_ok.events.lock().unwrap().is_empty());
}
