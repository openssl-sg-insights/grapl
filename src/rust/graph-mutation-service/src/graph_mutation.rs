use std::sync::Arc;

use rust_proto::{
    graplinc::grapl::{
        api::{
            graph::v1beta1::Property,
            graph_mutation::v1beta1::{
                messages::{
                    CreateEdgeRequest,
                    CreateEdgeResponse,
                    CreateNodeRequest,
                    CreateNodeResponse,
                    MutationRedundancy,
                    SetNodePropertyRequest,
                    SetNodePropertyResponse,
                },
                server::GraphMutationApi,
            },
            uid_allocator::v1beta1::client::UidAllocatorServiceClientError,
        },
        common::v1beta1::types::{
            EdgeName,
            NodeType,
            PropertyName,
            Uid,
        },
    },
    protocol::status::Status,
};
use scylla::{
    query::Query,
    CachingSession,
};
use uid_allocator::client::CachingUidAllocatorServiceClient as UidAllocatorClient;

use crate::{
    reverse_edge_resolver::{
        ReverseEdgeResolver,
        ReverseEdgeResolverError,
    },
    table_names::{
        IMM_I_64_TABLE_NAME,
        IMM_STRING_TABLE_NAME,
        IMM_U_64_TABLE_NAME,
        MAX_I_64_TABLE_NAME,
        MAX_U_64_TABLE_NAME,
        MIN_I_64_TABLE_NAME,
        MIN_U_64_TABLE_NAME,
    },
    write_dropper::{
        Write,
        WriteDropper,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationManagerError {
    #[error("UidAllocatorServiceClientError {0}")]
    UidAllocatorServiceClientError(#[from] UidAllocatorServiceClientError),
    #[error("Allocated Zero Uid")]
    ZeroUid,
    #[error("Scylla Error {0}")]
    ScyllaError(#[from] scylla::transport::errors::QueryError),
    #[error("ReverseEdgeResolverError {0}")]
    ReverseEdgeResolverError(#[from] ReverseEdgeResolverError),
}

impl From<GraphMutationManagerError> for Status {
    fn from(e: GraphMutationManagerError) -> Self {
        match e {
            GraphMutationManagerError::UidAllocatorServiceClientError(
                UidAllocatorServiceClientError::SerDeError(e),
            ) => Status::internal(format!(
                "Failed to deserialize response from UidAllocator {:?}",
                e
            )),
            GraphMutationManagerError::UidAllocatorServiceClientError(
                UidAllocatorServiceClientError::Status(e),
            ) => e,
            GraphMutationManagerError::UidAllocatorServiceClientError(
                UidAllocatorServiceClientError::ConnectError(e),
            ) => Status::internal(format!("Failed to connect to UidAllocator {:?}", e)),
            GraphMutationManagerError::ZeroUid => Status::failed_precondition("Allocated Zero Uid"),
            e => Status::internal(e.to_string()),
        }
    }
}

pub struct GraphMutationManager {
    scylla_client: Arc<CachingSession>,
    uid_allocator_client: UidAllocatorClient,
    reverse_edge_resolver: ReverseEdgeResolver,
    write_dropper: WriteDropper,
}

impl GraphMutationManager {
    pub fn new(
        scylla_client: Arc<CachingSession>,
        uid_allocator_client: UidAllocatorClient,
        reverse_edge_resolver: ReverseEdgeResolver,
        max_write_drop_size: usize,
    ) -> Self {
        Self {
            scylla_client,
            uid_allocator_client,
            reverse_edge_resolver,
            write_dropper: WriteDropper::new(max_write_drop_size),
        }
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_u64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: u64,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_max_u64(
                tenant_keyspace,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || async move {
                    let property_value = property_value as i64;
                    let tenant_urn = tenant_keyspace.urn();
                    let mut query = Query::new(format!(r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.{MAX_U_64_TABLE_NAME} (uid, node_type, property_name, property_value)
                        VALUES (?, ?, ?, ?)
                    "));
                    query.set_timestamp(Some(property_value));

                    self.scylla_client
                        .execute(
                            query,
                            &(
                                uid.as_i64(),
                                node_type.value,
                                property_name.value,
                                property_value,
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_min_u64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: u64,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_min_u64(
                tenant_keyspace,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || async move {
                    let property_value = property_value as i64;
                    let tenant_urn = tenant_keyspace.urn();
                    let mut query = Query::new(format!(r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.{MIN_U_64_TABLE_NAME} (uid, node_type, property_name, property_value)
                        VALUES (?, ?, ?, ?)
                    "));

                    query.set_timestamp(Some(-property_value));

                    self.scylla_client
                        .execute(
                            query,
                            &(
                                uid.as_i64(),
                                node_type.value,
                                property_name.value,
                                property_value,
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_immutable_u64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: u64,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_imm_u64(
                tenant_keyspace,
                node_type.clone(),
                property_name.clone(),
                || async move {
                    let property_value = property_value as i64;
                    // todo: We should only prepare statements once

                    let tenant_urn = tenant_keyspace.urn();
                    let query = Query::new(format!(r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.{IMM_U_64_TABLE_NAME} (uid, node_type, property_name, property_value)
                        VALUES (?, ?, ?, ?)
                    "));

                    self.scylla_client
                        .execute(
                            query,
                            &(
                                uid.as_i64(),
                                node_type.value,
                                property_name.value,
                                property_value,
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_i64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: i64,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_max_i64(
                tenant_keyspace,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || async move {
                    let tenant_urn = tenant_keyspace.urn();
                    let mut query = Query::new(format!(r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.{MAX_I_64_TABLE_NAME} (uid, node_type, property_name, property_value)
                        VALUES (?, ?, ?, ?)
                    "));
                    query.set_timestamp(Some(property_value));

                    self.scylla_client
                        .execute(
                            query,
                            &(
                                uid.as_i64(),
                                node_type.value,
                                property_name.value,
                                property_value,
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_min_i64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: i64,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_min_i64(
                tenant_keyspace,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || async move {
                    let tenant_urn = tenant_keyspace.urn();
                    let mut query = Query::new(format!(r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.{MIN_I_64_TABLE_NAME} (uid, node_type, property_name, property_value)
                        VALUES (?, ?, ?, ?)
                    "));

                    query.set_timestamp(Some(-property_value));

                    self.scylla_client
                        .execute(
                            query,
                            &(
                                uid.as_i64(),
                                node_type.value,
                                property_name.value,
                                property_value,
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_immutable_i64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: i64,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_imm_i64(
                tenant_keyspace,
                node_type.clone(),
                property_name.clone(),
                || async move {
                    let tenant_urn = tenant_keyspace.urn();
                    let query = Query::new(format!(r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.{IMM_I_64_TABLE_NAME} (uid, node_type, property_name, property_value)
                        VALUES (?, ?, ?, ?)
                    "));

                    self.scylla_client
                        .execute(
                            query,
                            &(
                                uid.as_i64(),
                                node_type.value,
                                property_name.value,
                                property_value,
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn set_node_type(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_node_type(tenant_keyspace, uid, || async move {
                let tenant_urn = tenant_keyspace.urn();
                let query = Query::new(format!(
                    r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.node_type (uid, node_type)
                        VALUES (?, ?)
                    "
                ));

                self.scylla_client
                    .execute(query, &(uid.as_i64(), node_type.value))
                    .await?;
                Ok(())
            })
            .await
    }

    async fn upsert_immutable_string(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: String,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_imm_string(
                tenant_keyspace,
                node_type.clone(),
                property_name.clone(),
                || async move {

                    let tenant_urn = tenant_keyspace.urn();
                    let query = Query::new(format!(r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.{IMM_STRING_TABLE_NAME} (uid, node_type, property_name, property_value)
                        VALUES (?, ?, ?, ?)
                    "));

                    self.scylla_client
                        .execute(
                            query,
                            &(
                                uid.as_i64(),
                                node_type.value,
                                property_name.value,
                                property_value,
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_edges(
        &self,
        tenant_keyspace: uuid::Uuid,
        from_uid: Uid,
        to_uid: Uid,
        f_edge_name: EdgeName,
        r_edge_name: EdgeName,
    ) -> Result<Write, GraphMutationManagerError> {
        self.write_dropper
            .check_edges(
                tenant_keyspace,
                from_uid,
                to_uid,
                f_edge_name,
                r_edge_name,
                |f_edge_name, r_edge_name| async move {
                    // todo: Batch statements are currently not supported by the Scylla rust client
                    //       https://github.com/scylladb/scylla-rust-driver/issues/469
                    let tenant_urn = tenant_keyspace.urn();

                    let f_statement = format!(
                        r"
                        INSERT INTO tenant_keyspace_{tenant_urn}.edges (
                            source_uid,
                            f_edge_name,
                            r_edge_name,
                            destination_uid,
                        )
                        VALUES (?, ?, ?, ?)
                        ",
                    );
                    let r_statement = f_statement.clone();

                    let mut batch: scylla::batch::Batch = Default::default();
                    batch.statements.reserve(2);
                    batch.append_statement(Query::from(f_statement));
                    batch.append_statement(Query::from(r_statement));
                    batch.set_is_idempotent(true);

                    self.scylla_client
                        .session
                        .batch(
                            &batch,
                            (
                                (
                                    from_uid.as_i64(),
                                    &f_edge_name.value,
                                    &r_edge_name.value,
                                    to_uid.as_i64(),
                                ),
                                (
                                    to_uid.as_i64(),
                                    &r_edge_name.value,
                                    &f_edge_name.value,
                                    from_uid.as_i64(),
                                ),
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }
}

#[async_trait::async_trait]
impl GraphMutationApi for GraphMutationManager {
    type Error = GraphMutationManagerError;

    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn create_node(
        &self,
        request: CreateNodeRequest,
    ) -> Result<CreateNodeResponse, Self::Error> {
        let uid = self
            .uid_allocator_client
            .allocate_id(request.tenant_id)
            .await?;

        self.set_node_type(request.tenant_id, uid, request.node_type)
            .await?;

        Ok(CreateNodeResponse { uid })
    }

    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    async fn set_node_property(
        &self,
        request: SetNodePropertyRequest,
    ) -> Result<SetNodePropertyResponse, Self::Error> {
        let SetNodePropertyRequest {
            tenant_id,
            uid,
            node_type,
            property_name,
            property,
        } = request;
        let write = match property.property {
            Property::IncrementOnlyUintProp(property) => {
                self.upsert_max_u64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?
            }
            Property::DecrementOnlyUintProp(property) => {
                self.upsert_min_u64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?
            }
            Property::ImmutableUintProp(property) => {
                self.upsert_immutable_u64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?
            }
            Property::IncrementOnlyIntProp(property) => {
                self.upsert_max_i64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?
            }
            Property::DecrementOnlyIntProp(property) => {
                self.upsert_min_i64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?
            }
            Property::ImmutableIntProp(property) => {
                self.upsert_immutable_i64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?
            }
            Property::ImmutableStrProp(property) => {
                self.upsert_immutable_string(
                    tenant_id,
                    uid,
                    node_type,
                    property_name,
                    property.prop,
                )
                .await?
            }
        };

        let mutation_redundancy = match write {
            Write::Dropped => MutationRedundancy::True,
            Write::Issued => MutationRedundancy::Maybe,
        };
        Ok(SetNodePropertyResponse {
            mutation_redundancy,
        })
    }

    /// CreateEdge will create an edge with the name edge_name between the nodes
    /// that have the given uids. It will also create the reverse edge.
    async fn create_edge(
        &self,
        request: CreateEdgeRequest,
    ) -> Result<CreateEdgeResponse, Self::Error> {
        let CreateEdgeRequest {
            edge_name,
            tenant_id,
            from_uid,
            to_uid,
            source_node_type,
        } = request;

        let reverse_edge_name = self
            .reverse_edge_resolver
            .resolve_reverse_edge(tenant_id, source_node_type.clone(), edge_name.clone())
            .await?;

        let write = self
            .upsert_edges(tenant_id, from_uid, to_uid, edge_name, reverse_edge_name)
            .await?;

        let mutation_redundancy = match write {
            Write::Dropped => MutationRedundancy::True,
            Write::Issued => MutationRedundancy::Maybe,
        };

        Ok(CreateEdgeResponse {
            mutation_redundancy,
        })
    }
}