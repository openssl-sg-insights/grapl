use futures::FutureExt;
use thiserror::Error;
use tokio::time::error::Elapsed;
use tonic::Request;

use crate::protocol::{
    healthcheck::HealthcheckError,
    service_client::NamedService,
};
/// This module contains the gRPC client for the pipeline ingress API. We
/// encapsulate all the types generated by the protocol buffer compiler and
/// instead expose our own "sanitized" version of the API.
use crate::{
    graplinc::grapl::api::pipeline_ingress::v1beta1::{
        PublishRawLogRequest,
        PublishRawLogResponse,
    },
    protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::pipeline_ingress_service_client::PipelineIngressServiceClient as PipelineIngressServiceClientProto,
    SerDeError,
};
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("failed to connect {0}")]
    ConnectionError(#[from] tonic::transport::Error),

    #[error("healthcheck failed {0}")]
    HealtcheckFailed(#[from] HealthcheckError),

    #[error("timeout elapsed {0}")]
    TimeoutElapsed(#[from] Elapsed),
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PipelineIngressClientError {
    #[error("failed to serialize/deserialize {0}")]
    SerDeError(#[from] SerDeError),

    #[error("received unfavorable gRPC status {0}")]
    GrpcStatus(#[from] tonic::Status),
}

pub struct PipelineIngressClient {
    proto_client: PipelineIngressServiceClientProto<tonic::transport::Channel>,
}

impl PipelineIngressClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, ConfigurationError>
    where
        T: std::convert::TryInto<tonic::transport::Endpoint>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(PipelineIngressClient {
            proto_client: PipelineIngressServiceClientProto::connect(endpoint).await?,
        })
    }

    pub async fn publish_raw_log(
        &mut self,
        raw_log: PublishRawLogRequest,
    ) -> Result<PublishRawLogResponse, PipelineIngressClientError> {
        self.proto_client
            .publish_raw_log(Request::new(raw_log.into()))
            .map(
                |response| -> Result<PublishRawLogResponse, PipelineIngressClientError> {
                    let inner = response?.into_inner();
                    Ok(inner.try_into()?)
                },
            )
            .await
    }
}

impl NamedService for PipelineIngressClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.pipeline_ingress.v1beta1.PipelineIngressService";
}