//! Differential and property assurance for the SDK-less wire decoder.

use std::collections::HashMap;

use proptest::prelude::*;
use serde_json::Value;
use solana_guard::core::tx::{decode_transaction_base64, decode_transaction_bytes};
use solana_guard::core::{base58, base64};
use solana_guard::guard::{analyze_with_intent, GuardConfig, GuardIntent, Verdict};
use solana_transaction::Transaction;

fn fixture(name: &str) -> Value {
    let body = match name {
        "ordinary" => include_str!("../fixtures/sdk/sol-transfer.json"),
        "nonce" => include_str!("../fixtures/sdk/durable-nonce-sol-transfer.json"),
        _ => unreachable!(),
    };
    serde_json::from_str(body).expect("valid checked-in SDK fixture")
}

fn string<'a>(value: &'a Value, pointer: &str) -> &'a str {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .expect("fixture string")
}

#[test]
fn sdk_wire_decoder_agrees_with_official_transaction_deserializer() {
    for fixture in [fixture("ordinary"), fixture("nonce")] {
        let encoded = string(&fixture, "/transaction_base64");
        let raw = base64::decode(encoded).expect("base64");
        let official: Transaction = bincode::deserialize(&raw).expect("official SDK decode");
        let decoded = decode_transaction_base64(encoded).expect("guard decode");

        assert_eq!(decoded.signatures.len(), official.signatures.len());
        assert_eq!(
            decoded.message.instructions.len(),
            official.message.instructions.len()
        );
        assert_eq!(
            decoded.message.account_keys.len(),
            official.message.account_keys.len()
        );
        assert_eq!(
            decoded.message.recent_blockhash.as_slice(),
            official.message.recent_blockhash.as_ref()
        );
        for (ours, sdk) in decoded
            .message
            .account_keys
            .iter()
            .zip(&official.message.account_keys)
        {
            assert_eq!(ours.as_bytes(), sdk.as_ref());
        }
        for (ours, sdk) in decoded
            .message
            .instructions
            .iter()
            .zip(&official.message.instructions)
        {
            assert_eq!(ours.program_id_index, sdk.program_id_index);
            assert_eq!(ours.accounts, sdk.accounts);
            assert_eq!(ours.data, sdk.data);
        }
    }
}

#[test]
fn accepts_fully_bound_official_sdk_durable_nonce_payment() {
    let fixture = fixture("nonce");
    let recipient = string(&fixture, "/expected/recipient");
    let nonce_account = string(&fixture, "/expected/nonce_account");
    let nonce_authority = string(&fixture, "/expected/nonce_authority");
    let nonce_value = string(&fixture, "/expected/nonce_value");

    let mut section = HashMap::new();
    section.insert("max_sol_lamports".into(), "100000000".into());
    section.insert("allowed_recipients".into(), recipient.into());
    section.insert("require_durable_nonce".into(), "true".into());
    section.insert("allowed_nonce_accounts".into(), nonce_account.into());
    section.insert("allowed_nonce_authorities".into(), nonce_authority.into());
    let config = GuardConfig::from_section(&section);
    let intent = GuardIntent {
        description: "Pay with the approved offline durable nonce".into(),
        expected_recipient: recipient.into(),
        expected_mint: None,
        max_lamports: 100_000_000,
        max_token_amount: 0,
        expected_nonce_account: Some(nonce_account.into()),
        expected_nonce_authority: Some(nonce_authority.into()),
        expected_nonce_value: Some(nonce_value.into()),
    };

    let report = analyze_with_intent(
        string(&fixture, "/transaction_base64"),
        &config,
        Some(&intent),
    )
    .expect("analyze");
    assert_eq!(report.verdict, Verdict::Allow);
    assert!(report.uses_durable_nonce);
    assert!(report.nonce_bound);
    assert!(report.transaction_digest.starts_with("sha256:"));
    assert_eq!(report.transaction_digest.len(), 71);
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == "DURABLE_NONCE_ADVANCE"));
}

#[test]
fn rejects_sdk_nonce_advance_when_not_first() {
    let fixture = fixture("nonce");
    let encoded = string(&fixture, "/transaction_base64");
    let raw = base64::decode(encoded).expect("base64");
    let mut transaction: Transaction = bincode::deserialize(&raw).expect("official decode");
    transaction.message.instructions.swap(0, 1);
    let reordered = base64::encode(&bincode::serialize(&transaction).expect("official encode"));

    let report = analyze_with_intent(&reordered, &GuardConfig::default(), None).expect("analyze");
    assert_eq!(report.verdict, Verdict::Reject);
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == "NONCE_ADVANCE_NOT_FIRST"));
}

#[test]
fn rejects_ordinary_sdk_transaction_when_nonce_is_required() {
    let fixture = fixture("ordinary");
    let mut section = HashMap::new();
    section.insert("require_durable_nonce".into(), "true".into());
    let report = analyze_with_intent(
        string(&fixture, "/transaction_base64"),
        &GuardConfig::from_section(&section),
        None,
    )
    .expect("analyze");
    assert_eq!(report.verdict, Verdict::Reject);
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == "DURABLE_NONCE_REQUIRED"));
}

#[test]
fn rejects_wrong_durable_nonce_value() {
    let fixture = fixture("nonce");
    let recipient = string(&fixture, "/expected/recipient");
    let nonce_account = string(&fixture, "/expected/nonce_account");
    let nonce_authority = string(&fixture, "/expected/nonce_authority");
    let mut section = HashMap::new();
    section.insert("max_sol_lamports".into(), "100000000".into());
    section.insert("allowed_recipients".into(), recipient.into());
    section.insert("allowed_nonce_accounts".into(), nonce_account.into());
    section.insert("allowed_nonce_authorities".into(), nonce_authority.into());
    let intent = GuardIntent {
        description: "Mismatched nonce proof".into(),
        expected_recipient: recipient.into(),
        expected_mint: None,
        max_lamports: 100_000_000,
        max_token_amount: 0,
        expected_nonce_account: Some(nonce_account.into()),
        expected_nonce_authority: Some(nonce_authority.into()),
        expected_nonce_value: Some(base58::encode(&[9; 32])),
    };
    let report = analyze_with_intent(
        string(&fixture, "/transaction_base64"),
        &GuardConfig::from_section(&section),
        Some(&intent),
    )
    .expect("analyze");
    assert_eq!(report.verdict, Verdict::Reject);
    assert!(!report.nonce_bound);
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == "INTENT_NONCE_VALUE_MISMATCH"));
}

proptest! {
    #[test]
    fn arbitrary_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
        let _ = decode_transaction_bytes(&bytes);
    }

    #[test]
    fn trailing_bytes_are_never_accepted(suffix in prop::collection::vec(any::<u8>(), 1..64)) {
        let fixture = fixture("ordinary");
        let mut bytes = base64::decode(string(&fixture, "/transaction_base64")).expect("base64");
        bytes.extend_from_slice(&suffix);
        prop_assert!(decode_transaction_bytes(&bytes).is_err());
    }
}
