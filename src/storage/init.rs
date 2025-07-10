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
#[cfg(feature = "json-storage")]
use crate::storage::json::JsonStore;
#[cfg(feature = "postgres")]
use crate::storage::postgres::PostgreSQLStore;
#[cfg(feature = "sqlite")]
use crate::storage::sqlite::SQLiteStore;
use crate::storage::CheckpointStore;
use std::path::Path;

pub async fn init_store(
    database_url: Option<String>,
) -> Result<Box<dyn CheckpointStore>, IndexerError> {
    if let Some(url) = database_url {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            #[cfg(feature = "postgres")]
            {
                let store = PostgreSQLStore::new(&url).await?;
                return Ok(Box::new(store));
            }
            #[cfg(not(feature = "postgres"))]
            {
                return Err(IndexerError::invalid_config(
                    "database_url",
                    "postgres feature disabled",
                ));
            }
        } else if url.starts_with("sqlite://") {
            #[cfg(feature = "sqlite")]
            {
                let path = url.trim_start_matches("sqlite://");
                let store = SQLiteStore::new(path).await?;
                return Ok(Box::new(store));
            }
            #[cfg(not(feature = "sqlite"))]
            {
                return Err(IndexerError::invalid_config(
                    "database_url",
                    "sqlite feature disabled",
                ));
            }
        } else {
            return Err(IndexerError::invalid_config(
                "database_url",
                "Unsupported database URL",
            ));
        }
    }

    // Default to JSON storage
    #[cfg(feature = "json-storage")]
    {
        let base_dir = Path::new("database");
        if !base_dir.exists() {
            tokio::fs::create_dir_all(base_dir).await?;
        }
        let json_path = base_dir.join("checkpoint.json");
        let store = JsonStore::new(json_path);
        Ok(Box::new(store))
    }

    #[cfg(not(feature = "json-storage"))]
    Err(IndexerError::invalid_config(
        "storage",
        "No storage backend enabled",
    ))
}
