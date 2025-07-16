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
use async_trait::async_trait;
use common::*;
use flamewire_bittensor_indexer::handler::{Context, EventFilter, Handler};
use flamewire_bittensor_indexer::handler_group::HandlerGroup;
use flamewire_bittensor_indexer::{ChainEvent, IndexerError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use subxt::config::substrate::SubstrateConfig;
use subxt::events::Phase;
use subxt::utils::H256;
use tokio::time::sleep;

struct TestHandler {
    id: &'static str,
    log: Arc<Mutex<Vec<String>>>,
    errors: Arc<Mutex<Vec<String>>>,
    delay: Duration,
    fail_event: bool,
    set_data: Option<(&'static str, u32)>,
    get_data: Option<&'static str>,
}

impl TestHandler {
    fn new(
        id: &'static str,
        log: Arc<Mutex<Vec<String>>>,
        errors: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self {
            id,
            log,
            errors,
            delay: Duration::from_millis(0),
            fail_event: false,
            set_data: None,
            get_data: None,
        }
    }
    fn with_delay(mut self, d: Duration) -> Self {
        self.delay = d;
        self
    }
    fn fail_event(mut self) -> Self {
        self.fail_event = true;
        self
    }
    fn set_data(mut self, key: &'static str, val: u32) -> Self {
        self.set_data = Some((key, val));
        self
    }
    fn get_data(mut self, key: &'static str) -> Self {
        self.get_data = Some(key);
        self
    }
}

#[async_trait]
impl Handler<SubstrateConfig> for TestHandler {
    fn event_filter(&self) -> EventFilter {
        EventFilter::all()
    }

    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        if self.delay != Duration::ZERO {
            sleep(self.delay).await;
        }
        self.log.lock().unwrap().push(format!("event-{}", self.id));
        if let Some((k, v)) = self.set_data {
            ctx.set_pipeline_data(k, v);
        }
        if let Some(k) = self.get_data {
            let data = ctx.get_pipeline_data::<u32>(k);
            self.log
                .lock()
                .unwrap()
                .push(format!("{}-data:{:?}", self.id, data));
        }
        if self.fail_event {
            return Err(IndexerError::HandlerFailed {
                handler: self.id.into(),
                block: ctx.block_number,
                source: Box::new(std::io::Error::other("fail")),
            });
        }
        Ok(())
    }

    async fn handle_block(
        &self,
        _ctx: &Context<SubstrateConfig>,
        _events: &[ChainEvent<SubstrateConfig>],
    ) -> Result<(), IndexerError> {
        if self.delay != Duration::ZERO {
            sleep(self.delay).await;
        }
        self.log.lock().unwrap().push(format!("block-{}", self.id));
        Ok(())
    }

    async fn handle_error(&self, error: &IndexerError, _ctx: &Context<SubstrateConfig>) {
        self.errors.lock().unwrap().push(format!("{error}"));
    }
}

#[tokio::test]
async fn test_sequential_execution_order() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let log = Arc::new(Mutex::new(Vec::new()));
    let errs = Arc::new(Mutex::new(Vec::new()));
    let group = HandlerGroup::new()
        .add(TestHandler::new("1", log.clone(), errs.clone()))
        .add(TestHandler::new("2", log.clone(), errs.clone()))
        .add(TestHandler::new("3", log.clone(), errs.clone()));
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());
    let chain_events: Vec<ChainEvent<SubstrateConfig>> = evs
        .iter()
        .enumerate()
        .map(|(i, e)| ChainEvent::new(e.unwrap(), i as u32))
        .collect();
    group.handle_block(&ctx, &chain_events).await.unwrap();
    for ce in &chain_events {
        group.handle_event(ce, &ctx).await.unwrap();
    }
    let calls = log.lock().unwrap().clone();
    assert_eq!(
        calls,
        vec!["block-1", "block-2", "block-3", "event-1", "event-2", "event-3"]
    );
    assert!(errs.lock().unwrap().is_empty());
}

#[tokio::test]
async fn test_tolerant_mode_continues() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let log = Arc::new(Mutex::new(Vec::new()));
    let errs = Arc::new(Mutex::new(Vec::new()));
    let group = HandlerGroup::new()
        .add(TestHandler::new("1", log.clone(), errs.clone()).fail_event())
        .add(TestHandler::new("2", log.clone(), errs.clone()));
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());
    for (index, ev) in evs.iter().enumerate() {
        let ev = ev.unwrap();
        group
            .handle_event(&ChainEvent::new(ev, index as u32), &ctx)
            .await
            .unwrap();
    }
    let calls = log.lock().unwrap().clone();
    assert_eq!(calls, vec!["event-1", "event-2"]);
    assert_eq!(errs.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_strict_mode_stops_on_error() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let log = Arc::new(Mutex::new(Vec::new()));
    let errs = Arc::new(Mutex::new(Vec::new()));
    let group = HandlerGroup::new()
        .strict()
        .add(TestHandler::new("1", log.clone(), errs.clone()).fail_event())
        .add(TestHandler::new("2", log.clone(), errs.clone()));
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());
    let ev = evs.iter().next().unwrap().unwrap();
    let res = group.handle_event(&ChainEvent::new(ev, 0), &ctx).await;
    assert!(res.is_err());
    let calls = log.lock().unwrap().clone();
    assert_eq!(calls, vec!["event-1"]);
    assert_eq!(errs.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_parallel_performance() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata.clone(),
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let ce = {
        let ev = evs.iter().next().unwrap().unwrap();
        ChainEvent::new(ev, 0)
    };
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());
    let log_seq = Arc::new(Mutex::new(Vec::new()));
    let err_seq = Arc::new(Mutex::new(Vec::new()));
    let seq_group = HandlerGroup::new()
        .add(
            TestHandler::new("s1", log_seq.clone(), err_seq.clone())
                .with_delay(Duration::from_millis(50)),
        )
        .add(
            TestHandler::new("s2", log_seq.clone(), err_seq.clone())
                .with_delay(Duration::from_millis(50)),
        );
    let start = Instant::now();
    seq_group.handle_event(&ce, &ctx).await.unwrap();
    let seq_dur = start.elapsed();

    let log_par = Arc::new(Mutex::new(Vec::new()));
    let err_par = Arc::new(Mutex::new(Vec::new()));
    let par_group = HandlerGroup::parallel()
        .add(
            TestHandler::new("p1", log_par.clone(), err_par.clone())
                .with_delay(Duration::from_millis(50)),
        )
        .add(
            TestHandler::new("p2", log_par.clone(), err_par.clone())
                .with_delay(Duration::from_millis(50)),
        );
    let start = Instant::now();
    par_group.handle_event(&ce, &ctx).await.unwrap();
    let par_dur = start.elapsed();

    assert!(par_dur < seq_dur);
}

#[tokio::test]
async fn test_parallel_error_collection() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let log = Arc::new(Mutex::new(Vec::new()));
    let errs1 = Arc::new(Mutex::new(Vec::new()));
    let errs2 = Arc::new(Mutex::new(Vec::new()));
    let group = HandlerGroup::parallel()
        .add(TestHandler::new("1", log.clone(), errs1.clone()).fail_event())
        .add(TestHandler::new("2", log.clone(), errs2.clone()).fail_event());
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());
    let ev = evs.iter().next().unwrap().unwrap();
    group
        .handle_event(&ChainEvent::new(ev, 0), &ctx)
        .await
        .unwrap();
    assert_eq!(errs1.lock().unwrap().len(), 1);
    assert_eq!(errs2.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_pipeline_data_flow() {
    let metadata = test_metadata::<TestEvent>();
    let evs = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let log = Arc::new(Mutex::new(Vec::new()));
    let errs = Arc::new(Mutex::new(Vec::new()));
    let h1 = TestHandler::new("1", log.clone(), errs.clone()).set_data("num", 42);
    let h2 = TestHandler::new("2", log.clone(), errs.clone()).get_data("num");
    let group = HandlerGroup::new().add(h1).pipe_to(h2);
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());
    let ev = evs.iter().next().unwrap().unwrap();
    group
        .handle_event(&ChainEvent::new(ev, 0), &ctx)
        .await
        .unwrap();
    let calls = log.lock().unwrap();
    assert!(calls.iter().any(|c| c == "event-1"));
    assert!(calls.iter().any(|c| c.contains("2-data:Some(42)")));
    assert!(ctx.get_pipeline_data::<u32>("num").is_none());
    assert!(ctx.get_pipeline_data::<String>("num").is_none());
}

#[tokio::test]
async fn test_conditional_handler() {
    let metadata = test_metadata::<TestEvent>();
    let ev_a = events(
        metadata.clone(),
        vec![EventRecord::new(Phase::Initialization, TestEvent::A(1))],
    );
    let ev_b = events(
        metadata,
        vec![EventRecord::new(Phase::Initialization, TestEvent::B(true))],
    );
    let log = Arc::new(Mutex::new(Vec::new()));
    let errs = Arc::new(Mutex::new(Vec::new()));
    let cond = TestHandler::new("c", log.clone(), errs.clone());
    let uncond = TestHandler::new("u", log.clone(), errs.clone());
    let group = HandlerGroup::new()
        .add_conditional(cond, |e: &ChainEvent<SubstrateConfig>| {
            e.variant_name() == "A"
        })
        .add(uncond);
    let ctx = Context::<SubstrateConfig>::new(1, H256::zero());
    let ev = ev_a.iter().next().unwrap().unwrap();
    group
        .handle_event(&ChainEvent::new(ev, 0), &ctx)
        .await
        .unwrap();
    assert_eq!(
        log.lock()
            .unwrap()
            .iter()
            .filter(|s| s.contains("c"))
            .count(),
        1
    );
    log.lock().unwrap().clear();
    let ev = ev_b.iter().next().unwrap().unwrap();
    group
        .handle_event(&ChainEvent::new(ev, 0), &ctx)
        .await
        .unwrap();
    assert_eq!(
        log.lock()
            .unwrap()
            .iter()
            .filter(|s| s.contains("c"))
            .count(),
        0
    );
}
