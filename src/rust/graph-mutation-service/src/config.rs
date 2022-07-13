use std::net::SocketAddr;

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphDbConfig {
    #[clap(long, env)]
    /// The address of the graph database
    pub graph_db_addresses: Vec<SocketAddr>,
    #[clap(env)]
    /// The username for the graph database
    pub graph_db_auth_username: String,
    #[clap(env)]
    /// The password for the graph database
    pub graph_db_auth_password: String,
}

#[derive(clap::Parser, Debug, Clone)]
pub struct UidAllocatorClientConfig {
    #[clap(env)]
    /// The address to connect to for the uid allocator
    pub uid_allocator_address: String,
}

#[derive(clap::Parser, Debug, Clone)]
pub struct SchemaManagerClientConfig {
    #[clap(env)]
    /// The address to connect to for the schema manager
    pub schema_manager_address: String,
}

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphMutationServiceConfig {
    #[clap(env)]
    /// The address to bind the graph mutation service to
    pub graph_mutation_service_bind_address: SocketAddr,

    #[clap(flatten)]
    pub uid_allocator_client_config: UidAllocatorClientConfig,

    #[clap(flatten)]
    pub schema_manager_client_config: SchemaManagerClientConfig,

    #[clap(flatten)]
    pub graph_db_config: GraphDbConfig,
}