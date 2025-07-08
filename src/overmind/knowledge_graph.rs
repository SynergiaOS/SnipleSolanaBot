//! Knowledge Graph Implementation
//!
//! Rust-native implementation of AutoSchema Knowledge Graph
//! using Qdrant for vector storage and Polars for data processing

use anyhow::Result;
use reqwest::Client;
// Future Qdrant integration - currently using HTTP API
// use qdrant_client::{
//     client::QdrantClient,
//     qdrant::{
//         CreateCollection, Distance, VectorParams, PointStruct,
//         SearchPoints, Filter, Condition, FieldCondition, Match
//     }
// };
// use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug};

/// Entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub observations: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

/// Relation between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: Uuid,
    pub from_entity: Uuid,
    pub to_entity: Uuid,
    pub relation_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

/// Knowledge graph implementation with Qdrant HTTP API integration
pub struct KnowledgeGraph {
    /// In-memory entity cache
    entities: RwLock<HashMap<Uuid, Entity>>,

    /// In-memory relation cache
    relations: RwLock<HashMap<Uuid, Relation>>,

    /// HTTP client for Qdrant API
    http_client: Client,

    /// Qdrant configuration
    qdrant_config: QdrantConfig,
}

/// Qdrant configuration
#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub collection_name: String,
    pub vector_size: u64,
}

impl KnowledgeGraph {
    /// Create new knowledge graph instance
    pub async fn new() -> Result<Self> {
        info!("🧠 Initializing Knowledge Graph with Qdrant HTTP API");

        let http_client = Client::new();

        let qdrant_config = QdrantConfig {
            url: std::env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6334".to_string()),
            api_key: std::env::var("QDRANT_API_KEY").ok(),
            collection_name: "overmind_knowledge".to_string(),
            vector_size: 384, // Default embedding size
        };

        let kg = Self {
            entities: RwLock::new(HashMap::new()),
            relations: RwLock::new(HashMap::new()),
            http_client,
            qdrant_config,
        };

        // Initialize Qdrant collection
        if let Err(e) = kg.initialize_qdrant_collection().await {
            tracing::warn!("⚠️ Failed to initialize Qdrant collection: {}", e);
            tracing::info!("📝 Continuing with in-memory mode only");
        }

        Ok(kg)
    }

    /// Initialize Qdrant collection via HTTP API
    async fn initialize_qdrant_collection(&self) -> Result<()> {
        debug!("🔧 Initializing Qdrant collection: {}", self.qdrant_config.collection_name);

        let collection_url = format!(
            "{}/collections/{}",
            self.qdrant_config.url,
            self.qdrant_config.collection_name
        );

        // Check if collection exists
        let mut request = self.http_client.get(&collection_url);

        if let Some(api_key) = &self.qdrant_config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            debug!("✅ Qdrant collection already exists");
            return Ok(());
        }

        // Create collection
        let create_collection_url = format!("{}/collections/{}", self.qdrant_config.url, self.qdrant_config.collection_name);

        let create_payload = serde_json::json!({
            "vectors": {
                "size": self.qdrant_config.vector_size,
                "distance": "Cosine"
            }
        });

        let mut create_request = self.http_client
            .put(&create_collection_url)
            .json(&create_payload);

        if let Some(api_key) = &self.qdrant_config.api_key {
            create_request = create_request.header("api-key", api_key);
        }

        let create_response = create_request.send().await?;

        if create_response.status().is_success() {
            info!("✅ Created Qdrant collection: {}", self.qdrant_config.collection_name);
        } else {
            let error_text = create_response.text().await?;
            anyhow::bail!("Failed to create Qdrant collection: {}", error_text);
        }

        Ok(())
    }

    /// Store entity embedding in Qdrant
    async fn store_entity_embedding(&self, entity: &Entity, embedding: Vec<f32>) -> Result<()> {
        debug!("📊 Storing entity embedding: {}", entity.name);

        let points_url = format!(
            "{}/collections/{}/points",
            self.qdrant_config.url,
            self.qdrant_config.collection_name
        );

        let point_payload = serde_json::json!({
            "points": [{
                "id": entity.id.to_string(),
                "vector": embedding,
                "payload": {
                    "entity_id": entity.id.to_string(),
                    "name": entity.name,
                    "entity_type": entity.entity_type,
                    "properties": entity.properties,
                    "created_at": entity.created_at.to_rfc3339(),
                    "updated_at": entity.updated_at.to_rfc3339()
                }
            }]
        });

        let mut request = self.http_client
            .put(&points_url)
            .json(&point_payload);

        if let Some(api_key) = &self.qdrant_config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to store entity embedding: {}", error_text);
        }

        Ok(())
    }

    /// Search entities by vector similarity
    async fn search_entities_by_vector(&self, query_vector: Vec<f32>, limit: u64) -> Result<Vec<Entity>> {
        debug!("🔍 Searching entities by vector similarity");

        let search_url = format!(
            "{}/collections/{}/points/search",
            self.qdrant_config.url,
            self.qdrant_config.collection_name
        );

        let search_payload = serde_json::json!({
            "vector": query_vector,
            "limit": limit,
            "with_payload": true
        });

        let mut request = self.http_client
            .post(&search_url)
            .json(&search_payload);

        if let Some(api_key) = &self.qdrant_config.api_key {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to search entities: {}", error_text);
        }

        let search_result: serde_json::Value = response.json().await?;
        let mut entities = Vec::new();

        if let Some(result_array) = search_result["result"].as_array() {
            for result in result_array {
                if let Some(payload) = result["payload"].as_object() {
                    // Parse entity from payload
                    if let (Some(id_str), Some(name), Some(entity_type)) = (
                        payload.get("entity_id").and_then(|v| v.as_str()),
                        payload.get("name").and_then(|v| v.as_str()),
                        payload.get("entity_type").and_then(|v| v.as_str())
                    ) {
                        if let Ok(id) = Uuid::parse_str(id_str) {
                            let entity = Entity {
                                id,
                                name: name.to_string(),
                                entity_type: entity_type.to_string(),
                                properties: payload.get("properties")
                                    .and_then(|v| v.as_object())
                                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                                    .unwrap_or_default(),
                                observations: Vec::new(), // Not stored in vector search
                                created_at: chrono::Utc::now(), // Simplified
                                updated_at: chrono::Utc::now(),
                                version: 1,
                            };
                            entities.push(entity);
                        }
                    }
                }
            }
        }

        Ok(entities)
    }

    /// Generate embedding for text using Jina AI API
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        debug!("🧠 Generating embedding for text: {:.50}...", text);

        // Use Jina AI API for embeddings
        let jina_api_key = std::env::var("JINA_API_KEY")
            .unwrap_or_else(|_| "jina_72cc7ed00e21496290ed9e018d56de3bETDGPqW-TUXuYYIxk4jwHLN9h0C6".to_string());

        let embedding_url = "https://api.jina.ai/v1/embeddings";

        let embedding_payload = serde_json::json!({
            "model": "jina-embeddings-v2-base-en",
            "input": [text]
        });

        let response = self.http_client
            .post(embedding_url)
            .header("Authorization", format!("Bearer {}", jina_api_key))
            .header("Content-Type", "application/json")
            .json(&embedding_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to generate embedding: {}", error_text);
        }

        let embedding_result: serde_json::Value = response.json().await?;

        if let Some(data) = embedding_result["data"].as_array() {
            if let Some(first_embedding) = data.first() {
                if let Some(embedding_array) = first_embedding["embedding"].as_array() {
                    let embedding: Result<Vec<f32>, _> = embedding_array
                        .iter()
                        .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(|| anyhow::anyhow!("Invalid embedding value")))
                        .collect();

                    return embedding;
                }
            }
        }

        anyhow::bail!("Invalid embedding response format")
    }
    
    /// Create or update entity
    pub async fn upsert_entity(&self, entity: Entity) -> Result<()> {
        debug!("📝 Upserting entity: {}", entity.name);

        // Generate embedding for entity
        let entity_text = format!("{} {} {:?}", entity.name, entity.entity_type, entity.properties);

        match self.generate_embedding(&entity_text).await {
            Ok(embedding) => {
                // Store in Qdrant
                if let Err(e) = self.store_entity_embedding(&entity, embedding).await {
                    tracing::warn!("⚠️ Failed to store entity in Qdrant: {}", e);
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to generate embedding: {}", e);
            }
        }

        // Update local cache
        let mut entities = self.entities.write().await;
        entities.insert(entity.id, entity);

        Ok(())
    }
    
    /// Create or update relation
    pub async fn upsert_relation(&self, relation: Relation) -> Result<()> {
        debug!("🔗 Upserting relation: {} -> {}", relation.from_entity, relation.to_entity);

        // Update local cache (simplified version)
        let mut relations = self.relations.write().await;
        relations.insert(relation.id, relation);

        Ok(())
    }
    
    /// Search entities by text query using vector similarity
    pub async fn search_entities(&self, query: &str, limit: u64) -> Result<Vec<Entity>> {
        debug!("🔍 Searching entities: {}", query);

        // Try vector search first
        match self.generate_embedding(query).await {
            Ok(query_embedding) => {
                match self.search_entities_by_vector(query_embedding, limit).await {
                    Ok(vector_results) => {
                        if !vector_results.is_empty() {
                            debug!("✅ Found {} entities via vector search", vector_results.len());
                            return Ok(vector_results);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Vector search failed: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to generate query embedding: {}", e);
            }
        }

        // Fallback to simple text search in entity names and observations
        debug!("🔄 Falling back to text search");
        let entities = self.entities.read().await;
        let mut results = Vec::new();

        for entity in entities.values() {
            if entity.name.to_lowercase().contains(&query.to_lowercase()) ||
               entity.observations.iter().any(|obs| obs.to_lowercase().contains(&query.to_lowercase())) {
                results.push(entity.clone());
                if results.len() >= limit as usize {
                    break;
                }
            }
        }

        Ok(results)
    }
    
    /// Get entity by ID
    pub async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>> {
        let entities = self.entities.read().await;
        Ok(entities.get(&id).cloned())
    }
    
    /// Get relations for entity
    pub async fn get_entity_relations(&self, entity_id: Uuid) -> Result<Vec<Relation>> {
        let relations = self.relations.read().await;
        let entity_relations: Vec<Relation> = relations
            .values()
            .filter(|r| r.from_entity == entity_id || r.to_entity == entity_id)
            .cloned()
            .collect();
        
        Ok(entity_relations)
    }
    
    /// Process market data and extract entities/relations
    pub async fn process_market_data(&self, data: serde_json::Value) -> Result<()> {
        debug!("📊 Processing market data for knowledge extraction");
        
        // Extract entities from market data
        if let Some(token_address) = data.get("token_address").and_then(|v| v.as_str()) {
            let entity = Entity {
                id: Uuid::new_v4(),
                name: format!("Token_{}", token_address),
                entity_type: "token".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("address".to_string(), serde_json::Value::String(token_address.to_string()));
                    if let Some(price) = data.get("price") {
                        props.insert("current_price".to_string(), price.clone());
                    }
                    props
                },
                observations: vec![format!("Observed at {}", chrono::Utc::now())],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
            };
            
            self.upsert_entity(entity).await?;
        }
        
        // Extract developer information
        if let Some(developer) = data.get("developer").and_then(|v| v.as_str()) {
            let entity = Entity {
                id: Uuid::new_v4(),
                name: format!("Developer_{}", developer),
                entity_type: "developer".to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("address".to_string(), serde_json::Value::String(developer.to_string()));
                    props
                },
                observations: vec![format!("Created token at {}", chrono::Utc::now())],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
            };
            
            self.upsert_entity(entity).await?;
        }
        
        Ok(())
    }
    
    // Embedding functions removed for simplified version
    
    /// Get knowledge graph statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let entities = self.entities.read().await;
        let relations = self.relations.read().await;
        
        Ok(serde_json::json!({
            "entities_count": entities.len(),
            "relations_count": relations.len(),
            "entity_types": self.get_entity_type_counts(&entities).await,
            "relation_types": self.get_relation_type_counts(&relations).await
        }))
    }
    
    /// Get entity type counts
    async fn get_entity_type_counts(&self, entities: &HashMap<Uuid, Entity>) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entity in entities.values() {
            *counts.entry(entity.entity_type.clone()).or_insert(0) += 1;
        }
        counts
    }
    
    /// Get relation type counts
    async fn get_relation_type_counts(&self, relations: &HashMap<Uuid, Relation>) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for relation in relations.values() {
            *counts.entry(relation.relation_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Store knowledge as an entity
    pub async fn store_knowledge(&self, name: &str, data: serde_json::Value) -> Result<()> {
        let mut properties = HashMap::new();

        // Convert serde_json::Value to HashMap if it's an object
        if let serde_json::Value::Object(map) = data {
            for (key, value) in map {
                properties.insert(key, value);
            }
        } else {
            // If it's not an object, store it under "data" key
            properties.insert("data".to_string(), data);
        }

        let entity = Entity {
            id: Uuid::new_v4(),
            name: name.to_string(),
            entity_type: "knowledge".to_string(),
            properties,
            observations: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };

        self.upsert_entity(entity).await
    }
}
