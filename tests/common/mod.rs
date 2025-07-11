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

#![allow(dead_code)]
#![allow(clippy::duplicate_mod)]
use async_trait::async_trait;
use flamewire_bittensor_indexer::handler::{Context, EventFilter, Handler};
use flamewire_bittensor_indexer::storage::CheckpointStore;
use flamewire_bittensor_indexer::types::ChainEvent;
use flamewire_bittensor_indexer::IndexerError;
use frame_metadata::{
    v15::{
        CustomMetadata, ExtrinsicMetadata, OuterEnums, PalletEventMetadata, PalletMetadata,
        RuntimeMetadataV15,
    },
    RuntimeMetadataPrefixed,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::{meta_type, TypeInfo};
use std::sync::{Arc, Mutex};
use subxt::config::substrate::SubstrateConfig;
use subxt::events::{Events, Phase};
use subxt::metadata::Metadata;

// ----------------------- MockCheckpointStore ----------------------------
pub struct MockCheckpointStore {
    pub checkpoints: Arc<Mutex<Vec<u64>>>,
    pub fail_load: bool,
    pub fail_store: bool,
}

impl MockCheckpointStore {
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(Mutex::new(Vec::new())),
            fail_load: false,
            fail_store: false,
        }
    }
}

#[async_trait]
impl CheckpointStore for MockCheckpointStore {
    async fn load_checkpoint(&self) -> Result<Option<u64>, IndexerError> {
        if self.fail_load {
            Err(IndexerError::CheckpointError {
                operation: "load_checkpoint".into(),
                backend: "mock".into(),
                source: Box::new(std::io::Error::other("fail")),
            })
        } else {
            Ok(self.checkpoints.lock().unwrap().last().copied())
        }
    }

    async fn store_checkpoint(&self, block: u64) -> Result<(), IndexerError> {
        if self.fail_store {
            Err(IndexerError::CheckpointError {
                operation: "store_checkpoint".into(),
                backend: "mock".into(),
                source: Box::new(std::io::Error::other("fail")),
            })
        } else {
            self.checkpoints.lock().unwrap().push(block);
            Ok(())
        }
    }
}

// ----------------------- MockHandler -----------------------------------
pub struct MockHandler {
    pub pallet: Option<&'static str>,
    pub event: Option<&'static str>,
    pub events: Arc<Mutex<Vec<String>>>,
    pub errors: Arc<Mutex<Vec<String>>>,
    pub fail: bool,
}

impl MockHandler {
    pub fn new(filter: EventFilter) -> Self {
        Self {
            pallet: filter.pallet,
            event: filter.event,
            events: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
            fail: false,
        }
    }
}
#[async_trait]
impl Handler<SubstrateConfig> for MockHandler {
    fn event_filter(&self) -> EventFilter {
        match (self.pallet, self.event) {
            (Some(p), Some(e)) => EventFilter::event(p, e),
            (Some(p), None) => EventFilter::pallet(p),
            _ => EventFilter::all(),
        }
    }

    async fn handle_event(
        &self,
        event: &ChainEvent<SubstrateConfig>,
        _ctx: &Context<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        self.events.lock().unwrap().push(format!(
            "{}.{}",
            event.pallet_name(),
            event.variant_name()
        ));
        if self.fail {
            Err(IndexerError::HandlerFailed {
                handler: "mock".into(),
                block: _ctx.block_number,
                source: Box::new(std::io::Error::other("fail")),
            })
        } else {
            Ok(())
        }
    }

    async fn handle_block(
        &self,
        ctx: &Context<SubstrateConfig>,
        _events: &Events<SubstrateConfig>,
    ) -> Result<(), IndexerError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("block:{}", ctx.block_number));
        Ok(())
    }

    async fn handle_error(&self, error: &IndexerError, _ctx: &Context<SubstrateConfig>) {
        self.errors.lock().unwrap().push(format!("{error}"));
    }
}

// ----------------------- Test Event Builders ---------------------------
#[derive(Encode, Decode, Clone, Debug, PartialEq, TypeInfo)]
pub enum TestEvent {
    A(u8),
    B(bool),
}

#[derive(Encode)]
pub struct EventRecord<E: Encode> {
    phase: Phase,
    event: TestAllEvents<E>,
    topics: Vec<subxt::config::HashFor<SubstrateConfig>>,
}

impl<E: Encode> EventRecord<E> {
    pub fn new(phase: Phase, event: E) -> Self {
        Self {
            phase,
            event: TestAllEvents::Test(event),
            topics: Vec::new(),
        }
    }
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub enum TestAllEvents<E> {
    Test(E),
}

pub fn test_metadata<E: TypeInfo + 'static>() -> Metadata {
    #[derive(TypeInfo)]
    struct ExtrinsicType<Call> {
        call: Call,
    }
    #[derive(TypeInfo)]
    enum RuntimeCall {
        PalletName(Pallet),
    }
    #[derive(TypeInfo)]
    enum Pallet {
        SomeCall,
    }

    let pallets = vec![PalletMetadata {
        name: "Test",
        storage: None,
        calls: None,
        event: Some(PalletEventMetadata {
            ty: meta_type::<E>(),
        }),
        constants: vec![],
        error: None,
        index: 0,
        docs: vec![],
    }];

    let extrinsic = ExtrinsicMetadata {
        version: 0,
        signed_extensions: vec![],
        address_ty: meta_type::<()>(),
        call_ty: meta_type::<RuntimeCall>(),
        signature_ty: meta_type::<()>(),
        extra_ty: meta_type::<()>(),
    };

    let meta = RuntimeMetadataV15::new(
        pallets,
        extrinsic,
        meta_type::<()>(),
        vec![],
        OuterEnums {
            call_enum_ty: meta_type::<()>(),
            event_enum_ty: meta_type::<TestAllEvents<E>>(),
            error_enum_ty: meta_type::<()>(),
        },
        CustomMetadata {
            map: Default::default(),
        },
    );
    let runtime_metadata: RuntimeMetadataPrefixed = meta.into();
    let metadata: subxt_metadata::Metadata = runtime_metadata.try_into().unwrap();
    Metadata::from(metadata)
}

pub fn events<E: Decode + Encode>(
    metadata: Metadata,
    records: Vec<EventRecord<E>>,
) -> Events<SubstrateConfig> {
    let num_events = records.len() as u32;
    let mut event_bytes = Vec::new();
    for ev in records {
        ev.encode_to(&mut event_bytes);
    }
    let mut all_event_bytes = parity_scale_codec::Compact(num_events).encode();
    all_event_bytes.extend(event_bytes);
    subxt::events::Events::decode_from(all_event_bytes, metadata)
}
