use std::io::Read;
use std::sync::atomic::Ordering;
use tonic::{Status};
use tonic::transport::Server;
use rust_proto::plugin_bootstrap::{GetBootstrapInfoRequestProto, GetBootstrapInfoResponse, GetBootstrapInfoResponseProto, PluginBootstrapService, PluginBootstrapServiceServer};
use crate::PluginBootstrapServiceConfig;

#[derive(Debug, thiserror::Error)]
pub enum PluginBootstrapperError {
    #[error("IoError")]
    IoError(#[from] std::io::Error)
}

pub struct PluginBootstrapper {
    // todo: https://docs.rs/rustls/latest/rustls/struct.Certificate.html
    pub certificate: Vec<u8>,
    pub plugin_binary: Vec<u8>,
    pub ctr: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl PluginBootstrapper {
    pub fn new(certificate: Vec<u8>, plugin_binary: Vec<u8>) -> Self {
        Self {
            certificate,
            plugin_binary,
            ctr: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn load(certificate_path: &std::path::Path, plugin_binary_path: &std::path::Path) -> Result<Self, PluginBootstrapperError> {
        let certificate_file = std::fs::File::open(certificate_path)?;
        let plugin_binary_file = std::fs::File::open(plugin_binary_path)?;

        let mut certificate = Vec::with_capacity(512);
        let mut plugin_binary = Vec::with_capacity(128_000_000);

        let mut reader = std::io::BufReader::new(certificate_file);
        reader.read_to_end(&mut certificate)?;
        let mut reader = std::io::BufReader::new(plugin_binary_file);
        reader.read_to_end(&mut plugin_binary)?;

        Ok(PluginBootstrapper::new(certificate, plugin_binary))
    }

    async fn get_bootstrap_info(
        &self,
    ) -> GetBootstrapInfoResponse
    {
        let ctr = self.ctr.fetch_add(1, Ordering::SeqCst);
        if ctr != 0 {
            tracing::warn!(
                message="Bootstrap information has been requested more than once.",
                count=%ctr,
            );
        }
        GetBootstrapInfoResponse {
            plugin_binary: self.plugin_binary.clone(),
            certificate: self.certificate.clone(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn serve(
        self,
        service_config: PluginBootstrapServiceConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<PluginBootstrapServiceServer<PluginBootstrapper>>()
            .await;

        let addr = service_config.plugin_registry_bind_address;
        tracing::info!(
            message="Starting PluginBootstrap",
            addr=?addr,
        );

        Server::builder()
            .trace_fn(|request| {
                tracing::info_span!(
                "PluginBootstrap",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
            })
            .add_service(health_service)
            .add_service(PluginBootstrapServiceServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }

}

#[tonic::async_trait]
impl PluginBootstrapService for PluginBootstrapper {
    #[tracing::instrument(skip(self))]
    async fn get_bootstrap_info(
        &self,
        _request: tonic::Request<GetBootstrapInfoRequestProto>,
    ) -> Result<tonic::Response<GetBootstrapInfoResponseProto>, Status>
    {
        let response = self.get_bootstrap_info().await;
        Ok(tonic::Response::new(
            response.into()
        ))
    }
}