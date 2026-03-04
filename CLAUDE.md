# CLAUDE.md - Flamewire Indexer Provider

> **⛔ STOP**: Read this ENTIRE file before making ANY code changes. EVERY rule here is MANDATORY.

---

## 🚨 ABSOLUTE RULES - VIOLATIONS = REJECTED CODE

### 1. FILE SIZE LIMIT

```
⛔ MAXIMUM 200 LINES PER FILE - NO EXCEPTIONS
```

If a file exceeds 200 lines, SPLIT IT before doing anything else.

### 2. IMPORTS - ALWAYS IMPORT, NEVER FULLY-QUALIFIED

```rust
// ✅ CORRECT - Import at top, use short name
use std::time::Duration;
use instant::Instant;
use tracing::{info, debug, warn, error};

let timeout = Duration::from_millis(500);
let start = Instant::now();
info!("started");
```

```rust
// ❌ WRONG - NEVER write fully-qualified paths in code
let timeout = std::time::Duration::from_millis(500);  // ❌ REJECTED
let start = std::time::Instant::now();                 // ❌ REJECTED
tracing::info!("started");                             // ❌ REJECTED
```

### 3. IMPORT ORDER (ALWAYS THIS ORDER)

```rust
// 1. Internal crate imports (crate::)
use crate::{app_state::AppState, types::MyRequest};

// 2. Auth imports
use auth_client::AuthenticatedUser;
use auth_client_macros::roles;

// 3. Axum imports
use axum::{Json, extract::{Path, Query, State}};

// 4. Common/libs imports
use common::{UserRole, error::{ApiError, ApiResult}};

// 5. External crates (alphabetically)
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

// 6. Standard library LAST
use std::sync::Arc;
use std::time::Duration;
```

### 4. COMMENTS - SHORT, FACTUAL, NEUTRAL

```rust
// ✅ CORRECT - Factual, describes what
/// Creates a new client with the given options
// Filter nodes by region.
// Retry on timeout.

// ❌ WRONG - Marketing speak, qualifiers
/// Super optimized function for maximum performance  // ❌ REJECTED
// Advanced algorithm that efficiently handles...      // ❌ REJECTED
// This brilliant solution optimally manages...        // ❌ REJECTED
```

**Rules:**

- `///` for public API (functions, structs, enums)
- `//` for internal notes (only when necessary)
- NO redundant comments that repeat the function name
- NO qualifiers: "optimized", "efficient", "advanced", "robust", "elegant"

### 5. LOGGING LEVELS

```rust
// info!  → Essential high-level events ONLY (created, deleted, completed)
// debug! → Developer details, request params, timing
// warn!  → Non-critical failures (cache miss, retry)
// error! → Critical failures needing attention
// trace! → AVOID unless strictly necessary
```

```rust
// ✅ CORRECT - Structured logging with key=value
debug!(
    user_id = %user.user_id,
    resource_id = %id,
    "Deleting resource"
);

info!(
    user_id = %user.user_id,
    resource_id = %resource.id,
    "Resource created"
);

// ❌ WRONG - Unstructured, verbose
info!("User {} is deleting resource {}", user.user_id, id);  // ❌
debug!("Starting the deletion process for resource");         // ❌
```

**NEVER log:** secrets, tokens, passwords, full API keys, emails

### 6. TYPES LOCATION

```
⛔ NO structs/enums in handler files or business logic files
⛔ ALL types go in src/types/ directory
```

```
src/types/
├── mod.rs
├── user.rs        # User-related types
├── resources.rs   # Resource request/response types
└── config.rs      # Config types
```

### 7. HANDLER RETURN TYPES

```rust
// ✅ CORRECT - Return ApiResult<MyResponse>
pub async fn create_resource(...) -> ApiResult<CreateResourceResponse>

// ❌ WRONG - Never wrap in Json
pub async fn create_resource(...) -> ApiResult<Json<CreateResourceResponse>>  // ❌ REJECTED
```

All response types MUST implement `IntoResponse`:

```rust
impl IntoResponse for CreateResourceResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
```

### 8. HANDLER PARAMETER ORDER

```rust
pub async fn update_resource(
    user: AuthenticatedUser,              // 1. Auth (if using #[roles])
    State(state): State<AppState>,        // 2. State
    Path(id): Path<Uuid>,                 // 3. Path params
    Query(query): Query<ListQuery>,       // 4. Query params
    Json(req): Json<UpdateRequest>,       // 5. Body ALWAYS LAST
) -> ApiResult<UpdateResponse>
```

---

## Project Overview

Flamewire is a **decentralized RPC provider** for the Bittensor ecosystem. It handles 20,000+ requests/second across multiple geographic regions with a microservices architecture.

**Tech Stack**: Rust, Tokio, Axum, PostgreSQL (sqlx), Redis, NATS, Kubernetes

---

## ⚠️ MANDATORY RULES FOR CLAUDE

### Before ANY Implementation

1. **DO NOT ASSUME** - Always `cat` or read files before suggesting changes
2. **CHECK EXISTING PATTERNS** - Look at similar implementations in the codebase first
3. **ASK if uncertain** - Better to ask than to guess wrong

### Investigation Protocol

When asked to investigate or debug:

```
1. FIRST: Read the relevant source files completely
2. THEN: Trace the flow through the code
3. THEN: Check related libs/ for shared logic
4. ONLY THEN: Form a hypothesis and explain what you found
```

### Implementation Protocol

When asked to implement something:

```
1. FIRST: Find 2-3 similar implementations in the codebase
2. THEN: Show me the patterns you found
3. THEN: Propose implementation following those EXACT patterns
4. WAIT for approval before writing code
```

## Questions to Ask Before Implementing

1. "Is there existing code that does something similar?"
2. "Which lib should this logic live in?"
3. "What events need to be published/subscribed?"
4. "What config values should be dynamic?"
5. "What errors can occur and how are similar errors handled?"

---
