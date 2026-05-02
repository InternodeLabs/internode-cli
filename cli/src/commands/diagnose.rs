use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/diagnostics";

async fn run(
    entity: &str,
    order_by: Option<&str>,
    top: Option<i64>,
    min_edges: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let mut params = vec![];
    if let Some(by) = order_by {
        params.push(format!("order_by={}", urlenc(by)));
    }
    if let Some(n) = top {
        params.push(format!("limit={n}"));
    }
    if let Some(m) = min_edges {
        params.push(format!("min_edges={m}"));
    }
    if let Some(o) = offset {
        params.push(format!("offset={o}"));
    }
    let qs = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    let resp = client.get(&format!("{BASE}/{entity}{qs}")).await?;
    output::print_success(resp);
    Ok(())
}

pub async fn decisions(
    by: Option<&str>,
    top: Option<i64>,
    min_edges: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    run("decisions", by, top, min_edges, offset).await
}

pub async fn topics(
    by: Option<&str>,
    top: Option<i64>,
    min_edges: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    run("topics", by, top, min_edges, offset).await
}

pub async fn subtopics(
    top: Option<i64>,
    min_edges: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    run("sub-topics", None, top, min_edges, offset).await
}

pub async fn intents(
    top: Option<i64>,
    min_edges: Option<i64>,
    offset: Option<i64>,
) -> Result<(), CliError> {
    run("intents", None, top, min_edges, offset).await
}

fn urlenc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
