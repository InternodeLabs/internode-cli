use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/statuses";

pub async fn list(team: Option<&str>) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let qs = match team {
        Some(t) => format!("?oiteam_id={}", urlenc(t)),
        None => String::new(),
    };
    let resp = client.get(&format!("{BASE}{qs}")).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn create(
    team: &str,
    name: &str,
    description: Option<&str>,
    category: Option<&str>,
    created_date: Option<&str>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut body = json!({ "oiteam_id": team, "name": name });
    if let Some(d) = description { body["description"] = Value::String(d.to_string()); }
    if let Some(c) = category { body["category"] = Value::String(c.to_string()); }
    if let Some(cd) = created_date { body["created_date"] = Value::String(cd.to_string()); }
    let resp = client.post(BASE, &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn set_created_date(id: &str, created_date: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let body = json!({ "created_date": created_date });
    let resp = client
        .post(&format!("{BASE}/{id}/set-created-date"), &body)
        .await?;
    output::print_success(resp);
    Ok(())
}

fn urlenc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
