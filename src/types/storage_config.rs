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

use serde::{Deserialize, Serialize};

/// PostgreSQL state handler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgStateConfig {
    pub url: String,
    pub max_connections: u32,
    pub run_migrations: bool,
}

impl Default for PgStateConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            run_migrations: true,
        }
    }
}

/// ClickHouse analytics handler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            database: "bittensor".to_string(),
            batch_size: 1000,
            flush_interval_ms: 5000,
        }
    }
}

/// Redis cache handler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCacheConfig {
    pub url: String,
    pub default_ttl_secs: u64,
    pub block_cache_ttl_secs: u64,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            default_ttl_secs: 60,
            block_cache_ttl_secs: 300,
        }
    }
}
