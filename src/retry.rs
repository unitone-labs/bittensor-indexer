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

use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::error::IndexerError;
use tracing::warn;

pub struct RetryConfig {
    pub max_retries: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

pub struct CircuitBreaker {
    failures: AtomicUsize,
    threshold: usize,
    cooldown: Duration,
    open_until: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(threshold: usize, cooldown: Duration) -> Self {
        Self {
            failures: AtomicUsize::new(0),
            threshold,
            cooldown,
            open_until: Mutex::new(None),
        }
    }

    pub fn is_open(&self) -> bool {
        if let Some(until) = *self.open_until.lock().unwrap() {
            if Instant::now() < until {
                return true;
            }
        }
        false
    }

    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
        *self.open_until.lock().unwrap() = None;
    }

    pub fn record_failure(&self) {
        let failures = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= self.threshold {
            *self.open_until.lock().unwrap() = Some(Instant::now() + self.cooldown);
            self.failures.store(0, Ordering::Relaxed);
        }
    }
}

fn is_retryable_subxt_error(err: &subxt::Error) -> bool {
    if err.is_rpc_limit_reached() {
        return false;
    }
    if let subxt::Error::Rpc(subxt::error::RpcError::ClientError(_)) = err {
        return false;
    }
    true
}

pub fn is_retryable_error(err: &IndexerError) -> bool {
    match err {
        IndexerError::BlockNotFound { .. } | IndexerError::InvalidConfig { .. } => false,
        IndexerError::Subxt(e)
        | IndexerError::ConnectionFailed { source: e, .. }
        | IndexerError::MetadataUpdateFailed { source: e } => is_retryable_subxt_error(e.as_ref()),
        _ => true,
    }
}

pub async fn retry_with_backoff<F, Fut, T>(
    mut op: F,
    config: &RetryConfig,
    circuit_breaker: &CircuitBreaker,
) -> Result<T, IndexerError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, IndexerError>>,
{
    let mut delay = config.initial_delay;
    for attempt in 0..config.max_retries {
        if circuit_breaker.is_open() {
            return Err(IndexerError::Subxt(Box::new(subxt::Error::Other(
                "circuit open".into(),
            ))));
        }
        match op().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if !is_retryable_error(&e) || attempt + 1 == config.max_retries {
                    return Err(e);
                }
                warn!(target: "indexer", "retrying in {:?} after error", delay);
                sleep(delay).await;
                let next = (delay.as_millis() as f32 * config.backoff_multiplier) as u64;
                delay = Duration::from_millis(next).min(config.max_delay);
            }
        }
    }
    unreachable!()
}
