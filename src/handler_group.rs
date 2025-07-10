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
use crate::handler::{Context, EventFilter, Handler};
use crate::types::ChainEvent;
use async_trait::async_trait;
use futures::future::join_all;
use std::marker::PhantomData;
use subxt::events::Events;
use subxt::Config;

/// A group of handlers that can be added as a single unit.
pub struct HandlerGroup<C: Config> {
    handlers: Vec<Box<dyn Handler<C>>>,
    strict: bool,
    parallel: bool,
}

impl<C: Config> Default for HandlerGroup<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Config> HandlerGroup<C> {
    /// Create an empty handler group.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            strict: false,
            parallel: false,
        }
    }

    /// Create a handler group that runs handlers in parallel
    pub fn parallel() -> Self {
        Self {
            handlers: Vec::new(),
            strict: false,
            parallel: true,
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// Add a handler to the group.
    pub fn add(mut self, handler: impl Handler<C> + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// Enable strict mode which aborts execution on the first handler error
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Add a handler that will only run when the predicate returns true.
    pub fn add_conditional<F>(mut self, handler: impl Handler<C> + 'static, pred: F) -> Self
    where
        F: Fn(&ChainEvent<C>) -> bool + Send + Sync + 'static,
    {
        self.handlers.push(Box::new(ConditionalHandler {
            handler,
            pred,
            _marker: PhantomData,
        }));
        self
    }

    /// Shortcut for building simple pipelines.
    pub fn pipe_to(self, handler: impl Handler<C> + 'static) -> Self {
        self.add(handler)
    }
}

#[async_trait]
impl<C> Handler<C> for HandlerGroup<C>
where
    C: Config + Send + Sync + 'static,
{
    fn event_filter(&self) -> EventFilter {
        EventFilter::all()
    }

    async fn handle_event(&self, event: &ChainEvent<C>, ctx: &Context) -> Result<(), IndexerError> {
        if self.parallel {
            let futures: Vec<_> = self
                .handlers
                .iter()
                .enumerate()
                .filter(|(_, h)| {
                    h.event_filter()
                        .matches(event.pallet_name(), event.variant_name())
                })
                .map(|(i, h)| async move { (i, h.handle_event(event, ctx).await) })
                .collect();
            let results = join_all(futures).await;
            for (i, res) in results {
                if let Err(e) = res {
                    let h = &self.handlers[i];
                    h.handle_error(&e, ctx).await;
                    if self.strict {
                        return Err(e);
                    }
                }
            }
        } else {
            for h in &self.handlers {
                if h.event_filter()
                    .matches(event.pallet_name(), event.variant_name())
                {
                    if let Err(e) = h.handle_event(event, ctx).await {
                        h.handle_error(&e, ctx).await;
                        if self.strict {
                            return Err(e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_block(&self, ctx: &Context, events: &Events<C>) -> Result<(), IndexerError> {
        if self.parallel {
            let futures: Vec<_> = self
                .handlers
                .iter()
                .enumerate()
                .map(|(i, h)| async move { (i, h.handle_block(ctx, events).await) })
                .collect();
            let results = join_all(futures).await;
            for (i, res) in results {
                if let Err(e) = res {
                    let h = &self.handlers[i];
                    h.handle_error(&e, ctx).await;
                    if self.strict {
                        return Err(e);
                    }
                }
            }
        } else {
            for h in &self.handlers {
                if let Err(e) = h.handle_block(ctx, events).await {
                    h.handle_error(&e, ctx).await;
                    if self.strict {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_error(&self, error: &IndexerError, ctx: &Context) {
        for h in &self.handlers {
            h.handle_error(error, ctx).await;
        }
    }
}

struct ConditionalHandler<C: Config, H: Handler<C>, F> {
    handler: H,
    pred: F,
    _marker: PhantomData<C>,
}

#[async_trait]
impl<C, H, F> Handler<C> for ConditionalHandler<C, H, F>
where
    C: Config + Send + Sync + 'static,
    H: Handler<C> + 'static,
    F: Fn(&ChainEvent<C>) -> bool + Send + Sync + 'static,
{
    fn event_filter(&self) -> EventFilter {
        self.handler.event_filter()
    }

    async fn handle_event(&self, event: &ChainEvent<C>, ctx: &Context) -> Result<(), IndexerError> {
        if (self.pred)(event) {
            self.handler.handle_event(event, ctx).await
        } else {
            Ok(())
        }
    }

    async fn handle_block(&self, ctx: &Context, events: &Events<C>) -> Result<(), IndexerError> {
        self.handler.handle_block(ctx, events).await
    }

    async fn handle_error(&self, error: &IndexerError, ctx: &Context) {
        self.handler.handle_error(error, ctx).await;
    }
}
