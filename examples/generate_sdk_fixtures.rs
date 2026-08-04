//! Reproducibly generate wire fixtures using the official modular Solana SDK.
//!
//! Run: `cargo run --example generate_sdk_fixtures`

use std::{fs, path::Path};

use serde::Serialize;
use solana_guard::core::base64;
use solana_hash::Hash;
use solana_message::Message;
use solana_signature::Signature;
use solana_system_interface::instruction::{advance_nonce_account, transfer};
use solana_transaction::Transaction;

#[derive(Serialize)]
struct Fixture<'a> {
    name: &'a str,
    provenance: Provenance<'a>,
    transaction_base64: String,
    expected: Expected<'a>,
}

#[derive(Serialize)]
struct Provenance<'a> {
    generator: &'a str,
    solana_transaction: &'a str,
    solana_message: &'a str,
    solana_system_interface: &'a str,
    note: &'a str,
}

#[derive(Serialize)]
struct Expected<'a> {
    version: &'a str,
    instruction_count: usize,
    lamports: u64,
    payer: String,
    recipient: String,
    nonce_account: Option<String>,
    nonce_authority: Option<String>,
    nonce_value: String,
}

fn main() {
    let fixture_dir = Path::new("fixtures/sdk");
    fs::create_dir_all(fixture_dir).expect("create fixture directory");

    let payer = solana_transaction::Address::new_from_array([1; 32]);
    let recipient = solana_transaction::Address::new_from_array([2; 32]);
    let nonce_account = solana_transaction::Address::new_from_array([3; 32]);
    let nonce_authority = payer;
    let recent = Hash::new_from_array([4; 32]);
    let durable = Hash::new_from_array([5; 32]);
    let lamports = 100_000_000;
    let payment = transfer(&payer, &recipient, lamports);

    let ordinary_message =
        Message::new_with_blockhash(std::slice::from_ref(&payment), Some(&payer), &recent);
    let ordinary = Transaction {
        signatures: vec![
            Signature::default();
            ordinary_message.header.num_required_signatures as usize
        ],
        message: ordinary_message,
    };
    write_fixture(
        fixture_dir.join("sol-transfer.json"),
        Fixture {
            name: "official-sdk-sol-transfer",
            provenance: provenance(),
            transaction_base64: encode_transaction(&ordinary),
            expected: Expected {
                version: "legacy",
                instruction_count: 1,
                lamports,
                payer: payer.to_string(),
                recipient: recipient.to_string(),
                nonce_account: None,
                nonce_authority: None,
                nonce_value: recent.to_string(),
            },
        },
    );

    let nonce_ix = advance_nonce_account(&nonce_account, &nonce_authority);
    let mut nonce_message = Message::new_with_nonce(
        vec![payment],
        Some(&payer),
        &nonce_account,
        &nonce_authority,
    );
    nonce_message.recent_blockhash = durable.clone();
    assert_eq!(nonce_message.instructions.len(), 2);
    assert_eq!(nonce_message.instructions[0].data, nonce_ix.data);
    let durable_tx = Transaction {
        signatures: vec![
            Signature::default();
            nonce_message.header.num_required_signatures as usize
        ],
        message: nonce_message,
    };
    write_fixture(
        fixture_dir.join("durable-nonce-sol-transfer.json"),
        Fixture {
            name: "official-sdk-durable-nonce-sol-transfer",
            provenance: provenance(),
            transaction_base64: encode_transaction(&durable_tx),
            expected: Expected {
                version: "legacy",
                instruction_count: 2,
                lamports,
                payer: payer.to_string(),
                recipient: recipient.to_string(),
                nonce_account: Some(nonce_account.to_string()),
                nonce_authority: Some(nonce_authority.to_string()),
                nonce_value: durable.to_string(),
            },
        },
    );
}

fn provenance() -> Provenance<'static> {
    Provenance {
        generator: "examples/generate_sdk_fixtures.rs",
        solana_transaction: "4.2.0",
        solana_message: "4.5.0",
        solana_system_interface: "3.3.0",
        note: "Generated from official modular Solana crates; signatures are placeholders and fixtures are not broadcastable.",
    }
}

fn encode_transaction(transaction: &Transaction) -> String {
    base64::encode(&bincode::serialize(transaction).expect("serialize official SDK transaction"))
}

fn write_fixture(path: impl AsRef<Path>, fixture: Fixture<'_>) {
    let json = serde_json::to_string_pretty(&fixture).expect("serialize fixture") + "\n";
    fs::write(path, json).expect("write fixture");
}
