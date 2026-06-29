use crate::client::ApiClient;
use crate::error::CliError;
use crate::output;

const BASE: &str = "/internode-tools/cli/oi/graph";

/// Export the owner's distilled OI graph (nodes + edges) for a local NetworkX
/// MultiDiGraph mirror. Raw external work-item mirror nodes are excluded and
/// version chains are collapsed to their live head server-side; everything else
/// is discovered dynamically.
pub async fn export(include_deleted: bool) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let qs = if include_deleted {
        "?include_deleted=true"
    } else {
        ""
    };
    let resp = client.get(&format!("{BASE}/export{qs}")).await?;
    output::print_success(resp);
    Ok(())
}

/// Derive the schema (labels, relationship types, endpoint pairings, property
/// types) of the owner's exported OI graph, purely from the live data.
pub async fn schema(include_deleted: bool) -> Result<(), CliError> {
    let client = ApiClient::new()?;
    let qs = if include_deleted {
        "?include_deleted=true"
    } else {
        ""
    };
    let resp = client.get(&format!("{BASE}/schema{qs}")).await?;
    output::print_success(resp);
    Ok(())
}
