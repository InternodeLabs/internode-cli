use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/intents";

pub async fn list(limit: Option<i64>, offset: Option<i64>) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut params = vec![];
    if let Some(l) = limit { params.push(format!("limit={l}")); }
    if let Some(o) = offset { params.push(format!("offset={o}")); }
    let qs = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    let resp = client.get(&format!("{BASE}{qs}")).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn inspect(id: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let resp = client.get(&format!("{BASE}/{id}/relationships")).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn update(
    id: &str,
    title: Option<&str>,
    statement: Option<&str>,
    scope: Option<&str>,
    signals: Option<&str>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut body = json!({});
    if let Some(t) = title { body["intent_title"] = Value::String(t.to_string()); }
    if let Some(s) = statement { body["statement"] = Value::String(s.to_string()); }
    if let Some(s) = scope { body["scope"] = Value::String(s.to_string()); }
    if let Some(sigs) = signals {
        let parts: Vec<Value> = sigs
            .split(',')
            .map(|s| Value::String(s.trim().to_string()))
            .filter(|v| !matches!(v, Value::String(s) if s.is_empty()))
            .collect();
        body["signals"] = Value::Array(parts);
    }
    let resp = client.patch(&format!("{BASE}/{id}"), &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn archive(id: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/{id}/archive"), &json!({})).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn merge(source_id: &str, target_id: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let body = json!({ "target_intent_id": target_id });
    let resp = client.post(&format!("{BASE}/{source_id}/merge"), &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn add_signal(id: &str, signals: &[String]) -> Result<(), CliError> {
    if signals.is_empty() {
        return Err(CliError::BadInput(
            "At least one --signal is required.".into(),
        ));
    }
    let body = json!({
        "signals": signals.iter().map(|s| Value::String(s.clone())).collect::<Vec<_>>(),
    });
    let client = ApiClient::new()?;
    let resp = client
        .post(&format!("{BASE}/{id}/signals/add"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}

pub async fn remove_signal(id: &str, signals: &[String]) -> Result<(), CliError> {
    if signals.is_empty() {
        return Err(CliError::BadInput(
            "At least one --signal is required.".into(),
        ));
    }
    let body = json!({
        "signals": signals.iter().map(|s| Value::String(s.clone())).collect::<Vec<_>>(),
    });
    let client = ApiClient::new()?;
    let resp = client
        .post(&format!("{BASE}/{id}/signals/remove"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}

pub async fn set_scope(id: &str, scope: &str) -> Result<(), CliError> {
    let body = json!({ "scope": scope });
    let client = ApiClient::new()?;
    let resp = client
        .post(&format!("{BASE}/{id}/set-scope"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}

pub async fn split(
    source_id: &str,
    splits_file: &str,
    archive_source: bool,
    dry_run: bool,
) -> Result<(), CliError> {
    let raw = std::fs::read_to_string(splits_file).map_err(|e| {
        CliError::BadInput(format!("Failed to read splits file '{splits_file}': {e}"))
    })?;
    let splits: Value = serde_json::from_str(&raw).map_err(|e| {
        CliError::BadInput(format!("Splits file is not valid JSON: {e}"))
    })?;
    if !splits.is_array() {
        return Err(CliError::BadInput(
            "Splits file must contain a JSON array of entries.".into(),
        ));
    }
    let body = json!({
        "splits": splits,
        "archive_source": archive_source,
        "dry_run": dry_run,
    });
    let client = ApiClient::new()?;
    let resp = client
        .post(&format!("{BASE}/{source_id}/split"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}

pub async fn consolidate(
    target_id: &str,
    source_ids: &[String],
    statement_strategy: &str,
    scope_strategy: &str,
    signals_strategy: &str,
    dry_run: bool,
) -> Result<(), CliError> {
    if source_ids.is_empty() {
        return Err(CliError::BadInput(
            "At least one --source intent id is required.".into(),
        ));
    }
    let body = json!({
        "target_intent_id": target_id,
        "source_intent_ids": source_ids
            .iter()
            .map(|s| Value::String(s.clone()))
            .collect::<Vec<_>>(),
        "statement_strategy": statement_strategy,
        "scope_strategy": scope_strategy,
        "signals_strategy": signals_strategy,
        "dry_run": dry_run,
    });
    let client = ApiClient::new()?;
    let resp = client
        .post(&format!("{BASE}/consolidate"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}
