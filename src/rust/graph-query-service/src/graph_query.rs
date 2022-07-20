use futures::future::join_all;
pub use rust_proto::graplinc::grapl::api::graph_query_service::v1beta1::messages::StringCmp;
use rust_proto::graplinc::grapl::{
    api::graph_query_service::v1beta1::messages::{
        GraphQuery,
        GraphView,
    },
    common::v1beta1::types::Uid,
};

use crate::{
    node_query::fetch_node_with_edges,
    property_query::PropertyQueryExecutor,
    short_circuit::ShortCircuit,
    visited::Visited,
};

#[tracing::instrument(skip(graph_query, property_query_executor))]
pub async fn query_graph(
    graph_query: &GraphQuery,
    uid: Uid,
    tenant_id: uuid::Uuid,
    property_query_executor: PropertyQueryExecutor,
) -> Result<Option<(GraphView, Uid)>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut query_handles = Vec::with_capacity(graph_query.node_property_queries.len());
    let x_query_short_circuiter = ShortCircuit::new();
    // We should add a way for different queries to short circuit each other
    for node_query in graph_query.node_property_queries.values() {
        let property_query_executor = property_query_executor.clone();
        let node_query = node_query.clone();
        let x_query_short_circuiter = x_query_short_circuiter.clone();
        query_handles.push(async move {
            let visited = Visited::new();
            let mut root_query_uid = None;
            match fetch_node_with_edges(
                &node_query,
                &graph_query,
                uid,
                tenant_id,
                property_query_executor,
                visited,
                x_query_short_circuiter.clone(),
                &mut root_query_uid,
            )
            .await
            {
                Ok(Some(g)) => {
                    x_query_short_circuiter.set_short_circuit();
                    Ok(Some((g, root_query_uid)))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        });
    }
    // todo: We don't need to join_all, we can stop polling the other futures
    //       once one of them matches
    for graph in join_all(query_handles).await {
        match graph {
            Ok(Some((graph, Some(root_uid)))) => return Ok(Some((graph, root_uid))),
            Ok(Some((_, None))) => {
                tracing::error!(
                    message = "Graph query matched without finding root_uid. This is a bug.",
                )
            }
            Ok(None) => continue,
            Err(e) => {
                tracing::error!(
                    message="Graph query failed",
                    error=?e,
                )
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{
            self,
            BufRead,
        },
        path::Path,
        sync::Arc,
    };

    use maplit::hashmap;
    use scylla::{
        batch::Consistency,
        CachingSession,
        SessionBuilder,
    };

    use super::*;
    use crate::node_query::NodeQuery;

    async fn insert_string_ix(
        session: Arc<CachingSession>,
        tenant_id: &uuid::Uuid,
        node_type: &str,
        uid: i64,
        populated_field: String,
        value: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        session.session
            .query(
                format!(r#"
                    INSERT INTO tenant_keyspace_{}.immutable_string_index (node_type, uid, populated_field, value)
                      VALUES (?, ?, ?, ?)"#, tenant_id.to_simple()),
                (node_type, uid, populated_field, value),
            )
            .await?;

        Ok(())
    }

    async fn insert_edge(
        session: Arc<CachingSession>,
        tenant_id: &uuid::Uuid,
        source_uid: i64,
        f_edge_name: String,
        r_edge_name: String,
        destination_uid: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        session
            .session
            .query(
                format!(
                    r#"
            INSERT INTO tenant_keyspace_{}.edges (
                source_uid,
                f_edge_name,
                r_edge_name,
                destination_uid
            )
            VALUES (?, ?, ?, ?)"#,
                    tenant_id.to_simple()
                ),
                (
                    source_uid.clone(),
                    f_edge_name.clone(),
                    r_edge_name.clone(),
                    destination_uid.clone(),
                ),
            )
            .await?;

        session
            .session
            .query(
                format!(
                    r#"INSERT INTO tenant_keyspace_{}.edges (
                source_uid,
                f_edge_name,
                r_edge_name,
                destination_uid
            )
            VALUES (?, ?, ?, ?)"#,
                    tenant_id.to_simple()
                ),
                (destination_uid, r_edge_name, f_edge_name, source_uid),
            )
            .await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn smoke_test() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let uris = &["localhost"][..];
        println!("connecting to {uris:?}");
        let session: Session = SessionBuilder::new()
            .known_nodes(&uris[..])
            .default_consistency(Consistency::One)
            //     .user(
            //     "scylla", "cS0h4mLIWxaEB5D",
            // )
            .build()
            .await?;
        let session: CachingSession = CachingSession::from(session, 100_000);
        let session = Arc::new(session);

        let tenant_id = uuid::Uuid::new_v4();

        session.session.query(
            format!("CREATE KEYSPACE IF NOT EXISTS tenant_keyspace_{} WITH REPLICATION = {{'class' : 'SimpleStrategy', 'replication_factor' : 3}}", tenant_id.to_simple()),
            &[]
        ).await?;

        session
            .session
            .query(
                format!(
                    "DROP TABLE IF EXISTS tenant_keyspace_{}.immutable_string_index",
                    tenant_id.to_simple()
                ),
                &[],
            )
            .await?;
        // return Ok(());
        println!("created keyspace");

        session
            .session
            .query(
                format!(
                    "
                CREATE TABLE IF NOT EXISTS tenant_keyspace_{}.immutable_string_index (
                       uid bigint,
                       node_type text,
                       populated_field text,
                       value text,
                       PRIMARY KEY ((node_type, uid, populated_field))
                )
                WITH compression = {{
                    'sstable_compression': 'LZ4Compressor',
                    'chunk_length_in_kb': 64
                }};
                ",
                    tenant_id.to_simple()
                ),
                &[],
            )
            .await?;

        println!("created imm");

        // TODO: Create a secondary index on the edge table
        //       and stop inserting both edges
        session
            .session
            .query(
                format!(
                    "
                CREATE TABLE IF NOT EXISTS tenant_keyspace_{}.edges (
                       source_uid bigint,
                       f_edge_name text,
                       r_edge_name text,
                       destination_uid bigint,
                       PRIMARY KEY((source_uid, f_edge_name, r_edge_name), destination_uid)
                )
                WITH compression = {{
                    'sstable_compression': 'LZ4Compressor',
                    'chunk_length_in_kb': 64
                }};
                ",
                    tenant_id.to_simple()
                ),
                &[],
            )
            .await?;
        println!("Created edge");
        let uid = 1000;

        insert_string_ix(
            session.clone(),
            &tenant_id,
            "Process",
            uid,
            "process_name".to_string(),
            "chrome.exe".to_string(),
        )
        .await?;
        println!("inserted string ix");

        insert_string_ix(
            session.clone(),
            &tenant_id,
            "Process",
            uid,
            "arguments".to_string(),
            "-a -f -b --boop=bop".to_string(),
        )
        .await?;

        insert_string_ix(
            session.clone(),
            &tenant_id,
            "Process",
            uid + 123,
            "process_name".to_string(),
            "evil.exe".to_string(),
        )
        .await?;

        insert_string_ix(
            session.clone(),
            &tenant_id,
            "File",
            uid + 234,
            "file_path".to_string(),
            "some/sorta/path".to_string(),
        )
        .await?;

        println!("inserted string ix");

        insert_edge(
            session.clone(),
            &tenant_id,
            uid,
            "children".into(),
            "parent".into(),
            uid + 123,
        )
        .await?;

        insert_edge(
            session.clone(),
            &tenant_id,
            uid + 123,
            "read_files".into(),
            "read_by_processes".into(),
            uid + 234,
        )
        .await?;

        insert_edge(
            session.clone(),
            &tenant_id,
            uid,
            "read_files".into(),
            "read_by_processes".into(),
            uid + 234,
        )
        .await?;

        let shared_node = NodePropertiesQuery::new("File".try_into()?);

        let query = NodeQuery::root("Process".try_into()?)
            .with_string_comparisons(
                "process_name".try_into()?,
                [StringCmp::eq("chrome.exe", false)],
            )
            .with_shared_edge(
                "read_files".try_into()?,
                "read_by_processes".try_into()?,
                shared_node.clone(),
                |neighbor| {
                    neighbor.with_string_comparisons(
                        "file_path".try_into().expect("invalid name"),
                        [StringCmp::eq("idk", true)],
                    );
                },
            )
            .with_edge_to(
                "children".try_into()?,
                "parent".try_into()?,
                "Process".try_into()?,
                |neighbor| {
                    neighbor
                        .with_string_comparisons(
                            "process_name".try_into().expect("invalid name"),
                            [StringCmp::eq("chrome.exe", true)],
                        )
                        .with_shared_edge(
                            "read_files".try_into().expect("invalid"),
                            "read_by_processes".try_into().expect("invalid"),
                            shared_node,
                            |neighbor| {
                                neighbor.with_string_comparisons(
                                    "file_path".try_into().expect("invalid name"),
                                    [StringCmp::eq("idk", true)],
                                );
                            },
                        );
                },
            )
            .build();

        let property_query_executor = PropertyQueryExecutor::new(session);
        let response = query
            .query_graph(
                Uid::from_i64(uid + 123).unwrap(),
                tenant_id,
                property_query_executor,
            )
            .await?;
        if let Some(ref graph) = response {
            println!("node_count: {}", graph.get_nodes().len());
            for node in graph.get_nodes() {
                println!("node: {:?}", node);
            }
            let root_node = graph.find_node_by_query_id(query.root_query_id).unwrap();
            println!("----response----\n{root_node:#?}");

            for (edge_name, neighbor) in graph.edges.iter() {
                println!("edge_name: {:?} neighbor: {:?}", edge_name, neighbor);
            }
        } else {
            println!("no response");
        }
        Ok(())
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}
