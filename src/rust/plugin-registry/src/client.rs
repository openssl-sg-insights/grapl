use rust_proto::plugin_registry::{
    plugin_registry_service_client::PluginRegistryServiceClient as _PluginRegistryServiceClient,
    CreatePluginRequest,
    CreatePluginRequestProto,
    CreatePluginResponse,
    DeployPluginRequest,
    DeployPluginRequestProto,
    DeployPluginResponse,
    GetAnalyzersForTenantRequest,
    GetAnalyzersForTenantRequestProto,
    GetAnalyzersForTenantResponse,
    GetGeneratorsForEventSourceRequest,
    GetGeneratorsForEventSourceRequestProto,
    GetGeneratorsForEventSourceResponse,
    GetPluginRequest,
    GetPluginRequestProto,
    GetPluginResponse,
    TearDownPluginRequest,
    TearDownPluginRequestProto,
    TearDownPluginResponse,
};
use tonic::codegen::{
    Body,
    StdError,
};

use crate::server::PluginRegistryServiceError;

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceClientError {}

pub struct PluginRegistryServiceClient<T> {
    inner: _PluginRegistryServiceClient<T>,
}

impl<T> PluginRegistryServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(inner: _PluginRegistryServiceClient<T>) -> Self {
        Self { inner }
    }

    /// create a new plugin
    pub async fn create_plugin(
        &mut self,
        request: CreatePluginRequest,
    ) -> Result<CreatePluginResponse, PluginRegistryServiceError> {
        self.inner
            .create_plugin(CreatePluginRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
    /// retrieve the plugin corresponding to the given plugin_id
    pub async fn get_plugin(
        &mut self,
        request: GetPluginRequest,
    ) -> Result<GetPluginResponse, PluginRegistryServiceError> {
        self.inner
            .get_plugin(GetPluginRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
    /// turn on a particular plugin's code
    pub async fn deploy_plugin(
        &mut self,
        request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, PluginRegistryServiceError> {
        self.inner
            .deploy_plugin(DeployPluginRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
    /// turn off a particular plugin's code
    pub async fn tear_down_plugin(
        &mut self,
        request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, PluginRegistryServiceError> {
        self.inner
            .tear_down_plugin(TearDownPluginRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
    /// Given information about an event source, return all generators that handle that event source
    pub async fn get_generator_for_event_source(
        &mut self,
        request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, PluginRegistryServiceError> {
        self.inner
            .get_generator_for_event_source(GetGeneratorsForEventSourceRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
    /// Given information about a tenant, return all analyzers for that tenant
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, PluginRegistryServiceError> {
        self.inner
            .get_analyzers_for_tenant(GetAnalyzersForTenantRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
}
