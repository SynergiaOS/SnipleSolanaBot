use qdrant_client::prelude::*;

#[tokio::main]
async fn main() {
    println!("🔗 Testing Qdrant Cloud with new JWT token...");
    
    let mut config = QdrantClient::from_url("https://b2654b99-faa9-4e39-b9b5-cf2e1b176bca.us-east4-0.gcp.cloud.qdrant.io:6334");
    config.set_api_key("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhY2Nlc3MiOiJtIn0.YZdwoy8_iAFBBTw2Z89i5_VhjHSAzKUF78u6qdYAwkU");
    
    match config.build() {
        Ok(client) => {
            println!("✅ Client created successfully");
            
            let collections_list = client.list_collections().await;
            match collections_list {
                Ok(collections) => {
                    println!("🎉 SUCCESS! Collections retrieved:");
                    println!("{:#?}", collections);
                }
                Err(e) => {
                    println!("❌ Failed to list collections: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create client: {}", e);
        }
    }
}
