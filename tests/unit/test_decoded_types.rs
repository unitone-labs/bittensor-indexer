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

use flamewire_bittensor_indexer::types::decoded::{
    DecodedBlock, DecodedEvent, DecodedExtrinsic,
};

use serde_json::{json, Value};

#[test]
fn decoded_block_construction_and_fields() {
    let block = DecodedBlock {
        block_number: 100,
        block_hash: "0xabc".to_string(),
        extrinsic_count: 3,
        event_count: 5,
    };
    assert_eq!(block.block_number, 100);
    assert_eq!(block.block_hash, "0xabc");
    assert_eq!(block.extrinsic_count, 3);
    assert_eq!(block.event_count, 5);
}

#[test]
fn decoded_block_json_serialization() {
    let block = DecodedBlock {
        block_number: 42,
        block_hash: "0xdead".to_string(),
        extrinsic_count: 1,
        event_count: 2,
    };
    let val = serde_json::to_value(&block).unwrap();
    assert_eq!(val["block_number"], json!(42));
    assert_eq!(val["block_hash"], json!("0xdead"));
    assert_eq!(val["extrinsic_count"], json!(1));
    assert_eq!(val["event_count"], json!(2));
}

#[test]
fn decoded_block_json_roundtrip() {
    let block = DecodedBlock {
        block_number: 999,
        block_hash: "0xbeef".to_string(),
        extrinsic_count: 10,
        event_count: 20,
    };
    let json_str = serde_json::to_string(&block).unwrap();
    let restored: DecodedBlock = serde_json::from_str(&json_str).unwrap();
    assert_eq!(restored.block_number, block.block_number);
    assert_eq!(restored.block_hash, block.block_hash);
    assert_eq!(restored.extrinsic_count, block.extrinsic_count);
    assert_eq!(restored.event_count, block.event_count);
}

#[test]
fn decoded_extrinsic_with_signer() {
    let ext = DecodedExtrinsic {
        index: 0,
        pallet: "Balances".to_string(),
        call: "transfer".to_string(),
        signer: Some("5GrwvaEF...".to_string()),
        args: json!({"dest": "5FHneW46...", "value": 1000}),
    };
    assert_eq!(ext.index, 0);
    assert_eq!(ext.pallet, "Balances");
    assert_eq!(ext.signer.as_deref(), Some("5GrwvaEF..."));
    assert_eq!(ext.args["value"], json!(1000));
}

#[test]
fn decoded_extrinsic_without_signer() {
    let ext = DecodedExtrinsic {
        index: 1,
        pallet: "Timestamp".to_string(),
        call: "set".to_string(),
        signer: None,
        args: json!({"now": 1234567890}),
    };
    assert!(ext.signer.is_none());
    let val = serde_json::to_value(&ext).unwrap();
    assert_eq!(val["signer"], Value::Null);
}

#[test]
fn decoded_extrinsic_args_serialization() {
    let ext = DecodedExtrinsic {
        index: 2,
        pallet: "SubtensorModule".to_string(),
        call: "set_weights".to_string(),
        signer: Some("5Abc...".to_string()),
        args: json!({"netuid": 1, "uids": [0, 1, 2], "weights": [100, 200, 300]}),
    };
    let val = serde_json::to_value(&ext).unwrap();
    assert_eq!(val["args"]["netuid"], json!(1));
    assert!(val["args"]["uids"].is_array());
}

#[test]
fn decoded_event_construction() {
    let event = DecodedEvent {
        index: 0,
        pallet: "System".to_string(),
        variant: "ExtrinsicSuccess".to_string(),
        fields: json!({"dispatch_info": {"weight": 100}}),
    };
    assert_eq!(event.index, 0);
    assert_eq!(event.pallet, "System");
    assert_eq!(event.variant, "ExtrinsicSuccess");
    assert!(event.fields.is_object());
}

#[test]
fn decoded_event_null_fields() {
    let event = DecodedEvent {
        index: 3,
        pallet: "Unknown".to_string(),
        variant: "Something".to_string(),
        fields: Value::Null,
    };
    let val = serde_json::to_value(&event).unwrap();
    assert_eq!(val["fields"], Value::Null);
}

#[test]
fn decoded_event_json_roundtrip() {
    let event = DecodedEvent {
        index: 7,
        pallet: "Balances".to_string(),
        variant: "Transfer".to_string(),
        fields: json!({"from": "5Abc", "to": "5Def", "amount": 500}),
    };
    let json_str = serde_json::to_string(&event).unwrap();
    let restored: DecodedEvent = serde_json::from_str(&json_str).unwrap();
    assert_eq!(restored.index, event.index);
    assert_eq!(restored.pallet, event.pallet);
    assert_eq!(restored.variant, event.variant);
    assert_eq!(restored.fields, event.fields);
}

#[test]
fn decoded_block_clone() {
    let block = DecodedBlock {
        block_number: 1,
        block_hash: "0x01".to_string(),
        extrinsic_count: 0,
        event_count: 0,
    };
    let cloned = block.clone();
    assert_eq!(cloned.block_number, block.block_number);
    assert_eq!(cloned.block_hash, block.block_hash);
}
