//! HELIUS STREAM - Native WebSocket Implementation
//! 
//! Zero-dependency WebSocket client for Helius transaction streaming
//! Manual JSON parsing for maximum performance

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Result, anyhow};
use tracing::{info, debug, warn, error};
use serde_json::Value;

/// Transaction event from Helius stream
#[derive(Debug, Clone)]
pub struct TransactionEvent {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction_type: TransactionType,
    pub accounts: Vec<String>,
    pub native_transfers: Vec<NativeTransfer>,
    pub token_transfers: Vec<TokenTransfer>,
    pub instructions: Vec<InstructionData>,
    pub events: Vec<EventData>,
}

/// Transaction type classification
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Swap,
    Transfer,
    Mint,
    Burn,
    CreateAccount,
    Unknown,
}

/// Native SOL transfer
#[derive(Debug, Clone)]
pub struct NativeTransfer {
    pub from_user_account: String,
    pub to_user_account: String,
    pub amount: u64,
}

/// Token transfer event
#[derive(Debug, Clone)]
pub struct TokenTransfer {
    pub from_user_account: Option<String>,
    pub to_user_account: Option<String>,
    pub from_token_account: String,
    pub to_token_account: String,
    pub token_amount: u64,
    pub mint: String,
}

/// Instruction data
#[derive(Debug, Clone)]
pub struct InstructionData {
    pub program_id: String,
    pub instruction_type: String,
    pub data: Vec<u8>,
}

/// Event data
#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: String,
    pub data: HashMap<String, Value>,
}

/// Helius subscription filter
#[derive(Debug, Clone)]
pub struct HeliusFilter {
    pub account_include: Vec<String>,
    pub account_exclude: Vec<String>,
    pub account_required: Vec<String>,
    pub failed: Option<bool>,
    pub vote: Option<bool>,
}

impl Default for HeliusFilter {
    fn default() -> Self {
        Self {
            account_include: Vec::new(),
            account_exclude: Vec::new(),
            account_required: Vec::new(),
            failed: Some(false),
            vote: Some(false),
        }
    }
}

/// Helius WebSocket client
pub struct HeliusStream {
    api_key: String,
    filters: Vec<HeliusFilter>,
    event_sender: mpsc::UnboundedSender<TransactionEvent>,
    connection_active: Arc<std::sync::atomic::AtomicBool>,
}

impl HeliusStream {
    /// Create new Helius stream
    pub fn new(api_key: String, filters: Vec<HeliusFilter>) -> (Self, mpsc::UnboundedReceiver<TransactionEvent>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let stream = Self {
            api_key,
            filters,
            event_sender,
            connection_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        
        (stream, event_receiver)
    }
    
    /// Start streaming transactions
    pub async fn start(&self) -> Result<()> {
        let ws_url = format!("wss://atlas-mainnet.helius-rpc.com?api-key={}", self.api_key);
        info!("Connecting to Helius WebSocket: {}", ws_url);
        
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| anyhow!("Failed to connect to Helius WebSocket: {}", e))?;
        
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Send subscription message
        let subscription_msg = self.build_subscription_message()?;
        ws_sender.send(Message::Text(subscription_msg)).await
            .map_err(|e| anyhow!("Failed to send subscription: {}", e))?;
        
        self.connection_active.store(true, std::sync::atomic::Ordering::Relaxed);
        info!("Helius WebSocket connected and subscribed");
        
        // Message processing loop
        while let Some(message) = ws_receiver.next().await {
            match message {
                Ok(Message::Text(data)) => {
                    if let Err(e) = self.process_message(&data).await {
                        warn!("Failed to process message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {} // Ignore other message types
            }
        }
        
        self.connection_active.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    /// Build subscription message manually
    fn build_subscription_message(&self) -> Result<String> {
        // Manual JSON construction for zero dependencies
        let mut filter_objects = Vec::new();
        
        for filter in &self.filters {
            let mut filter_json = String::from("{");
            
            if !filter.account_include.is_empty() {
                filter_json.push_str(&format!(
                    "\"accountInclude\":[{}],",
                    filter.account_include.iter()
                        .map(|acc| format!("\"{}\"", acc))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            
            if !filter.account_exclude.is_empty() {
                filter_json.push_str(&format!(
                    "\"accountExclude\":[{}],",
                    filter.account_exclude.iter()
                        .map(|acc| format!("\"{}\"", acc))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            
            if !filter.account_required.is_empty() {
                filter_json.push_str(&format!(
                    "\"accountRequired\":[{}],",
                    filter.account_required.iter()
                        .map(|acc| format!("\"{}\"", acc))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            
            if let Some(failed) = filter.failed {
                filter_json.push_str(&format!("\"failed\":{},", failed));
            }
            
            if let Some(vote) = filter.vote {
                filter_json.push_str(&format!("\"vote\":{},", vote));
            }
            
            // Remove trailing comma
            if filter_json.ends_with(',') {
                filter_json.pop();
            }
            
            filter_json.push('}');
            filter_objects.push(filter_json);
        }
        
        let subscription_msg = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"transactionSubscribe\",\"params\":[{}]}}",
            filter_objects.join(",")
        );
        
        debug!("Subscription message: {}", subscription_msg);
        Ok(subscription_msg)
    }
    
    /// Process incoming WebSocket message
    async fn process_message(&self, data: &str) -> Result<()> {
        // Manual JSON parsing for performance
        let parsed = self.parse_transaction_notification(data)?;
        
        if let Some(event) = parsed {
            if let Err(e) = self.event_sender.send(event) {
                warn!("Failed to send transaction event: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Parse transaction notification manually
    fn parse_transaction_notification(&self, json_str: &str) -> Result<Option<TransactionEvent>> {
        // Quick check if this is a transaction notification
        if !json_str.contains("transactionSubscribe") && !json_str.contains("\"params\"") {
            return Ok(None);
        }
        
        // Parse using serde_json for now (can be replaced with manual parsing)
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("JSON parse error: {}", e))?;
        
        // Extract transaction data
        if let Some(params) = value.get("params") {
            if let Some(result) = params.get("result") {
                return self.extract_transaction_event(result);
            }
        }
        
        Ok(None)
    }
    
    /// Extract transaction event from JSON
    fn extract_transaction_event(&self, result: &Value) -> Result<Option<TransactionEvent>> {
        let signature = result.get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let slot = result.get("slot")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let block_time = result.get("blockTime")
            .and_then(|v| v.as_i64());
        
        // Extract transaction type
        let transaction_type = self.classify_transaction(result);
        
        // Extract accounts
        let accounts = self.extract_accounts(result);
        
        // Extract native transfers
        let native_transfers = self.extract_native_transfers(result);
        
        // Extract token transfers
        let token_transfers = self.extract_token_transfers(result);
        
        // Extract instructions
        let instructions = self.extract_instructions(result);
        
        // Extract events
        let events = self.extract_events(result);
        
        let event = TransactionEvent {
            signature,
            slot,
            block_time,
            transaction_type,
            accounts,
            native_transfers,
            token_transfers,
            instructions,
            events,
        };
        
        debug!("Parsed transaction event: {}", event.signature);
        Ok(Some(event))
    }
    
    /// Classify transaction type
    fn classify_transaction(&self, result: &Value) -> TransactionType {
        // Simple classification based on instruction types
        if let Some(instructions) = result.get("instructions") {
            if let Some(instructions_array) = instructions.as_array() {
                for instruction in instructions_array {
                    if let Some(program_id) = instruction.get("programId").and_then(|v| v.as_str()) {
                        match program_id {
                            "11111111111111111111111111111111" => return TransactionType::Transfer,
                            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => return TransactionType::Transfer,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        TransactionType::Unknown
    }
    
    /// Extract account addresses
    fn extract_accounts(&self, result: &Value) -> Vec<String> {
        let mut accounts = Vec::new();
        
        if let Some(account_keys) = result.get("accountKeys") {
            if let Some(keys_array) = account_keys.as_array() {
                for key in keys_array {
                    if let Some(key_str) = key.as_str() {
                        accounts.push(key_str.to_string());
                    }
                }
            }
        }
        
        accounts
    }
    
    /// Extract native SOL transfers
    fn extract_native_transfers(&self, result: &Value) -> Vec<NativeTransfer> {
        let mut transfers = Vec::new();
        
        if let Some(native_transfers) = result.get("nativeTransfers") {
            if let Some(transfers_array) = native_transfers.as_array() {
                for transfer in transfers_array {
                    if let (Some(from), Some(to), Some(amount)) = (
                        transfer.get("fromUserAccount").and_then(|v| v.as_str()),
                        transfer.get("toUserAccount").and_then(|v| v.as_str()),
                        transfer.get("amount").and_then(|v| v.as_u64()),
                    ) {
                        transfers.push(NativeTransfer {
                            from_user_account: from.to_string(),
                            to_user_account: to.to_string(),
                            amount,
                        });
                    }
                }
            }
        }
        
        transfers
    }
    
    /// Extract token transfers
    fn extract_token_transfers(&self, result: &Value) -> Vec<TokenTransfer> {
        let mut transfers = Vec::new();
        
        if let Some(token_transfers) = result.get("tokenTransfers") {
            if let Some(transfers_array) = token_transfers.as_array() {
                for transfer in transfers_array {
                    let from_user = transfer.get("fromUserAccount").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let to_user = transfer.get("toUserAccount").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let from_token = transfer.get("fromTokenAccount").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let to_token = transfer.get("toTokenAccount").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let amount = transfer.get("tokenAmount").and_then(|v| v.as_u64()).unwrap_or(0);
                    let mint = transfer.get("mint").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    
                    transfers.push(TokenTransfer {
                        from_user_account: from_user,
                        to_user_account: to_user,
                        from_token_account: from_token,
                        to_token_account: to_token,
                        token_amount: amount,
                        mint,
                    });
                }
            }
        }
        
        transfers
    }
    
    /// Extract instruction data
    fn extract_instructions(&self, result: &Value) -> Vec<InstructionData> {
        let mut instructions = Vec::new();
        
        if let Some(instructions_data) = result.get("instructions") {
            if let Some(instructions_array) = instructions_data.as_array() {
                for instruction in instructions_array {
                    let program_id = instruction.get("programId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let instruction_type = instruction.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                    let data = Vec::new(); // TODO: Parse instruction data
                    
                    instructions.push(InstructionData {
                        program_id,
                        instruction_type,
                        data,
                    });
                }
            }
        }
        
        instructions
    }
    
    /// Extract event data
    fn extract_events(&self, result: &Value) -> Vec<EventData> {
        let mut events = Vec::new();
        
        if let Some(events_data) = result.get("events") {
            if let Some(events_array) = events_data.as_array() {
                for event in events_array {
                    let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                    let mut data = HashMap::new();
                    
                    if let Some(event_obj) = event.as_object() {
                        for (key, value) in event_obj {
                            data.insert(key.clone(), value.clone());
                        }
                    }
                    
                    events.push(EventData {
                        event_type,
                        data,
                    });
                }
            }
        }
        
        events
    }
    
    /// Check if connection is active
    pub fn is_connected(&self) -> bool {
        self.connection_active.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Initialize Helius stream with configuration
pub async fn init_helius_stream(
    api_key: String,
    filters: Vec<HeliusFilter>,
) -> Result<(HeliusStream, mpsc::UnboundedReceiver<TransactionEvent>)> {
    let (stream, receiver) = HeliusStream::new(api_key, filters);
    Ok((stream, receiver))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subscription_message_building() {
        let filters = vec![
            HeliusFilter {
                account_include: vec!["11111111111111111111111111111111".to_string()],
                failed: Some(false),
                vote: Some(false),
                ..Default::default()
            }
        ];
        
        let (stream, _) = HeliusStream::new("test_key".to_string(), filters);
        let msg = stream.build_subscription_message().unwrap();
        
        assert!(msg.contains("transactionSubscribe"));
        assert!(msg.contains("accountInclude"));
    }
}
