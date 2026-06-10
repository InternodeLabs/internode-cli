use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/decisions";

pub async fn list(
    search: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut params = vec![];
    if let Some(l) = limit { params.push(format!("limit={l}")); }
    if let Some(o) = offset { params.push(format!("offset={o}")); }
    if let Some(s) = search { params.push(format!("search={}", urlenc(s))); }
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

#[allow(clippy::too_many_arguments)]
pub async fn create(
    title: &str,
    description: Option<&str>,
    rationale: Option<&str>,
    status: Option<&str>,
    decision_maker: Option<&str>,
    decision_type: Option<&str>,
    priority: Option<&str>,
    data_date: Option<&str>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut body = json!({ "decision_title": title });
    if let Some(d) = description { body["description"] = Value::String(d.to_string()); }
    if let Some(r) = rationale { body["rationale"] = Value::String(r.to_string()); }
    if let Some(s) = status { body["decision_status"] = Value::String(s.to_string()); }
    if let Some(m) = decision_maker { body["decision_maker_email"] = Value::String(m.to_string()); }
    if let Some(t) = decision_type { body["decision_type"] = Value::String(t.to_string()); }
    if let Some(p) = priority { body["priority"] = Value::String(p.to_string()); }
    if let Some(dd) = data_date { body["data_date"] = Value::String(dd.to_string()); }
    let resp = client.post(BASE, &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn version_set_date(version_id: &str, data_date: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let body = json!({ "data_date": data_date });
    let resp = client
        .post(&format!("{BASE}/versions/{version_id}/set-date"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}

pub async fn version_delete(version_id: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let resp = client
        .post(&format!("{BASE}/versions/{version_id}/delete"), &json!({}))
        .await?;
    output::print_success(resp);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    id: &str,
    title: Option<&str>,
    description: Option<&str>,
    rationale: Option<&str>,
    status: Option<&str>,
    decision_maker: Option<&str>,
    decision_type: Option<&str>,
    priority: Option<&str>,
    data_date: Option<&str>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut body = json!({});
    if let Some(t) = title { body["decision_title"] = Value::String(t.to_string()); }
    if let Some(d) = description { body["description"] = Value::String(d.to_string()); }
    if let Some(r) = rationale { body["rationale"] = Value::String(r.to_string()); }
    if let Some(s) = status { body["decision_status"] = Value::String(s.to_string()); }
    if let Some(m) = decision_maker { body["decision_maker_email"] = Value::String(m.to_string()); }
    if let Some(dt) = decision_type { body["decision_type"] = Value::String(dt.to_string()); }
    if let Some(p) = priority { body["priority"] = Value::String(p.to_string()); }
    if let Some(dd) = data_date { body["data_date"] = Value::String(dd.to_string()); }
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
    let body = json!({ "target_decision_id": target_id });
    let resp = client.post(&format!("{BASE}/{source_id}/merge"), &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn link(
    id: &str,
    sub_topic: Option<&str>,
    task: Option<&str>,
    intent: Option<&str>,
    rel_type: Option<&str>,
) -> Result<(), CliError> {
    let body = build_edge_body(sub_topic, task, intent, rel_type)?;
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/{id}/edges/add"), &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn unlink(
    id: &str,
    sub_topic: Option<&str>,
    task: Option<&str>,
    intent: Option<&str>,
    rel_type: Option<&str>,
) -> Result<(), CliError> {
    let body = build_edge_body(sub_topic, task, intent, rel_type)?;
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/{id}/edges/remove"), &body).await?;
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

pub async fn normalize_edges(
    decision_id: Option<&str>,
    sub_topic_prefer: &[String],
    task_prefer: &[String],
    dry_run: bool,
) -> Result<(), CliError> {
    let mut body = json!({ "dry_run": dry_run });
    if let Some(d) = decision_id {
        body["decision_id"] = Value::String(d.to_string());
    }
    if !sub_topic_prefer.is_empty() {
        body["sub_topic_prefer"] = Value::Array(
            sub_topic_prefer
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        );
    }
    if !task_prefer.is_empty() {
        body["task_prefer"] = Value::Array(
            task_prefer
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        );
    }
    let client = ApiClient::new()?;
    let resp = client
        .post(&format!("{BASE}/normalize-edges"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}

fn build_edge_body(
    sub_topic: Option<&str>,
    task: Option<&str>,
    intent: Option<&str>,
    rel_type: Option<&str>,
) -> Result<Value, CliError> {
    let provided = [sub_topic.is_some(), task.is_some(), intent.is_some()]
        .iter()
        .filter(|x| **x)
        .count();
    if provided != 1 {
        return Err(CliError::BadInput(
            "Specify exactly one of --sub-topic, --task, --intent.".into(),
        ));
    }
    let (kind, target_id) = if let Some(v) = sub_topic {
        ("sub_topic", v)
    } else if let Some(v) = task {
        ("task", v)
    } else {
        ("intent", intent.unwrap())
    };
    let mut body = json!({ "kind": kind, "target_id": target_id });
    if let Some(rt) = rel_type {
        body["rel_type"] = Value::String(rt.to_string());
    }
    Ok(body)
}

fn urlenc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
