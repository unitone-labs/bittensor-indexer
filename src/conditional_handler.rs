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
use subxt::Config;

use std::marker::PhantomData;

pub(crate) struct ConditionalHandler<C: Config, H: Handler<C>, F> {
    pub(crate) handler: H,
    pub(crate) pred: F,
    pub(crate) _marker: PhantomData<C>,
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

    async fn handle_event(
        &self,
        event: &ChainEvent<C>,
        ctx: &Context<C>,
    ) -> Result<(), IndexerError> {
        if (self.pred)(event) {
            self.handler.handle_event(event, ctx).await
        } else {
            Ok(())
        }
    }

    async fn handle_block(
        &self,
        ctx: &Context<C>,
        events: &[ChainEvent<C>],
    ) -> Result<(), IndexerError> {
        self.handler.handle_block(ctx, events).await
    }

    async fn handle_error(&self, error: &IndexerError, ctx: &Context<C>) {
        self.handler.handle_error(error, ctx).await;
    }
}
