use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi";

/// Minimum length of the search string. Replacing very short fragments across an
/// entire knowledge graph is almost never an intended correction, so the CLI
/// rejects it locally (the API enforces the same rule authoritatively).
const MIN_SEARCH_LEN: usize = 4;

/// Replace ``search`` with ``replacement`` across every text property of every
/// node the caller owns.
///
/// Dry-run by default: with ``apply`` false the API previews the matches and
/// writes nothing. With ``apply`` true the rewrite is performed in place and the
/// response carries a ``suggested_followup`` nudge (``internode embeddings
/// sync``) which is surfaced to stderr.
pub async fn replace_text(search: &str, replacement: &str, apply: bool) -> Result<(), CliError> {
    if search.chars().count() < MIN_SEARCH_LEN {
        return Err(CliError::BadInput(format!(
            "search must be at least {MIN_SEARCH_LEN} characters long."
        )));
    }
    if search == replacement {
        return Err(CliError::BadInput(
            "search and replacement are identical; nothing to do.".into(),
        ));
    }

    let body = json!({
        "search": search,
        "replace": replacement,
        "apply": apply,
    });
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/replace-text"), &body).await?;

    // After a real write the API returns a non-null ``suggested_followup``
    // (``internode embeddings sync``); bubble it to stderr so the user is nudged
    // to re-align semantic search. Stdout still gets the JSON envelope.
    if let Some(hint) = resp
        .get("suggested_followup")
        .and_then(|v: &Value| v.as_str())
    {
        if !hint.is_empty() {
            eprintln!("hint: {hint}");
        }
    }
    output::print_success(resp);
    Ok(())
}
