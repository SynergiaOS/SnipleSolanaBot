use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing Qdrant Cloud connection...");
    
    let client = Client::new();
    let qdrant_url = "https://b2654b99-faa9-4e39-b9b5-cf2e1b176bca.us-east4-0.gcp.cloud.qdrant.io:6334";
    let api_key = "5ee5b660-e4dc-4676-8e1d-a2b69b72ce36";
    
    // Test 1: List collections
    println!("📋 Testing collections list...");
    let collections_url = format!("{}/collections", qdrant_url);
    
    println!("🔑 Using API key: {}...", &api_key[..20]);
    println!("🌐 URL: {}", collections_url);

    // Try different authorization headers
    let response = client
        .get(&collections_url)
        .header("api-key", api_key)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;
    
    if response.status().is_success() {
        let collections: serde_json::Value = response.json().await?;
        println!("✅ Collections response: {}", serde_json::to_string_pretty(&collections)?);
    } else {
        println!("❌ Failed to list collections: {}", response.status());
        let error_text = response.text().await?;
        println!("Error: {}", error_text);
        return Ok(());
    }
    
    // Test 2: Create test collection
    println!("🏗️ Testing collection creation...");
    let collection_name = "overmind_test";
    let create_url = format!("{}/collections/{}", qdrant_url, collection_name);
    
    let create_payload = json!({
        "vectors": {
            "size": 384,
            "distance": "Cosine"
        }
    });
    
    let create_response = client
        .put(&create_url)
        .header("api-key", api_key)
        .json(&create_payload)
        .send()
        .await?;
    
    if create_response.status().is_success() {
        println!("✅ Test collection created successfully");
    } else {
        println!("⚠️ Collection creation response: {}", create_response.status());
        let error_text = create_response.text().await?;
        println!("Response: {}", error_text);
    }
    
    // Test 3: Insert test point
    println!("📊 Testing point insertion...");
    let points_url = format!("{}/collections/{}/points", qdrant_url, collection_name);
    
    let test_vector: Vec<f32> = (0..384).map(|i| (i as f32) / 384.0).collect();
    
    let point_payload = json!({
        "points": [{
            "id": "test-point-1",
            "vector": test_vector,
            "payload": {
                "test": true,
                "name": "overmind_test_point",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }]
    });
    
    let point_response = client
        .put(&points_url)
        .header("api-key", api_key)
        .json(&point_payload)
        .send()
        .await?;
    
    if point_response.status().is_success() {
        println!("✅ Test point inserted successfully");
    } else {
        println!("❌ Point insertion failed: {}", point_response.status());
        let error_text = point_response.text().await?;
        println!("Error: {}", error_text);
    }
    
    // Test 4: Search test
    println!("🔍 Testing vector search...");
    let search_url = format!("{}/collections/{}/points/search", qdrant_url, collection_name);
    
    let search_payload = json!({
        "vector": test_vector,
        "limit": 5,
        "with_payload": true
    });
    
    let search_response = client
        .post(&search_url)
        .header("api-key", api_key)
        .json(&search_payload)
        .send()
        .await?;
    
    if search_response.status().is_success() {
        let search_results: serde_json::Value = search_response.json().await?;
        println!("✅ Search results: {}", serde_json::to_string_pretty(&search_results)?);
    } else {
        println!("❌ Search failed: {}", search_response.status());
        let error_text = search_response.text().await?;
        println!("Error: {}", error_text);
    }
    
    println!("🎉 Qdrant Cloud connection test completed!");
    Ok(())
}
