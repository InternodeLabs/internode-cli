use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/cypher";

/// Set or rotate the per-owner Cypher passphrase.
///
/// Prompts twice for the passphrase via ``rpassword`` so the value
/// never appears on screen or in shell history.  The CLI never stores
/// the passphrase — it is sent over TLS to the API and salted+hashed
/// server-side before being persisted.
pub async fn set_passphrase() -> Result<(), CliError> {
    let pass = rpassword::prompt_password("New Cypher passphrase: ")
        .map_err(|e| CliError::BadInput(format!("Failed to read passphrase: {e}")))?;
    if pass.len() < 12 {
        return Err(CliError::BadInput(
            "Passphrase must be at least 12 characters long.".into(),
        ));
    }
    let confirm = rpassword::prompt_password("Confirm passphrase: ")
        .map_err(|e| CliError::BadInput(format!("Failed to read passphrase: {e}")))?;
    if pass != confirm {
        return Err(CliError::BadInput("Passphrases did not match.".into()));
    }
    let body = json!({ "passphrase": pass });
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/set-passphrase"), &body).await?;
    output::print_success(resp);
    Ok(())
}

/// Run the Cypher blocks in ``file`` after prompting for the passphrase.
///
/// Reads the file from disk, computes its SHA-256 client-side, prompts
/// the user for the Cypher passphrase, and sends ``{content, file_sha256,
/// passphrase, dry_run}`` to ``/cypher/run``.  The agent never sees the
/// passphrase — only the human at the terminal can complete the call.
pub async fn run(file: &str, dry_run: bool) -> Result<(), CliError> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| CliError::BadInput(format!("Failed to read '{file}': {e}")))?;
    if content.trim().is_empty() {
        return Err(CliError::BadInput("Cypher file is empty.".into()));
    }
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let file_sha256 = format!("{:x}", hasher.finalize());

    let pass = rpassword::prompt_password("Cypher passphrase: ")
        .map_err(|e| CliError::BadInput(format!("Failed to read passphrase: {e}")))?;

    let body = json!({
        "content": content,
        "file_sha256": file_sha256,
        "passphrase": pass,
        "dry_run": dry_run,
    });
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/run"), &body).await?;
    // The /cypher/run response carries a non-null ``suggested_followup``
    // string when blocks were mutating; bubble it up to stderr so the
    // user gets a nudge to run ``embeddings sync``.  Stdout still gets
    // the standard JSON envelope so scripts keep working.
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
