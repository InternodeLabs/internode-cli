use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/sub-topics";

pub async fn list(
    type_filter: Option<&str>,
    topic: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut params = vec![];
    if let Some(l) = limit { params.push(format!("limit={l}")); }
    if let Some(o) = offset { params.push(format!("offset={o}")); }
    if let Some(t) = type_filter { params.push(format!("type={}", urlenc(t))); }
    if let Some(t) = topic { params.push(format!("topic_id={}", urlenc(t))); }
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

pub async fn move_to(id: &str, target_topic_id: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let body = json!({ "target_topic_id": target_topic_id });
    let resp = client.post(&format!("{BASE}/{id}/move"), &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn archive(id: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let resp = client.post(&format!("{BASE}/{id}/archive"), &json!({})).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn update(
    id: &str,
    conclusion: Option<&str>,
    conclusion_type: Option<&str>,
    primary_contributor: Option<&str>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut body = json!({});
    if let Some(c) = conclusion {
        body["topic_conclusion"] = Value::String(c.to_string());
    }
    if let Some(t) = conclusion_type {
        body["topic_conclusion_type"] = Value::String(t.to_string());
    }
    if let Some(e) = primary_contributor {
        body["primary_contributor_email"] = Value::String(e.to_string());
    }
    let resp = client.patch(&format!("{BASE}/{id}"), &body).await?;
    output::print_success(resp);
    Ok(())
}

fn urlenc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
