use qdrant_client::qdrant::{
    CreateCollection, Distance, PointStruct, SearchPoints,
    UpsertPoints, VectorParams,
};
use qdrant_client::prelude::*;
use uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing Qdrant Cloud with official Rust client...");
    
    // Create client for Qdrant Cloud
    let mut config = QdrantClient::from_url("https://b2654b99-faa9-4e39-b9b5-cf2e1b176bca.us-east4-0.gcp.cloud.qdrant.io:6334");
    config.set_api_key("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3MiOiJtIn0.YZdwoy8_iAFBBTw2Z89i5_VhjHSAzKUF78u6qdYAwkU");
    let client = config.build()?;
    
    println!("✅ Client created successfully");
    
    // Test 1: List collections
    println!("📋 Testing collections list...");
    match client.list_collections().await {
        Ok(collections_list) => {
            println!("✅ Collections list retrieved successfully:");
            println!("{:#?}", collections_list);
        }
        Err(e) => {
            println!("❌ Failed to list collections: {}", e);
            return Err(e.into());
        }
    }
    
    // Test 2: Create test collection
    println!("🏗️ Testing collection creation...");
    let collection_name = "overmind_test_rust";
    
    // Delete collection if it exists
    let _ = client.delete_collection(collection_name).await;
    
    let create_collection = CreateCollection {
        collection_name: collection_name.to_string(),
        vectors_config: Some(qdrant_client::qdrant::VectorsConfig {
            config: Some(qdrant_client::qdrant::vectors_config::Config::Params(VectorParams {
                size: 384,
                distance: Distance::Cosine.into(),
                ..Default::default()
            })),
        }),
        ..Default::default()
    };

    match client.create_collection(&create_collection).await
    {
        Ok(_) => {
            println!("✅ Test collection created successfully");
        }
        Err(e) => {
            println!("❌ Failed to create collection: {}", e);
            return Err(e.into());
        }
    }
    
    // Test 3: Get collection info
    println!("📊 Testing collection info...");
    match client.collection_info(collection_name).await {
        Ok(collection_info) => {
            println!("✅ Collection info retrieved:");
            println!("{:#?}", collection_info);
        }
        Err(e) => {
            println!("❌ Failed to get collection info: {}", e);
        }
    }
    
    // Test 4: Insert test point
    println!("📊 Testing point insertion...");
    let payload: Payload = serde_json::json!({
        "test": true,
        "name": "overmind_test_point",
        "entity_type": "test",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })
    .try_into()
    .unwrap();
    
    let test_vector: Vec<f32> = (0..384).map(|i| (i as f32) / 384.0).collect();
    let points = vec![PointStruct::new(uuid::Uuid::new_v4().to_string(), test_vector.clone(), payload)];
    
    match client.upsert_points(collection_name, None, points, None).await
    {
        Ok(_) => {
            println!("✅ Test point inserted successfully");
        }
        Err(e) => {
            println!("❌ Failed to insert point: {}", e);
        }
    }
    
    // Test 5: Search test
    println!("🔍 Testing vector search...");
    let search_points = SearchPoints {
        collection_name: collection_name.to_string(),
        vector: test_vector,
        limit: 5,
        with_payload: Some(true.into()),
        ..Default::default()
    };

    match client.search_points(&search_points).await
    {
        Ok(search_result) => {
            println!("✅ Search completed successfully:");
            println!("{:#?}", search_result);
        }
        Err(e) => {
            println!("❌ Search failed: {}", e);
        }
    }
    
    println!("🎉 Qdrant Cloud Rust client test completed!");
    Ok(())
}
