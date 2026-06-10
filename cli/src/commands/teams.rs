use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/teams";

pub async fn list() -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let resp = client.get(BASE).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn create(
    name: &str,
    key: Option<&str>,
    description: Option<&str>,
    created_date: Option<&str>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut body = json!({ "name": name });
    if let Some(k) = key { body["key"] = Value::String(k.to_string()); }
    if let Some(d) = description { body["description"] = Value::String(d.to_string()); }
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
