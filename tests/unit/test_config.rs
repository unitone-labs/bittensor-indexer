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

use flamewire_bittensor_indexer::config::IndexerConfig;
use flamewire_bittensor_indexer::IndexerError;

#[tokio::test]
async fn builder_valid() {
    let cfg = IndexerConfig::builder()
        .node_url("wss://node")
        .with_sqlite("sqlite://:memory:")
        .start_from_block(10)
        .end_at_block(20)
        .build()
        .expect("should build");
    assert_eq!(cfg.node_url, "wss://node");
    assert_eq!(cfg.database_url.as_deref(), Some("sqlite://:memory:"));
    assert_eq!(cfg.start_block, Some(10));
    assert_eq!(cfg.end_block, Some(20));
}

#[tokio::test]
async fn builder_empty_node_url() {
    let result = IndexerConfig::builder().node_url("").build();
    let err = result.err().unwrap();
    match err {
        IndexerError::InvalidConfig { field, .. } => assert_eq!(field, "node_url"),
        _ => panic!("wrong error: {err:?}"),
    }
}

#[tokio::test]
async fn builder_wrong_format() {
    let result = IndexerConfig::builder()
        .node_url("http://localhost")
        .build();
    let err = result.err().unwrap();
    assert!(format!("{err}").contains("must start"));
}

#[tokio::test]
async fn builder_empty_db_url() {
    let result = IndexerConfig::builder()
        .node_url("ws://node")
        .with_postgres("")
        .build();
    let err = result.err().unwrap();
    match err {
        IndexerError::InvalidConfig { field, .. } => assert_eq!(field, "database_url"),
        _ => panic!("wrong error"),
    }
}

#[tokio::test]
async fn builder_end_before_start() {
    let result = IndexerConfig::builder()
        .node_url("ws://node")
        .start_from_block(10)
        .end_at_block(5)
        .build();
    let err = result.err().unwrap();
    match err {
        IndexerError::InvalidConfig { field, .. } => assert_eq!(field, "end_block"),
        _ => panic!("wrong error"),
    }
}

#[tokio::test]
async fn builder_end_at_block() {
    let cfg = IndexerConfig::builder()
        .node_url("ws://node")
        .end_at_block(50)
        .build()
        .expect("should build");
    assert_eq!(cfg.end_block, Some(50));
}
