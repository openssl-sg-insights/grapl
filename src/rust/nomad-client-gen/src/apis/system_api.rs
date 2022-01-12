/*
 * Nomad
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.4
 * Contact: support@hashicorp.com
 * Generated by: https://openapi-generator.tech
 */

use reqwest;

use super::{
    configuration,
    Error,
};
use crate::apis::ResponseContent;

/// struct for passing parameters to the method [`put_system_gc`]
#[derive(Clone, Debug, Default)]
pub struct PutSystemGcParams {
    /// Filters results based on the specified region.
    pub region: Option<String>,
    /// Filters results based on the specified namespace.
    pub namespace: Option<String>,
    /// A Nomad ACL token.
    pub x_nomad_token: Option<String>,
    /// Can be used to ensure operations are only run once.
    pub idempotency_token: Option<String>,
}

/// struct for passing parameters to the method [`put_system_reconcile_summaries`]
#[derive(Clone, Debug, Default)]
pub struct PutSystemReconcileSummariesParams {
    /// Filters results based on the specified region.
    pub region: Option<String>,
    /// Filters results based on the specified namespace.
    pub namespace: Option<String>,
    /// A Nomad ACL token.
    pub x_nomad_token: Option<String>,
    /// Can be used to ensure operations are only run once.
    pub idempotency_token: Option<String>,
}

/// struct for typed errors of method [`put_system_gc`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PutSystemGcError {
    Status400(),
    Status403(),
    Status405(),
    Status500(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`put_system_reconcile_summaries`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PutSystemReconcileSummariesError {
    Status400(),
    Status403(),
    Status405(),
    Status500(),
    UnknownValue(serde_json::Value),
}

pub async fn put_system_gc(
    configuration: &configuration::Configuration,
    params: PutSystemGcParams,
) -> Result<(), Error<PutSystemGcError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let region = params.region;
    let namespace = params.namespace;
    let x_nomad_token = params.x_nomad_token;
    let idempotency_token = params.idempotency_token;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/system/gc", local_var_configuration.base_path);
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = region {
        local_var_req_builder =
            local_var_req_builder.query(&[("region", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = namespace {
        local_var_req_builder =
            local_var_req_builder.query(&[("namespace", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = idempotency_token {
        local_var_req_builder =
            local_var_req_builder.query(&[("idempotency_token", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(local_var_param_value) = x_nomad_token {
        local_var_req_builder =
            local_var_req_builder.header("X-Nomad-Token", local_var_param_value.to_string());
    }
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("X-Nomad-Token", local_var_value);
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<PutSystemGcError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn put_system_reconcile_summaries(
    configuration: &configuration::Configuration,
    params: PutSystemReconcileSummariesParams,
) -> Result<(), Error<PutSystemReconcileSummariesError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let region = params.region;
    let namespace = params.namespace;
    let x_nomad_token = params.x_nomad_token;
    let idempotency_token = params.idempotency_token;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/system/reconcile/summaries",
        local_var_configuration.base_path
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = region {
        local_var_req_builder =
            local_var_req_builder.query(&[("region", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = namespace {
        local_var_req_builder =
            local_var_req_builder.query(&[("namespace", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = idempotency_token {
        local_var_req_builder =
            local_var_req_builder.query(&[("idempotency_token", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(local_var_param_value) = x_nomad_token {
        local_var_req_builder =
            local_var_req_builder.header("X-Nomad-Token", local_var_param_value.to_string());
    }
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("X-Nomad-Token", local_var_value);
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<PutSystemReconcileSummariesError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
