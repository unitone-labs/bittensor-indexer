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
use crate::types::decoded::{DecodedBlock, DecodedEvent};
use crate::types::ChainEvent;

use async_trait::async_trait;
use scale_value::{Value, ValueDef};
use serde_json::{to_value, Value as JsonValue};
use subxt::Config;
use tracing::debug;

/// Decodes blocks and events into structured types.
pub struct BittensorDecoder;

#[async_trait]
impl<C> Handler<C> for BittensorDecoder
where
    C: Config + Send + Sync + 'static,
{
    fn event_filter(&self) -> EventFilter {
        EventFilter::all()
    }

    async fn handle_block(
        &self,
        ctx: &Context<C>,
        events: &[ChainEvent<C>],
    ) -> Result<(), IndexerError> {
        let block = DecodedBlock {
            block_number: ctx.block_number,
            block_hash: format!("{:?}", ctx.block_hash),
            extrinsic_count: 0,
            event_count: events.len() as u32,
        };

        debug!(
            block_number = block.block_number,
            block_hash = %block.block_hash,
            event_count = block.event_count,
            "Decoded block"
        );

        ctx.set_pipeline_data("decoded_block", block);
        Ok(())
    }

    async fn handle_event(
        &self,
        event: &ChainEvent<C>,
        ctx: &Context<C>,
    ) -> Result<(), IndexerError> {
        let fields = match event.field_values() {
            Ok(composite) => {
                let value: Value<u32> = Value {
                    value: ValueDef::Composite(composite),
                    context: 0,
                };
                to_value(&value).unwrap_or(JsonValue::Null)
            }
            Err(_) => JsonValue::Null,
        };

        let decoded = DecodedEvent {
            index: event.index,
            pallet: event.pallet_name().to_string(),
            variant: event.variant_name().to_string(),
            fields,
        };

        debug!(
            index = decoded.index,
            pallet = %decoded.pallet,
            variant = %decoded.variant,
            "Decoded event"
        );

        // Accumulate decoded events in the pipeline
        let mut events: Vec<DecodedEvent> = ctx
            .peek_pipeline_data::<Vec<DecodedEvent>>("decoded_events")
            .unwrap_or_default();
        events.push(decoded);
        ctx.set_pipeline_data("decoded_events", events);

        Ok(())
    }
}
