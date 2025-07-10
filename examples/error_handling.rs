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
    async_trait, ChainEvent, Context, Handler, HandlerGroup, IndexerBuilder, IndexerError,
    SubstrateConfig, WebSocketUrl,
};
use flamewire_bittensor_indexer::{retry_with_backoff, CircuitBreaker, RetryConfig};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct FailingHandler {
    id: &'static str,
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl Handler<SubstrateConfig> for FailingHandler {
    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context,
    ) -> Result<(), IndexerError> {
        let attempt = self.count.fetch_add(1, Ordering::SeqCst);
        println!(
            "Handler {} attempt {} on block {}",
            self.id, attempt, ctx.block_number
        );
        Err(IndexerError::HandlerFailed {
            handler: self.id.into(),
            block: ctx.block_number,
            source: Box::new(std::io::Error::other("fail")),
        })
    }

    async fn handle_error(&self, error: &IndexerError, _ctx: &Context) {
        println!("{error}");
    }
}

struct DatabaseSaver {
    circuit_breaker: Arc<CircuitBreaker>,
    failure_count: Arc<AtomicUsize>,
}

#[async_trait]
impl Handler<SubstrateConfig> for DatabaseSaver {
    async fn handle_event(
        &self,
        _event: &ChainEvent<SubstrateConfig>,
        ctx: &Context,
    ) -> Result<(), IndexerError> {
        if self.circuit_breaker.is_open() {
            println!(
                "\u{1F6D1} Database circuit breaker OPEN - skipping save for block {}",
                ctx.block_number
            );
            return Ok(());
        }

        let attempt = self.failure_count.fetch_add(1, Ordering::SeqCst);
        if attempt % 4 == 0 || attempt % 4 == 1 {
            self.circuit_breaker.record_failure();
            return Err(IndexerError::HandlerFailed {
                handler: "DatabaseSaver".into(),
                block: ctx.block_number,
                source: Box::new(std::io::Error::other("Database connection timeout")),
            });
        } else {
            self.circuit_breaker.record_success();
            println!(
                "\u{2705} Transfer saved to database (block {})",
                ctx.block_number
            );
        }

        Ok(())
    }

    async fn handle_error(&self, error: &IndexerError, _ctx: &Context) {
        println!("{error}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let counter = Arc::new(AtomicUsize::new(0));
    let db_failures = Arc::new(AtomicUsize::new(0));
    let db_breaker = Arc::new(CircuitBreaker::new(2, Duration::from_secs(10)));

    // Tolerant mode: errors are logged but processing continues
    let tolerant = HandlerGroup::new()
        .add(FailingHandler {
            id: "A",
            count: counter.clone(),
        })
        .add(FailingHandler {
            id: "B",
            count: counter.clone(),
        })
        .add(DatabaseSaver {
            circuit_breaker: db_breaker.clone(),
            failure_count: db_failures.clone(),
        });

    // Strict mode: first error aborts the remaining handlers
    let strict = HandlerGroup::new()
        .strict()
        .add(FailingHandler {
            id: "A",
            count: counter.clone(),
        })
        .add(FailingHandler {
            id: "B",
            count: counter.clone(),
        });

    // Example retry logic with circuit breaker
    let retry_cfg = RetryConfig {
        max_retries: 3,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(2, Duration::from_secs(30));
    let op = || async {
        Err::<(), _>(IndexerError::ConnectionFailed {
            url: "wss://node".into(),
            source: Box::new(subxt::Error::Other("down".into())),
        })
    };
    let _ = retry_with_backoff(op, &retry_cfg, &breaker).await.err();

    let mut indexer = IndexerBuilder::<SubstrateConfig>::new()
        .connect(WebSocketUrl::parse(
            "wss://archive.chain.opentensor.ai:443",
        )?)
        .start_from_block(1017)
        .end_at_block(1033)
        .add_handler_group(tolerant)
        .add_handler_group(strict)
        .build()
        .await?;

    indexer.run().await?;
    Ok(())
}
