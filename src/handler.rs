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

use crate::error::IndexerError;
use crate::types::ChainEvent;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Mutex;
use subxt::config::HashFor;
use subxt::events::Events;
use subxt::Config;

pub struct Context<C: Config> {
    pub block_number: u64,
    pub block_hash: HashFor<C>,
    pipeline: Mutex<HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl<C: Config> Context<C> {
    pub fn new(block_number: u64, block_hash: HashFor<C>) -> Self {
        Self {
            block_number,
            block_hash,
            pipeline: Mutex::new(HashMap::new()),
        }
    }

    /// Store data for use by subsequent handlers in a pipeline
    pub fn set_pipeline_data<T: Send + Sync + 'static>(&self, key: &str, data: T) {
        let mut map = self.pipeline.lock().unwrap();
        map.insert(key.to_string(), Box::new(data));
    }

    /// Retrieve data stored by previous handlers
    pub fn get_pipeline_data<T: 'static>(&self, key: &str) -> Option<T> {
        let mut map = self.pipeline.lock().unwrap();
        map.remove(key)?.downcast::<T>().ok().map(|b| *b)
    }

    /// Peek at data without consuming it, useful for parallel handler groups
    pub fn peek_pipeline_data<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        let map = self.pipeline.lock().unwrap();
        map.get(key)?.downcast_ref::<T>().cloned()
    }
}

pub struct EventFilter {
    pub pallet: Option<&'static str>,
    pub event: Option<&'static str>,
}

impl EventFilter {
    pub const fn all() -> Self {
        Self {
            pallet: None,
            event: None,
        }
    }

    pub const fn pallet(pallet: &'static str) -> Self {
        Self {
            pallet: Some(pallet),
            event: None,
        }
    }

    pub const fn event(pallet: &'static str, event: &'static str) -> Self {
        Self {
            pallet: Some(pallet),
            event: Some(event),
        }
    }

    pub fn matches(&self, pallet: &str, event: &str) -> bool {
        match (self.pallet, self.event) {
            (Some(p), Some(e)) => p == pallet && e == event,
            (Some(p), None) => p == pallet,
            (None, None) => true,
            _ => false,
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
pub trait Handler<C: Config>: Send + Sync {
    fn event_filter(&self) -> EventFilter {
        EventFilter::all()
    }

    async fn handle_event(
        &self,
        event: &ChainEvent<C>,
        ctx: &Context<C>,
    ) -> Result<(), IndexerError> {
        Ok(())
    }

    async fn handle_block(&self, ctx: &Context<C>, events: &Events<C>) -> Result<(), IndexerError> {
        Ok(())
    }

    async fn handle_error(&self, error: &IndexerError, ctx: &Context<C>) {}
}
