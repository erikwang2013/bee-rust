// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Unique identifier for a vertex (node).
pub type VertexId = String;

/// Unique identifier for an edge (relationship).
pub type EdgeId = String;

/// Key-value properties attached to vertices and edges.
pub type Properties = serde_json::Map<String, serde_json::Value>;

/// Named parameters for parameterised graph queries.
pub type Params = serde_json::Map<String, serde_json::Value>;

/// A vertex (node) in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub id: VertexId,
    pub label: String,
    pub properties: Properties,
}

/// An edge (relationship) between two vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub label: String,
    pub from: VertexId,
    pub to: VertexId,
    pub properties: Properties,
}

/// High-level traversal specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Traversal {
    /// Starting vertex ID.
    pub start: VertexId,
    /// Edge label(s) to follow (empty = any).
    pub edge_labels: Vec<String>,
    /// Maximum depth / hops.
    pub max_depth: u32,
    /// Direction in which to traverse edges.
    pub direction: TraversalDirection,
}

/// Direction for graph traversals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraversalDirection {
    Outgoing,
    Incoming,
    Both,
}

/// A single path result from a traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    /// Ordered vertices visited along the path.
    pub vertices: Vec<Vertex>,
    /// Ordered edges traversed.
    pub edges: Vec<Edge>,
}

/// A generic result envelope for raw graph queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Column names returned by the query.
    pub columns: Vec<String>,
    /// Rows, each represented as a JSON value.
    pub rows: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur when interacting with a graph database.
#[derive(Error, Debug)]
pub enum GraphError {
    /// The requested vertex does not exist.
    #[error("vertex not found: {0}")]
    VertexNotFound(VertexId),

    /// The requested edge does not exist.
    #[error("edge not found: {0}")]
    EdgeNotFound(EdgeId),

    /// The connection to the graph backend failed.
    #[error("connection error: {0}")]
    ConnectionError(String),

    /// A query could not be executed.
    #[error("query error: {0}")]
    QueryError(String),
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A generic async graph database trait.
///
/// Backends such as Neo4j, NebulaGraph, or ArangoDB can implement this to
/// provide vertex/edge CRUD, traversals, and arbitrary query execution.
#[async_trait]
pub trait GraphDB: Send + Sync {
    /// Insert a new vertex.  The `id` field may be assigned by the caller
    /// or left empty for the backend to generate.
    async fn add_vertex(&self, vertex: Vertex) -> Result<Vertex, GraphError>;

    /// Return the vertex identified by `id`.
    async fn get_vertex(&self, id: &VertexId) -> Result<Option<Vertex>, GraphError>;

    /// Update properties of an existing vertex.  The provided `properties`
    /// are merged into (not fully replaced by) the existing ones.
    async fn update_vertex(
        &self,
        id: &VertexId,
        properties: Properties,
    ) -> Result<Vertex, GraphError>;

    /// Delete the vertex identified by `id` and all incident edges.
    async fn delete_vertex(&self, id: &VertexId) -> Result<(), GraphError>;

    /// Insert a new edge.
    async fn add_edge(&self, edge: Edge) -> Result<Edge, GraphError>;

    /// Traverse the graph starting from `traversal.start`.
    async fn traverse(&self, traversal: Traversal) -> Result<Vec<PathResult>, GraphError>;

    /// Execute an arbitrary backend-specific query with optional parameters.
    async fn query(&self, query: &str, params: Option<Params>) -> Result<QueryResult, GraphError>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct StubGraphDB {
        vertices: Mutex<HashMap<VertexId, Vertex>>,
        edges: Mutex<HashMap<EdgeId, Edge>>,
    }

    impl StubGraphDB {
        fn new() -> Self {
            Self { vertices: Mutex::new(HashMap::new()), edges: Mutex::new(HashMap::new()) }
        }
    }

    #[allow(clippy::needless_lifetimes)]
    #[async_trait]
    impl GraphDB for StubGraphDB {
        async fn add_vertex(&self, mut vertex: Vertex) -> Result<Vertex, GraphError> {
            let mut map = self.vertices.lock().unwrap();
            if vertex.id.is_empty() {
                vertex.id = format!("v{}", map.len());
            }
            map.insert(vertex.id.clone(), vertex.clone());
            Ok(vertex)
        }

        async fn get_vertex(&self, id: &VertexId) -> Result<Option<Vertex>, GraphError> {
            let map = self.vertices.lock().unwrap();
            Ok(map.get(id).cloned())
        }

        async fn update_vertex(
            &self,
            id: &VertexId,
            properties: Properties,
        ) -> Result<Vertex, GraphError> {
            let mut map = self.vertices.lock().unwrap();
            let vertex = map.get_mut(id).ok_or_else(|| GraphError::VertexNotFound(id.clone()))?;
            for (k, v) in properties {
                vertex.properties.insert(k, v);
            }
            Ok(vertex.clone())
        }

        async fn delete_vertex(&self, id: &VertexId) -> Result<(), GraphError> {
            let mut vmap = self.vertices.lock().unwrap();
            vmap.remove(id);
            let mut emap = self.edges.lock().unwrap();
            emap.retain(|_, e| e.from != *id && e.to != *id);
            Ok(())
        }

        async fn add_edge(&self, mut edge: Edge) -> Result<Edge, GraphError> {
            // Validate endpoints exist
            let vmap = self.vertices.lock().unwrap();
            if !vmap.contains_key(&edge.from) {
                return Err(GraphError::VertexNotFound(edge.from.clone()));
            }
            if !vmap.contains_key(&edge.to) {
                return Err(GraphError::VertexNotFound(edge.to.clone()));
            }
            drop(vmap);

            let mut map = self.edges.lock().unwrap();
            if edge.id.is_empty() {
                edge.id = format!("e{}", map.len());
            }
            map.insert(edge.id.clone(), edge.clone());
            Ok(edge)
        }

        async fn traverse(&self, traversal: Traversal) -> Result<Vec<PathResult>, GraphError> {
            let vmap = self.vertices.lock().unwrap();
            if !vmap.contains_key(&traversal.start) {
                return Err(GraphError::VertexNotFound(traversal.start.clone()));
            }
            drop(vmap);

            let emap = self.edges.lock().unwrap();
            let mut results = Vec::new();

            for edge in emap.values() {
                let follows = match &traversal.direction {
                    TraversalDirection::Outgoing => edge.from == traversal.start,
                    TraversalDirection::Incoming => edge.to == traversal.start,
                    TraversalDirection::Both => {
                        edge.from == traversal.start || edge.to == traversal.start
                    }
                };
                if !follows {
                    continue;
                }
                if !traversal.edge_labels.is_empty() && !traversal.edge_labels.contains(&edge.label)
                {
                    continue;
                }

                let vmap = self.vertices.lock().unwrap();
                let from_v = vmap.get(&edge.from).cloned();
                let to_v = vmap.get(&edge.to).cloned();
                drop(vmap);

                if let (Some(from), Some(to)) = (from_v, to_v) {
                    results
                        .push(PathResult { vertices: vec![from, to], edges: vec![edge.clone()] });
                }
                if results.len() >= 10 {
                    break;
                }
            }
            Ok(results)
        }

        async fn query(
            &self,
            _query: &str,
            _params: Option<Params>,
        ) -> Result<QueryResult, GraphError> {
            Ok(QueryResult { columns: Vec::new(), rows: Vec::new() })
        }
    }

    #[tokio::test]
    async fn test_add_and_get_vertex() {
        let db = StubGraphDB::new();
        let v = Vertex { id: "v1".into(), label: "Person".into(), properties: Properties::new() };
        let created = db.add_vertex(v).await.unwrap();
        assert_eq!(created.id, "v1");
        let found = db.get_vertex(&"v1".into()).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_update_vertex() {
        let db = StubGraphDB::new();
        let v = Vertex {
            id: "v1".into(),
            label: "Person".into(),
            properties: {
                let mut m = Properties::new();
                m.insert("name".into(), serde_json::json!("Alice"));
                m
            },
        };
        db.add_vertex(v).await.unwrap();

        let mut props = Properties::new();
        props.insert("age".into(), serde_json::json!(30));
        let updated = db.update_vertex(&"v1".into(), props).await.unwrap();
        assert_eq!(updated.properties.get("name").unwrap(), &serde_json::json!("Alice"));
        assert_eq!(updated.properties.get("age").unwrap(), &serde_json::json!(30));
    }

    #[tokio::test]
    async fn test_add_edge_and_traverse() {
        let db = StubGraphDB::new();
        let a = Vertex { id: "a".into(), label: "Person".into(), properties: Properties::new() };
        let b = Vertex { id: "b".into(), label: "Person".into(), properties: Properties::new() };
        db.add_vertex(a).await.unwrap();
        db.add_vertex(b).await.unwrap();

        let edge = Edge {
            id: String::new(),
            label: "KNOWS".into(),
            from: "a".into(),
            to: "b".into(),
            properties: Properties::new(),
        };
        let created = db.add_edge(edge).await.unwrap();
        assert!(!created.id.is_empty());

        let paths = db
            .traverse(Traversal {
                start: "a".into(),
                edge_labels: vec!["KNOWS".into()],
                max_depth: 1,
                direction: TraversalDirection::Outgoing,
            })
            .await
            .unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].vertices.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_vertex_cascades_edges() {
        let db = StubGraphDB::new();
        let a = Vertex { id: "a".into(), label: "N".into(), properties: Properties::new() };
        let b = Vertex { id: "b".into(), label: "N".into(), properties: Properties::new() };
        db.add_vertex(a).await.unwrap();
        db.add_vertex(b).await.unwrap();
        db.add_edge(Edge {
            id: "e1".into(),
            label: "L".into(),
            from: "a".into(),
            to: "b".into(),
            properties: Properties::new(),
        })
        .await
        .unwrap();

        db.delete_vertex(&"a".into()).await.unwrap();
        assert!(db.get_vertex(&"a".into()).await.unwrap().is_none());
        // traversal from a should now fail
        let result = db
            .traverse(Traversal {
                start: "a".into(),
                edge_labels: vec![],
                max_depth: 1,
                direction: TraversalDirection::Both,
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_query() {
        let db = StubGraphDB::new();
        let res = db.query("MATCH (n) RETURN n", None).await.unwrap();
        assert!(res.columns.is_empty());
        assert!(res.rows.is_empty());
    }
}
