use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/version-chains";

/// Re-linearize forked version chains for single-lineage entities
/// (OIDecision / OIIntent / OITask).
///
/// With `ids` set, repairs exactly those roots; otherwise scans the requested
/// `labels` for forked roots and repairs each. `dry_run` reports the roots that
/// would be repaired without writing.
pub async fn version_chains(
    labels: Option<&str>,
    ids: &[String],
    dry_run: bool,
) -> Result<(), CliError> {
    let mut body = json!({ "dry_run": dry_run });
    if let Some(l) = labels {
        body["labels"] = Value::String(l.to_string());
    }
    if !ids.is_empty() {
        body["ids"] = Value::Array(ids.iter().map(|s| Value::String(s.clone())).collect());
    }
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/repair"), &body).await?;
    output::print_success(resp);
    Ok(())
}
