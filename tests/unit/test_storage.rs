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

#[cfg(feature = "json-storage")]
use flamewire_bittensor_indexer::storage::json::JsonStore;
#[cfg(feature = "postgres")]
use flamewire_bittensor_indexer::storage::postgres::PostgreSQLStore;
#[cfg(feature = "sqlite")]
use flamewire_bittensor_indexer::storage::sqlite::SQLiteStore;
use flamewire_bittensor_indexer::CheckpointStore;
#[cfg(feature = "postgres")]
use flamewire_bittensor_indexer::IndexerError;
#[cfg(feature = "json-storage")]
use tempfile::tempdir;

#[cfg(feature = "json-storage")]
#[tokio::test]
async fn json_store_cycle() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("chk.json");
    let store = JsonStore::new(&path);
    assert_eq!(store.load_checkpoint().await.unwrap(), None);
    store.store_checkpoint(5).await.unwrap();
    assert_eq!(store.load_checkpoint().await.unwrap(), Some(5));
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_store_cycle() {
    let store = SQLiteStore::new("sqlite::memory:").await.unwrap();
    assert_eq!(store.load_checkpoint().await.unwrap(), None);
    store.store_checkpoint(7).await.unwrap();
    assert_eq!(store.load_checkpoint().await.unwrap(), Some(7));
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn postgres_store_failure() {
    let err = PostgreSQLStore::new("postgres://invalid:invalid@localhost:1/db").await;
    match err {
        Err(IndexerError::CheckpointError { backend, .. }) => assert_eq!(backend, "postgres"),
        _ => panic!("unexpected result"),
    }
}
