//! Pure guard API — decode → narrate → assess → bind intent/policy → verdict.
//! No WASM dependency; host-testable with `cargo test`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::base58;
use crate::core::narrate::narrate_transaction;
use crate::core::programs::{is_token_family, recent_blockhashes_sysvar, system_program};
use crate::core::pubkey::Pubkey;
use crate::core::risk::{assess, max_severity, Finding, Severity};
use crate::core::tx::{
    decode_transaction_base64, CompiledInstruction, DecodeError, DecodedTransaction,
};

/// Structured verdict returned to the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Verdict {
    Allow,
    Hold,
    Reject,
}

/// The user's claimed transaction intent. This is untrusted input and is checked
/// against both the decoded bytes and the operator-owned policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardIntent {
    pub description: String,
    pub expected_recipient: String,
    #[serde(default)]
    pub expected_mint: Option<String>,
    #[serde(default)]
    pub max_lamports: u64,
    #[serde(default)]
    pub max_token_amount: u64,
    /// Durable-nonce account expected at instruction 0, when used.
    #[serde(default)]
    pub expected_nonce_account: Option<String>,
    /// Signer authorized to advance the durable nonce, when used.
    #[serde(default)]
    pub expected_nonce_authority: Option<String>,
    /// Base58 durable nonce value expected in the message blockhash, when known.
    #[serde(default)]
    pub expected_nonce_value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuardReport {
    pub verdict: Verdict,
    pub summary: String,
    pub narration: String,
    pub findings: Vec<Finding>,
    pub intent_bound: bool,
    pub policy_configured: bool,
    pub uses_durable_nonce: bool,
    pub nonce_bound: bool,
    pub tx_version: String,
    pub instruction_count: usize,
    pub account_count: usize,
    /// SHA-256 identity of the exact serialized transaction bytes.
    pub transaction_digest: String,
}

#[derive(Debug, Clone)]
pub struct GuardConfig {
    /// Reject on Critical findings (default true).
    pub reject_on_critical: bool,
    /// Hold on High findings when not rejecting (default true).
    pub hold_on_high: bool,
    /// Hold on Medium findings (default false — Medium alone → ALLOW with notes).
    pub hold_on_medium: bool,
    /// Absolute operator cap for native SOL movement. Zero means no SOL transfer is allowed.
    pub max_sol_lamports: u64,
    /// Absolute operator cap for SPL-token movement. Zero means no token transfer is allowed.
    pub max_token_amount: u64,
    /// Operator-owned recipient allowlist.
    pub allowed_recipients: Vec<Pubkey>,
    /// Operator-owned mint allowlist for checked token transfers.
    pub allowed_mints: Vec<Pubkey>,
    /// Require every accepted transaction to use a policy-bound durable nonce.
    pub require_durable_nonce: bool,
    /// Operator-owned nonce account allowlist.
    pub allowed_nonce_accounts: Vec<Pubkey>,
    /// Operator-owned nonce authority allowlist.
    pub allowed_nonce_authorities: Vec<Pubkey>,
    /// Configuration errors are retained so analysis can fail closed in-band.
    pub config_errors: Vec<String>,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            reject_on_critical: true,
            hold_on_high: true,
            hold_on_medium: false,
            max_sol_lamports: 0,
            max_token_amount: 0,
            allowed_recipients: Vec::new(),
            allowed_mints: Vec::new(),
            require_durable_nonce: false,
            allowed_nonce_accounts: Vec::new(),
            allowed_nonce_authorities: Vec::new(),
            config_errors: Vec::new(),
        }
    }
}

impl GuardConfig {
    /// Build from the flat `string -> string` section the host injects.
    pub fn from_section(section: &HashMap<String, String>) -> Self {
        let mut cfg = Self::default();
        if let Some(v) = section.get("reject_on_critical") {
            cfg.reject_on_critical = parse_bool(v, true);
        }
        if let Some(v) = section.get("hold_on_high") {
            cfg.hold_on_high = parse_bool(v, true);
        }
        if let Some(v) = section.get("hold_on_medium") {
            cfg.hold_on_medium = parse_bool(v, false);
        }
        cfg.max_sol_lamports =
            parse_u64_config(section, "max_sol_lamports", &mut cfg.config_errors);
        cfg.max_token_amount =
            parse_u64_config(section, "max_token_amount", &mut cfg.config_errors);
        cfg.allowed_recipients =
            parse_pubkey_list(section, "allowed_recipients", &mut cfg.config_errors);
        cfg.allowed_mints = parse_pubkey_list(section, "allowed_mints", &mut cfg.config_errors);
        if let Some(v) = section.get("require_durable_nonce") {
            cfg.require_durable_nonce = parse_bool(v, false);
        }
        cfg.allowed_nonce_accounts =
            parse_pubkey_list(section, "allowed_nonce_accounts", &mut cfg.config_errors);
        cfg.allowed_nonce_authorities =
            parse_pubkey_list(section, "allowed_nonce_authorities", &mut cfg.config_errors);
        cfg
    }
}

fn parse_bool(v: &str, default: bool) -> bool {
    match v.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => true,
        "false" | "0" | "no" => false,
        _ => default,
    }
}

fn parse_u64_config(section: &HashMap<String, String>, key: &str, errors: &mut Vec<String>) -> u64 {
    let Some(raw) = section.get(key) else {
        return 0;
    };
    match raw.trim().parse::<u64>() {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!("{key} must be an unsigned integer"));
            0
        }
    }
}

fn parse_pubkey_list(
    section: &HashMap<String, String>,
    key: &str,
    errors: &mut Vec<String>,
) -> Vec<Pubkey> {
    let Some(raw) = section.get(key) else {
        return Vec::new();
    };
    raw.split(',')
        .filter_map(|item| {
            let value = item.trim();
            if value.is_empty() {
                return None;
            }
            match parse_pubkey(value) {
                Ok(pubkey) => Some(pubkey),
                Err(error) => {
                    errors.push(format!("{key}: {error}"));
                    None
                }
            }
        })
        .collect()
}

fn parse_pubkey(value: &str) -> Result<Pubkey, String> {
    let bytes = base58::decode(value).map_err(|_| format!("invalid base58 pubkey {value}"))?;
    Pubkey::from_slice(&bytes).map_err(|_| format!("pubkey {value} is not 32 bytes"))
}

/// Back-compatible entry point. Value transfers fail closed because no intent
/// is supplied; non-value inspection behavior remains unchanged.
pub fn analyze(transaction_base64: &str, cfg: &GuardConfig) -> Result<GuardReport, String> {
    analyze_with_intent(transaction_base64, cfg, None)
}

/// Analyze a transaction and cryptographically bind decoded transfer fields to
/// the caller's intent and the operator-owned policy.
pub fn analyze_with_intent(
    transaction_base64: &str,
    cfg: &GuardConfig,
    intent: Option<&GuardIntent>,
) -> Result<GuardReport, String> {
    let tx = decode_transaction_base64(transaction_base64).map_err(|e| e.to_string())?;
    let raw_transaction = crate::core::base64::decode(transaction_base64.trim())
        .map_err(|error| format!("base64: {error}"))?;
    let transaction_digest = format!("sha256:{:x}", Sha256::digest(&raw_transaction));
    let narration = narrate_transaction(&tx);
    let mut findings = assess(&tx);
    let value_transfer = has_value_transfer(&tx);

    if !cfg.config_errors.is_empty() {
        findings.push(Finding {
            code: "POLICY_CONFIG_INVALID".into(),
            severity: Severity::Critical,
            instruction_index: 0,
            message: cfg.config_errors.join("; "),
        });
    }

    let mut policy_configured = !cfg.allowed_recipients.is_empty()
        && (cfg.max_sol_lamports > 0 || cfg.max_token_amount > 0)
        && cfg.config_errors.is_empty();
    let intent_parsed = intent.map(parse_intent);
    let intent_bound = matches!(intent_parsed, Some(Ok(_)));

    let parsed_intent = intent_parsed
        .as_ref()
        .and_then(|result| result.as_ref().ok());
    let nonce_review = assess_durable_nonce(&tx, cfg, parsed_intent, &mut findings);
    if nonce_review.uses_nonce || cfg.require_durable_nonce {
        policy_configured &=
            !cfg.allowed_nonce_accounts.is_empty() && !cfg.allowed_nonce_authorities.is_empty();
    }

    if value_transfer {
        match intent_parsed.as_ref() {
            None => findings.push(policy_finding(
                "INTENT_UNBOUND",
                Severity::Critical,
                0,
                "Value transfer has no explicit recipient and amount intent",
            )),
            Some(Err(error)) => findings.push(policy_finding(
                "INTENT_INVALID",
                Severity::Critical,
                0,
                error,
            )),
            Some(Ok(parsed)) => assess_transfer_policy(&tx, cfg, parsed, &mut findings),
        }
    }

    findings.sort_by(|a, b| b.severity.cmp(&a.severity));
    let verdict = verdict_from_findings(&findings, cfg);
    let summary = summary_line(verdict, &findings);

    let tx_version = match tx.version {
        crate::core::tx::TxVersion::Legacy => "legacy",
        crate::core::tx::TxVersion::V0 => "v0",
    }
    .to_string();

    Ok(GuardReport {
        verdict,
        summary,
        narration,
        findings,
        intent_bound,
        policy_configured,
        uses_durable_nonce: nonce_review.uses_nonce,
        nonce_bound: nonce_review.bound,
        tx_version,
        instruction_count: tx.message.instructions.len(),
        account_count: tx.message.account_keys.len(),
        transaction_digest,
    })
}

struct ParsedIntent {
    recipient: Pubkey,
    mint: Option<Pubkey>,
    max_lamports: u64,
    max_token_amount: u64,
    nonce_account: Option<Pubkey>,
    nonce_authority: Option<Pubkey>,
    nonce_value: Option<[u8; 32]>,
}

fn parse_intent(intent: &GuardIntent) -> Result<ParsedIntent, String> {
    if intent.description.trim().is_empty() {
        return Err("intent description must not be empty".into());
    }
    let recipient = parse_pubkey(intent.expected_recipient.trim())?;
    let mint = intent
        .expected_mint
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_pubkey(value.trim()))
        .transpose()?;
    let nonce_account = parse_optional_pubkey(intent.expected_nonce_account.as_deref())?;
    let nonce_authority = parse_optional_pubkey(intent.expected_nonce_authority.as_deref())?;
    let nonce_value = intent
        .expected_nonce_value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            let bytes = base58::decode(value.trim())
                .map_err(|_| format!("invalid base58 nonce value {value}"))?;
            bytes
                .try_into()
                .map_err(|_| format!("nonce value {value} is not 32 bytes"))
        })
        .transpose()?;
    Ok(ParsedIntent {
        recipient,
        mint,
        max_lamports: intent.max_lamports,
        max_token_amount: intent.max_token_amount,
        nonce_account,
        nonce_authority,
        nonce_value,
    })
}

fn parse_optional_pubkey(value: Option<&str>) -> Result<Option<Pubkey>, String> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_pubkey(value.trim()))
        .transpose()
}

#[derive(Default)]
struct NonceReview {
    uses_nonce: bool,
    bound: bool,
}

/// Durable nonce safety is structural and fail-closed: the advance must be the
/// first instruction and its account, authority, and blockhash must be bound to
/// both caller intent and operator policy.
fn assess_durable_nonce(
    tx: &DecodedTransaction,
    cfg: &GuardConfig,
    intent: Option<&ParsedIntent>,
    findings: &mut Vec<Finding>,
) -> NonceReview {
    let nonce_indexes: Vec<usize> = tx
        .message
        .instructions
        .iter()
        .enumerate()
        .filter_map(|(index, ix)| {
            (tx.program_id_for(ix) == Some(&system_program()) && read_u32(&ix.data) == Some(4))
                .then_some(index)
        })
        .collect();

    if nonce_indexes.is_empty() {
        if cfg.require_durable_nonce {
            findings.push(policy_finding(
                "DURABLE_NONCE_REQUIRED",
                Severity::Critical,
                0,
                "Operator policy requires a durable nonce transaction",
            ));
        }
        return NonceReview::default();
    }

    if nonce_indexes.len() != 1 || nonce_indexes[0] != 0 {
        findings.push(policy_finding(
            "NONCE_ADVANCE_NOT_FIRST",
            Severity::Critical,
            nonce_indexes[0],
            "A durable nonce transaction must contain exactly one advance instruction at index 0",
        ));
        return NonceReview {
            uses_nonce: true,
            bound: false,
        };
    }

    let ix = &tx.message.instructions[0];
    let Some(nonce_account) = ix.accounts.first().and_then(|index| tx.account_at(*index)) else {
        findings.push(policy_finding(
            "NONCE_MALFORMED",
            Severity::Critical,
            0,
            "Nonce account is unresolved",
        ));
        return NonceReview {
            uses_nonce: true,
            bound: false,
        };
    };
    let Some(authority_index) = ix.accounts.get(2).copied() else {
        findings.push(policy_finding(
            "NONCE_MALFORMED",
            Severity::Critical,
            0,
            "Nonce authority is missing",
        ));
        return NonceReview {
            uses_nonce: true,
            bound: false,
        };
    };
    let Some(authority) = tx.account_at(authority_index) else {
        findings.push(policy_finding(
            "NONCE_MALFORMED",
            Severity::Critical,
            0,
            "Nonce authority is unresolved",
        ));
        return NonceReview {
            uses_nonce: true,
            bound: false,
        };
    };
    let recent_blockhashes_ok = ix.accounts.get(1).and_then(|index| tx.account_at(*index))
        == Some(&recent_blockhashes_sysvar());
    if !recent_blockhashes_ok {
        findings.push(policy_finding(
            "NONCE_SYSVAR_INVALID",
            Severity::Critical,
            0,
            "AdvanceNonceAccount does not reference the RecentBlockhashes sysvar",
        ));
    }
    let nonce_index = usize::from(ix.accounts[0]);
    let required = usize::from(tx.message.header.num_required_signatures);
    let writable_signed =
        required.saturating_sub(usize::from(tx.message.header.num_readonly_signed_accounts));
    let writable_unsigned_end = tx.message.account_keys.len().saturating_sub(usize::from(
        tx.message.header.num_readonly_unsigned_accounts,
    ));
    let nonce_writable = nonce_index < writable_signed
        || (nonce_index >= required && nonce_index < writable_unsigned_end);
    if !nonce_writable {
        findings.push(policy_finding(
            "NONCE_ACCOUNT_NOT_WRITABLE",
            Severity::Critical,
            0,
            "Durable nonce account is not writable in the message header",
        ));
    }
    let authority_is_signer =
        usize::from(authority_index) < usize::from(tx.message.header.num_required_signatures);
    if !authority_is_signer {
        findings.push(policy_finding(
            "NONCE_AUTHORITY_NOT_SIGNER",
            Severity::Critical,
            0,
            "Nonce authority is not a required signer",
        ));
    }

    let operator_bound = recent_blockhashes_ok
        && nonce_writable
        && authority_is_signer
        && cfg.allowed_nonce_accounts.contains(nonce_account)
        && cfg.allowed_nonce_authorities.contains(authority);
    if !cfg.allowed_nonce_accounts.contains(nonce_account) {
        findings.push(policy_finding(
            "NONCE_ACCOUNT_NOT_ALLOWED",
            Severity::Critical,
            0,
            &format!("Nonce account {nonce_account} is not in the operator allowlist"),
        ));
    }
    if !cfg.allowed_nonce_authorities.contains(authority) {
        findings.push(policy_finding(
            "NONCE_AUTHORITY_NOT_ALLOWED",
            Severity::Critical,
            0,
            &format!("Nonce authority {authority} is not in the operator allowlist"),
        ));
    }

    let Some(intent) = intent else {
        findings.push(policy_finding(
            "NONCE_INTENT_UNBOUND",
            Severity::Critical,
            0,
            "Durable nonce has no valid caller intent binding",
        ));
        return NonceReview {
            uses_nonce: true,
            bound: false,
        };
    };
    let intent_bound = intent.nonce_account == Some(*nonce_account)
        && intent.nonce_authority == Some(*authority)
        && intent.nonce_value == Some(tx.message.recent_blockhash);
    if intent.nonce_account != Some(*nonce_account) {
        findings.push(policy_finding(
            "INTENT_NONCE_ACCOUNT_MISMATCH",
            Severity::Critical,
            0,
            "Decoded nonce account differs from caller intent",
        ));
    }
    if intent.nonce_authority != Some(*authority) {
        findings.push(policy_finding(
            "INTENT_NONCE_AUTHORITY_MISMATCH",
            Severity::Critical,
            0,
            "Decoded nonce authority differs from caller intent",
        ));
    }
    if intent.nonce_value != Some(tx.message.recent_blockhash) {
        findings.push(policy_finding(
            "INTENT_NONCE_VALUE_MISMATCH",
            Severity::Critical,
            0,
            "Message blockhash differs from the intended durable nonce value",
        ));
    }

    NonceReview {
        uses_nonce: true,
        bound: operator_bound && intent_bound,
    }
}

fn has_value_transfer(tx: &DecodedTransaction) -> bool {
    tx.message.instructions.iter().any(|ix| {
        tx.program_id_for(ix).is_some_and(|program| {
            (*program == system_program() && matches!(read_u32(&ix.data), Some(2 | 11)))
                || (is_token_family(program) && matches!(ix.data.first(), Some(3 | 12)))
        })
    })
}

fn assess_transfer_policy(
    tx: &DecodedTransaction,
    cfg: &GuardConfig,
    intent: &ParsedIntent,
    findings: &mut Vec<Finding>,
) {
    for (index, ix) in tx.message.instructions.iter().enumerate() {
        let Some(program) = tx.program_id_for(ix) else {
            continue;
        };
        if *program == system_program() {
            match read_u32(&ix.data) {
                Some(2) => assess_sol_transfer(tx, ix, index, 1, cfg, intent, findings),
                Some(11) => assess_sol_transfer(tx, ix, index, 2, cfg, intent, findings),
                _ => {}
            }
        } else if is_token_family(program) {
            match ix.data.first().copied() {
                Some(3) => assess_token_transfer(tx, ix, index, None, 1, cfg, intent, findings),
                Some(12) => assess_token_transfer(tx, ix, index, Some(1), 2, cfg, intent, findings),
                _ => {}
            }
        }
    }
}

fn assess_sol_transfer(
    tx: &DecodedTransaction,
    ix: &CompiledInstruction,
    index: usize,
    recipient_position: usize,
    cfg: &GuardConfig,
    intent: &ParsedIntent,
    findings: &mut Vec<Finding>,
) {
    let Some(amount) = read_u64(ix.data.get(4..12).unwrap_or_default()) else {
        findings.push(policy_finding(
            "TRANSFER_MALFORMED",
            Severity::Critical,
            index,
            "SOL transfer amount is unreadable",
        ));
        return;
    };
    let Some(recipient) = ix
        .accounts
        .get(recipient_position)
        .and_then(|key| tx.account_at(*key))
    else {
        findings.push(policy_finding(
            "TRANSFER_MALFORMED",
            Severity::Critical,
            index,
            "SOL transfer recipient is unresolved",
        ));
        return;
    };
    check_recipient(recipient, index, cfg, intent, findings);
    if cfg.max_sol_lamports == 0 || amount > cfg.max_sol_lamports {
        findings.push(policy_finding(
            "SOL_CAP_EXCEEDED",
            Severity::Critical,
            index,
            &format!(
                "{amount} lamports exceeds operator cap {}",
                cfg.max_sol_lamports
            ),
        ));
    }
    if intent.max_lamports == 0 || amount > intent.max_lamports {
        findings.push(policy_finding(
            "INTENT_AMOUNT_EXCEEDED",
            Severity::Critical,
            index,
            &format!(
                "{amount} lamports exceeds intent cap {}",
                intent.max_lamports
            ),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn assess_token_transfer(
    tx: &DecodedTransaction,
    ix: &CompiledInstruction,
    index: usize,
    mint_position: Option<usize>,
    recipient_position: usize,
    cfg: &GuardConfig,
    intent: &ParsedIntent,
    findings: &mut Vec<Finding>,
) {
    let Some(amount) = read_u64(ix.data.get(1..9).unwrap_or_default()) else {
        findings.push(policy_finding(
            "TRANSFER_MALFORMED",
            Severity::Critical,
            index,
            "Token transfer amount is unreadable",
        ));
        return;
    };
    let Some(recipient) = ix
        .accounts
        .get(recipient_position)
        .and_then(|key| tx.account_at(*key))
    else {
        findings.push(policy_finding(
            "TRANSFER_MALFORMED",
            Severity::Critical,
            index,
            "Token transfer recipient is unresolved",
        ));
        return;
    };
    check_recipient(recipient, index, cfg, intent, findings);
    if cfg.max_token_amount == 0 || amount > cfg.max_token_amount {
        findings.push(policy_finding(
            "TOKEN_CAP_EXCEEDED",
            Severity::Critical,
            index,
            &format!(
                "{amount} token units exceeds operator cap {}",
                cfg.max_token_amount
            ),
        ));
    }
    if intent.max_token_amount == 0 || amount > intent.max_token_amount {
        findings.push(policy_finding(
            "INTENT_AMOUNT_EXCEEDED",
            Severity::Critical,
            index,
            &format!(
                "{amount} token units exceeds intent cap {}",
                intent.max_token_amount
            ),
        ));
    }

    let Some(mint_position) = mint_position else {
        findings.push(policy_finding("TOKEN_MINT_UNRESOLVED", Severity::High, index, "Unchecked token Transfer does not carry a mint; use TransferChecked for policy binding"));
        return;
    };
    let Some(mint) = ix
        .accounts
        .get(mint_position)
        .and_then(|key| tx.account_at(*key))
    else {
        findings.push(policy_finding(
            "TRANSFER_MALFORMED",
            Severity::Critical,
            index,
            "Token mint is unresolved",
        ));
        return;
    };
    if !cfg.allowed_mints.contains(mint) {
        findings.push(policy_finding(
            "MINT_NOT_ALLOWED",
            Severity::Critical,
            index,
            &format!("Mint {mint} is not in the operator allowlist"),
        ));
    }
    match intent.mint {
        Some(expected) if expected != *mint => findings.push(policy_finding(
            "INTENT_MINT_MISMATCH",
            Severity::Critical,
            index,
            &format!("Decoded mint {mint} differs from intended mint {expected}"),
        )),
        None => findings.push(policy_finding(
            "INTENT_MINT_UNBOUND",
            Severity::Critical,
            index,
            "Token transfer intent does not specify a mint",
        )),
        _ => {}
    }
}

fn check_recipient(
    recipient: &Pubkey,
    index: usize,
    cfg: &GuardConfig,
    intent: &ParsedIntent,
    findings: &mut Vec<Finding>,
) {
    if !cfg.allowed_recipients.contains(recipient) {
        findings.push(policy_finding(
            "RECIPIENT_NOT_ALLOWED",
            Severity::Critical,
            index,
            &format!("Recipient {recipient} is not in the operator allowlist"),
        ));
    }
    if intent.recipient != *recipient {
        findings.push(policy_finding(
            "INTENT_RECIPIENT_MISMATCH",
            Severity::Critical,
            index,
            &format!(
                "Decoded recipient {recipient} differs from intended recipient {}",
                intent.recipient
            ),
        ));
    }
}

fn policy_finding(
    code: &str,
    severity: Severity,
    instruction_index: usize,
    message: &str,
) -> Finding {
    Finding {
        code: code.into(),
        severity,
        instruction_index,
        message: message.into(),
    }
}

fn read_u32(data: &[u8]) -> Option<u32> {
    let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn read_u64(data: &[u8]) -> Option<u64> {
    let bytes: [u8; 8] = data.get(..8)?.try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}

pub fn verdict_from_findings(findings: &[Finding], cfg: &GuardConfig) -> Verdict {
    match max_severity(findings) {
        Some(Severity::Critical) if cfg.reject_on_critical => Verdict::Reject,
        Some(Severity::Critical) => Verdict::Hold,
        Some(Severity::High) if cfg.hold_on_high => Verdict::Hold,
        Some(Severity::Medium) if cfg.hold_on_medium => Verdict::Hold,
        _ => Verdict::Allow,
    }
}

fn summary_line(verdict: Verdict, findings: &[Finding]) -> String {
    let top = findings.first().map(|f| f.code.as_str()).unwrap_or("NONE");
    match verdict {
        Verdict::Allow => {
            if findings.is_empty() {
                "ALLOW — transaction matches intent and operator policy".into()
            } else {
                format!("ALLOW — {top} noted, below hold/reject threshold")
            }
        }
        Verdict::Hold => format!("HOLD — review required ({top})"),
        Verdict::Reject => format!("REJECT — policy or safety violation ({top})"),
    }
}

/// Render compact agent-friendly JSON to minimize channel/model token usage.
pub fn report_json(report: &GuardReport) -> String {
    serde_json::to_string(report).unwrap_or_else(|_| "{}".into())
}

/// Map decode errors into a stable tool error string.
pub fn format_decode_error(err: DecodeError) -> String {
    format!("failed to decode transaction: {err}")
}
