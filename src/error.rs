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
use serde_json;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Subxt error: {0}")]
    Subxt(Box<subxt::Error>),

    #[error("Database error: {0}")]
    Database(Box<sqlx::Error>),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "json-storage")]
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Connection to {url} failed: {source}")]
    ConnectionFailed {
        url: String,
        #[source]
        source: Box<subxt::Error>,
    },

    #[error("Block {block} not found")]
    BlockNotFound { block: u64 },

    #[error("Handler {handler} failed at block {block}: {source}")]
    HandlerFailed {
        handler: String,
        block: u64,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("Invalid config for `{field}`: {message}")]
    InvalidConfig { field: String, message: String },

    #[error("Checkpoint {operation} failed using {backend}: {source}")]
    CheckpointError {
        operation: String,
        backend: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("Metadata update failed: {source}")]
    MetadataUpdateFailed {
        #[source]
        source: Box<subxt::Error>,
    },

    #[error("Failed to decode event {pallet}.{event} in block {block}: {source}")]
    EventDecodingFailed {
        pallet: String,
        event: String,
        block: u64,
        #[source]
        source: Box<subxt::Error>,
    },
}

impl IndexerError {
    pub fn invalid_config(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl From<subxt::Error> for IndexerError {
    fn from(err: subxt::Error) -> Self {
        Self::Subxt(Box::new(err))
    }
}

impl From<Box<subxt::Error>> for IndexerError {
    fn from(err: Box<subxt::Error>) -> Self {
        Self::Subxt(err)
    }
}

impl From<sqlx::Error> for IndexerError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(Box::new(err))
    }
}
