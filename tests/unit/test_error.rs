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

use flamewire_bittensor_indexer::error::IndexerError;
use subxt::Error as SubxtError;

#[test]
fn construct_errors() {
    let e = IndexerError::BlockNotFound { block: 1 };
    assert_eq!(format!("{e}"), "Block 1 not found");

    let e = IndexerError::ConnectionFailed {
        url: "wss://node".into(),
        source: Box::new(SubxtError::Other("conn".into())),
    };
    assert!(format!("{e}").contains("Connection to wss://node failed"));

    let e = IndexerError::invalid_config("field", "bad");
    assert!(format!("{e}").contains("Invalid config"));

    let e = IndexerError::HandlerFailed {
        handler: "h".into(),
        block: 1,
        source: Box::new(std::io::Error::other("oops")),
    };
    assert!(format!("{e}").contains("Handler h failed"));

    let e = IndexerError::CheckpointError {
        operation: "load".into(),
        backend: "json".into(),
        source: Box::new(std::io::Error::other("fail")),
    };
    assert!(format!("{e}").contains("Checkpoint load failed"));

    let e = IndexerError::MetadataUpdateFailed {
        source: Box::new(SubxtError::Other("meta".into())),
    };
    assert!(format!("{e}").contains("Metadata update failed"));

    let e = IndexerError::EventDecodingFailed {
        pallet: "p".into(),
        event: "e".into(),
        block: 1,
        source: Box::new(SubxtError::Other("decode".into())),
    };
    assert!(format!("{e}").contains("Failed to decode event"));
}
