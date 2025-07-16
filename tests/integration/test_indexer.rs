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
use flamewire_bittensor_indexer::handler::{Context, Handler};
use flamewire_bittensor_indexer::CheckpointStore;
use flamewire_bittensor_indexer::IndexerError;
use flamewire_bittensor_indexer::{ChainEvent, EventFilter};
use std::sync::Arc;
use subxt::config::substrate::SubstrateConfig;
use subxt::events::Phase;
use subxt::utils::H256;

async fn process_blocks(
    handler: Arc<MockHandler>,
    store: &MockCheckpointStore,
) -> Result<(), IndexerError> {
    let metadata = test_metadata::<TestEvent>();
    let blocks = vec![
        (
            1u64,
            events(
                metadata.clone(),
                vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
            ),
        ),
        (
            2u64,
            events(
                metadata,
                vec![EventRecord::new(Phase::Initialization, TestEvent::B(true))],
            ),
        ),
    ];

    for (num, evs) in blocks {
        let ctx = Context::<SubstrateConfig>::new(num, H256::zero());
        let mut ces = Vec::new();
        for (i, e) in evs.iter().enumerate() {
            ces.push(ChainEvent::new(e?, i as u32));
        }
        handler.handle_block(&ctx, &ces).await?;
        for ce in &ces {
            if let Err(e) = handler.handle_event(ce, &ctx).await {
                handler.handle_error(&e, &ctx).await;
            }
        }
        store.store_checkpoint(num).await?;
    }
    Ok(())
}

#[tokio::test]
async fn full_workflow() {
    let handler = Arc::new(MockHandler::new(EventFilter::all()));
    let store = MockCheckpointStore::new();
    process_blocks(handler.clone(), &store).await.unwrap();
    assert_eq!(store.checkpoints.lock().unwrap().len(), 2);
    assert!(handler
        .events
        .lock()
        .unwrap()
        .iter()
        .any(|e| e.contains("Test.A")));
    assert!(handler
        .events
        .lock()
        .unwrap()
        .iter()
        .any(|e| e.contains("Test.B")));
}
