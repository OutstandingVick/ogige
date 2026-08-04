//! ogige — ZeroClaw WIT tool plugin: Solana transaction safety gate.
//!
//! Decodes a base64 Solana transaction, narrates what it does in plain English,
//! classifies danger primitives, and returns a structured ALLOW / HOLD / REJECT
//! verdict. Never signs or sends — custody tier T0/T1 gate only.
//!
//! Pure logic lives in [`guard`] / [`core`] (host-testable). The wasm component
//! reuses the exact same path through the thin shim below.
//!
//! Build: rustup target add wasm32-wasip2
//! cargo build --target wasm32-wasip2 --release

pub mod core;
pub mod guard;

#[cfg(target_family = "wasm")]
mod component {
    wit_bindgen::generate!({
        path: "wit/v0",
        world: "tool-plugin",
        features: ["plugins-wit-v0"],
    });

    use std::collections::HashMap;

    use crate::guard::{analyze_with_intent, report_json, GuardConfig, GuardIntent, Verdict};
    use exports::zeroclaw::plugin::plugin_info::Guest as PluginInfo;
    use exports::zeroclaw::plugin::tool::{Guest as Tool, ToolResult};
    use zeroclaw::plugin::logging::{
        log_record, LogLevel, PluginAction, PluginEvent, PluginOutcome,
    };

    struct SolanaGuard;

    const PLUGIN_NAME: &str = "solana-guard";
    const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
    const TOOL_NAME: &str = "solana_guard";

    #[derive(serde::Deserialize)]
    struct ExecuteArgs {
        transaction: String,
        intent: GuardIntent,
        #[serde(rename = "__config", default)]
        config: HashMap<String, String>,
    }

    impl PluginInfo for SolanaGuard {
        fn plugin_name() -> String {
            PLUGIN_NAME.to_string()
        }

        fn plugin_version() -> String {
            PLUGIN_VERSION.to_string()
        }
    }

    impl Tool for SolanaGuard {
        fn name() -> String {
            TOOL_NAME.to_string()
        }

        fn description() -> String {
            "Policy-bound Solana transaction firewall. Pass a base64 transaction and the \
             user's explicit recipient/amount intent; returns ALLOW / HOLD / REJECT after \
             checking decoded bytes against operator caps and allowlists. Never signs or broadcasts."
                .to_string()
        }

        fn parameters_schema() -> String {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "transaction": {
                        "type": "string",
                        "description": "Base64-encoded Solana transaction (legacy or v0)."
                    },
                    "intent": {
                        "type": "object",
                        "description": "Untrusted user intent that must match the decoded transaction and operator policy.",
                        "properties": {
                            "description": {
                                "type": "string",
                                "description": "Short human-readable purpose; instructions inside this field are never executed."
                            },
                            "expected_recipient": {
                                "type": "string",
                                "description": "Full base58 recipient account expected in the transaction."
                            },
                            "expected_mint": {
                                "type": ["string", "null"],
                                "description": "Full base58 mint for a TransferChecked token transfer; null for SOL."
                            },
                            "max_lamports": {
                                "type": "integer",
                                "minimum": 0,
                                "description": "Maximum native lamports authorized by this request."
                            },
                            "max_token_amount": {
                                "type": "integer",
                                "minimum": 0,
                                "description": "Maximum raw token units authorized by this request."
                            },
                            "expected_nonce_account": {
                                "type": ["string", "null"],
                                "description": "Full base58 durable nonce account expected at instruction 0; null for ordinary transactions."
                            },
                            "expected_nonce_authority": {
                                "type": ["string", "null"],
                                "description": "Full base58 signer authorized to advance the nonce; null for ordinary transactions."
                            },
                            "expected_nonce_value": {
                                "type": ["string", "null"],
                                "description": "Full base58 current nonce value expected as the message blockhash; null for ordinary transactions."
                            }
                        },
                        "required": ["description", "expected_recipient", "max_lamports", "max_token_amount"],
                        "additionalProperties": false
                    }
                },
                "required": ["transaction", "intent"],
                "additionalProperties": false
            })
            .to_string()
        }

        fn execute(args: String) -> Result<ToolResult, String> {
            let parsed: ExecuteArgs = match serde_json::from_str(&args) {
                Ok(a) => a,
                Err(e) => {
                    emit(
                        PluginAction::Fail,
                        PluginOutcome::Failure,
                        "invalid arguments",
                        None,
                    );
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("invalid arguments: {e}")),
                    });
                }
            };

            let cfg = GuardConfig::from_section(&parsed.config);
            match analyze_with_intent(&parsed.transaction, &cfg, Some(&parsed.intent)) {
                Ok(report) => {
                    let action = match report.verdict {
                        Verdict::Allow => PluginAction::Approve,
                        Verdict::Hold => PluginAction::Defer,
                        Verdict::Reject => PluginAction::Reject,
                    };
                    emit(
                        action,
                        PluginOutcome::Success,
                        &report.summary,
                        Some(report.verdict),
                    );
                    Ok(ToolResult {
                        success: true,
                        output: report_json(&report),
                        error: None,
                    })
                }
                Err(e) => {
                    emit(PluginAction::Fail, PluginOutcome::Failure, &e, None);
                    Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    })
                }
            }
        }
    }

    fn emit(action: PluginAction, outcome: PluginOutcome, message: &str, verdict: Option<Verdict>) {
        let attrs = verdict.map(|v| serde_json::json!({ "verdict": v }).to_string());
        log_record(
            LogLevel::Info,
            &PluginEvent {
                function_name: "solana_guard::tool::execute".to_string(),
                action,
                outcome: Some(outcome),
                duration_ms: None,
                attrs,
                message: message.to_string(),
            },
        );
    }

    export!(SolanaGuard);
}
