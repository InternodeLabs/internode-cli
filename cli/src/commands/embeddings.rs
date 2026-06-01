use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

pub async fn status() -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let resp = client
        .get("/internode-tools/cli/admin/embeddings/status")
        .await?;
    output::print_success(resp);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn sync(
    scope: Option<&str>,
    ids: Option<Vec<String>>,
    since: Option<&str>,
    force: bool,
    dry_run: bool,
    no_wait: bool,
    timeout: Option<i64>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut body = json!({
        "scope": scope.unwrap_or("all"),
        "force": force,
        "dry_run": dry_run,
        "no_wait": no_wait,
    });
    if let Some(list) = ids {
        body["ids"] = Value::Array(list.into_iter().map(Value::String).collect());
    }
    if let Some(s) = since {
        body["since"] = Value::String(s.to_string());
    }
    if let Some(t) = timeout {
        body["timeout"] = Value::Number(t.into());
    }
    let resp = client
        .post("/internode-tools/cli/admin/embeddings/sync", &body)
        .await?;
    output::print_success(resp);
    Ok(())
}
