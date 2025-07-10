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
use flamewire_bittensor_indexer::handler::{Context, EventFilter, Handler};
use flamewire_bittensor_indexer::types::ChainEvent;
use subxt::events::Phase;

#[tokio::test]
async fn event_filter_matches() {
    assert!(EventFilter::all().matches("A", "B"));
    assert!(EventFilter::pallet("A").matches("A", "C"));
    assert!(!EventFilter::pallet("A").matches("B", "C"));
    assert!(EventFilter::event("A", "B").matches("A", "B"));
    assert!(!EventFilter::event("A", "B").matches("A", "C"));
}

#[tokio::test]
async fn handler_flow() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let handler = MockHandler::new(EventFilter::all());
    let ctx = Context::new(1);

    handler.handle_block(&ctx, &evs).await.unwrap();
    for ev in evs.iter() {
        let ev = ev.unwrap();
        let ce = ChainEvent::new(ev);
        handler.handle_event(&ce, &ctx).await.unwrap();
    }

    let calls = handler.events.lock().unwrap();
    assert!(calls.contains(&"block:1".to_string()));
    assert!(calls.iter().any(|c| c.contains("Test.A")));
}

#[tokio::test]
async fn handler_error_path() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::B(true))],
    );
    let mut handler = MockHandler::new(EventFilter::all());
    handler.fail = true;
    let ctx = Context::new(2);

    for ev in evs.iter() {
        let ev = ev.unwrap();
        let ce = ChainEvent::new(ev);
        let res = handler.handle_event(&ce, &ctx).await;
        assert!(res.is_err());
        handler
            .handle_error(res.as_ref().err().unwrap(), &ctx)
            .await;
    }

    assert!(!handler.errors.lock().unwrap().is_empty());
}
