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
use std::fmt;
use std::path::{Path, PathBuf};
use url::Url;

/// Validated websocket URL.
#[derive(Clone, Debug)]
pub struct WebSocketUrl(Url);

impl WebSocketUrl {
    /// Parse and validate a websocket URL (supports both ws:// and wss://).
    pub fn parse(input: &str) -> Result<Self, IndexerError> {
        let url = Url::parse(input)
            .map_err(|_| IndexerError::invalid_config("node_url", "invalid URL"))?;
        match url.scheme() {
            "ws" | "wss" => Ok(Self(url)),
            _ => Err(IndexerError::invalid_config(
                "node_url",
                "must start with ws:// or wss://",
            )),
        }
    }

    /// Get the inner string representation.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for WebSocketUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for WebSocketUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Validated PostgreSQL database URL
#[derive(Clone, Debug)]
pub struct PostgresUrl(Url);

impl PostgresUrl {
    pub fn parse(input: &str) -> Result<Self, IndexerError> {
        let url = Url::parse(input)
            .map_err(|_| IndexerError::invalid_config("database_url", "invalid URL"))?;
        match url.scheme() {
            "postgres" | "postgresql" => Ok(Self(url)),
            _ => Err(IndexerError::invalid_config(
                "database_url",
                "must start with postgres:// or postgresql://",
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for PostgresUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PostgresUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Validated SQLite database URL
#[derive(Clone, Debug)]
pub struct SqliteUrl(PathBuf);

impl SqliteUrl {
    pub fn parse(input: &str) -> Result<Self, IndexerError> {
        if !input.starts_with("sqlite://") {
            return Err(IndexerError::invalid_config(
                "database_url",
                "must start with sqlite://",
            ));
        }
        let path = input.trim_start_matches("sqlite://");
        Ok(Self(PathBuf::from(path)))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for SqliteUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sqlite://{}", self.0.display())
    }
}
