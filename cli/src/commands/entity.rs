use serde_json::{json, Value};

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

pub async fn get(ids: Vec<String>, include_deleted: bool) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let body = json!({ "entity_ids": ids, "include_deleted": include_deleted });
    let resp = client.post("/internode-tools/cli/oi/entities", &body).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn list_deleted(
    labels: Option<&str>,
    search: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut params: Vec<String> = vec![];
    if let Some(l) = labels {
        params.push(format!("labels={}", urlenc(l)));
    }
    if let Some(s) = search {
        params.push(format!("search={}", urlenc(s)));
    }
    if let Some(l) = limit {
        params.push(format!("limit={l}"));
    }
    if let Some(o) = offset {
        params.push(format!("offset={o}"));
    }
    let qs = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    let resp = client
        .get(&format!("/internode-tools/cli/oi/entities/deleted{qs}"))
        .await?;
    output::print_success(resp);
    Ok(())
}

pub async fn restore(id: &str, label: &str) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let body: Value = json!({ "label": label });
    let resp = client
        .post(
            &format!("/internode-tools/cli/oi/entities/{id}/restore"),
            &body,
        )
        .await?;
    output::print_success(resp);
    Ok(())
}

fn urlenc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
