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
use flamewire_bittensor_indexer::{config::IndexerConfig, CheckpointStore, IndexerError};
use once_cell::sync::Lazy;
use proptest::prelude::*;
use tokio::runtime::Runtime;

static RT: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

#[derive(Clone, Debug)]
struct OwnedEventFilter {
    pallet: Option<String>,
    event: Option<String>,
}

impl OwnedEventFilter {
    fn all() -> Self {
        Self {
            pallet: None,
            event: None,
        }
    }

    fn pallet(pallet: String) -> Self {
        Self {
            pallet: Some(pallet),
            event: None,
        }
    }

    fn event(pallet: String, event: String) -> Self {
        Self {
            pallet: Some(pallet),
            event: Some(event),
        }
    }

    fn matches(&self, pallet: &str, event: &str) -> bool {
        match (self.pallet.as_deref(), self.event.as_deref()) {
            (Some(p), Some(e)) => p == pallet && e == event,
            (Some(p), None) => p == pallet,
            (None, None) => true,
            _ => false,
        }
    }
}

// Event filter properties
#[test]
fn prop_event_filter_logic() {
    proptest!(|(p in "[a-zA-Z0-9]{0,20}", e in "[a-zA-Z0-9]{0,20}",
                other_p in "[a-zA-Z0-9]{0,20}", other_e in "[a-zA-Z0-9]{0,20}")| {
        let filter_all = OwnedEventFilter::all();
        assert!(filter_all.matches(&p, &e));

        prop_assume!(other_p != p);
        let filter_pallet = OwnedEventFilter::pallet(p.clone());
        assert!(filter_pallet.matches(&p, &e));
        assert!(!filter_pallet.matches(&other_p, &e));

        let filter_event = OwnedEventFilter::event(p.clone(), e.clone());
        assert!(filter_event.matches(&p, &e));
        if other_e != e {
            assert!(!filter_event.matches(&p, &other_e));
        }
        assert_eq!(filter_event.matches(&p, &e), filter_event.matches(&p, &e));
    });
}

// Checkpoint store properties
#[test]
fn prop_checkpoint_consistency() {
    proptest!(|(blocks in proptest::collection::vec(0u64..1_000_000, 1..5))| {
        let store = MockCheckpointStore::new();
        let mut last = 0u64;
        for b in &blocks {
            RT.block_on(async { store.store_checkpoint(*b).await.unwrap() });
            last = *b;
            let loaded = RT.block_on(async { store.load_checkpoint().await.unwrap() });
            assert_eq!(loaded, Some(*b));
        }
        let final_loaded = RT.block_on(async { store.load_checkpoint().await.unwrap() });
        assert_eq!(final_loaded, Some(last));
    });
}

#[test]
fn prop_empty_store() {
    let store = MockCheckpointStore::new();
    let res = RT.block_on(async { store.load_checkpoint().await.unwrap() });
    assert_eq!(res, None);
}

// Configuration validation properties
#[test]
fn prop_config_validation() {
    proptest!(|(secure in any::<bool>(), host in "[a-zA-Z0-9.-]{1,20}", port in 1u32..65535, db in "[a-zA-Z0-9_/]{1,20}")| {
        let proto = if secure { "wss" } else { "ws" };
        let url = format!("{proto}://{host}:{port}/rpc");
        let db_url = format!("postgres://{db}@localhost/db");

        let cfg = IndexerConfig { node_url: url.clone(), database_url: Some(db_url.clone()), start_block: Some(1), end_block: None };
        assert!(cfg.validate().is_ok());

        let built = IndexerConfig::builder()
            .node_url(url.clone())
            .with_postgres(db_url.clone())
            .start_from_block(1)
            .build()
            .unwrap();
        assert_eq!(built.node_url, cfg.node_url);
        assert_eq!(built.database_url, cfg.database_url);

        let bad = format!("ftp://{host}:{port}/rpc");
        let err = IndexerConfig::builder().node_url(bad).build().err().unwrap();
        match err { IndexerError::InvalidConfig { field, .. } => assert_eq!(field, "node_url"), _ => panic!("wrong error") }

        let err = IndexerConfig::builder().node_url("ws://n").with_postgres("").build().err().unwrap();
        match err { IndexerError::InvalidConfig { field, .. } => assert_eq!(field, "database_url"), _ => panic!("wrong error") }
    });
}

#[test]
fn prop_filter_composition_laws() {
    proptest!(|(p in "[a-zA-Z0-9]{0,20}", e1 in "[a-zA-Z0-9]{0,20}", e2 in "[a-zA-Z0-9]{0,20}")| {
        let all = OwnedEventFilter::all();
        let pallet = OwnedEventFilter::pallet(p.clone());
        let fe1 = OwnedEventFilter::event(p.clone(), e1.clone());
        let fe2 = OwnedEventFilter::event(p.clone(), e2.clone());

        // transitivity: event implies pallet implies all
        if fe1.matches(&p, &e1) {
            assert!(pallet.matches(&p, &e1));
            assert!(all.matches(&p, &e1));
        }

        if pallet.matches(&p, &e1) {
            assert!(all.matches(&p, &e1));
        }

        // commutativity of composition
        let c1 = fe1.matches(&p, &e1) && fe2.matches(&p, &e1);
        let c2 = fe2.matches(&p, &e1) && fe1.matches(&p, &e1);
        assert_eq!(c1, c2);
    });
}

#[test]
fn prop_checkpoint_monotonic_updates() {
    proptest!(|(a in 0u64..1000, b in 0u64..1000)| {
        let store = MockCheckpointStore::new();
        RT.block_on(async { store.store_checkpoint(a).await.unwrap() });
        RT.block_on(async { store.store_checkpoint(b).await.unwrap() });
        let last = RT.block_on(async { store.load_checkpoint().await.unwrap() });
        assert_eq!(last, Some(b));
    });
}

#[test]
fn prop_config_boundary_values() {
    proptest!(|(long in "[a-zA-Z0-9]{200,300}", port in prop_oneof![Just(0u32), Just(65536u32)])| {
        let url = format!("ws://{long}:{port}");
        let result = IndexerConfig::builder()
            .node_url(url.clone())
            .with_postgres("postgres://user@localhost/db")
            .build();
        assert!(result.is_ok());

        let empty = IndexerConfig::builder().node_url("").build();
        assert!(empty.is_err());
    });
}
