//! Type definitions for DeepSeek API communication
//! 
//! This module contains all the serde structures for request/response handling
//! with the DeepSeek API, following OpenAI-compatible format.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Main request structure for DeepSeek Chat Completions API
#[derive(Serialize, Debug, Clone)]
pub struct ChatCompletionRequest {
    /// Model to use (e.g., "deepseek-chat", "deepseek-reasoner")
    pub model: String,
    
    /// List of messages comprising the conversation
    pub messages: Vec<ChatMessage>,
    
    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Top-p nucleus sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// Response format specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    
    /// Tools available to the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    
    /// Tool choice strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

/// Individual chat message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    /// Role of the message author
    pub role: String,
    
    /// Content of the message
    pub content: String,
    
    /// Optional name for the participant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Tool calls (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    
    /// Tool call ID (for tool messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Response format specification
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseFormat {
    /// Format type: "text" or "json_object"
    #[serde(rename = "type")]
    pub format_type: String,
}

/// Tool definition
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    /// Tool type (currently only "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Function definition
    pub function: FunctionDefinition,
}

/// Function definition for tools
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    
    /// Function description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Function parameters as JSON Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool choice specification
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ToolChoice {
    /// String choice: "none", "auto", "required"
    String(String),
    
    /// Specific tool choice
    Specific {
        #[serde(rename = "type")]
        tool_type: String,
        function: FunctionChoice,
    },
}

/// Specific function choice
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionChoice {
    /// Function name to call
    pub name: String,
}

/// Tool call made by the model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Function call details
    pub function: FunctionCall,
}

/// Function call details
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    
    /// Function arguments as JSON string
    pub arguments: String,
}

/// Main response structure from DeepSeek API
#[derive(Deserialize, Debug, Clone)]
pub struct ChatCompletionResponse {
    /// Unique identifier for the completion
    pub id: String,
    
    /// Object type (always "chat.completion")
    pub object: String,
    
    /// Unix timestamp of creation
    pub created: u64,
    
    /// Model used for completion
    pub model: String,
    
    /// System fingerprint
    pub system_fingerprint: String,
    
    /// List of completion choices
    pub choices: Vec<Choice>,
    
    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Individual completion choice
#[derive(Deserialize, Debug, Clone)]
pub struct Choice {
    /// Index of the choice
    pub index: u32,
    
    /// The completion message
    pub message: ChatMessage,
    
    /// Reason for finishing
    pub finish_reason: String,
    
    /// Log probabilities (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<LogProbs>,
}

/// Log probability information
#[derive(Deserialize, Debug, Clone)]
pub struct LogProbs {
    /// Content log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentLogProb>>,
}

/// Content log probability
#[derive(Deserialize, Debug, Clone)]
pub struct ContentLogProb {
    /// The token
    pub token: String,
    
    /// Log probability
    pub logprob: f64,
    
    /// UTF-8 bytes representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    
    /// Top log probabilities
    pub top_logprobs: Vec<TopLogProb>,
}

/// Top log probability
#[derive(Deserialize, Debug, Clone)]
pub struct TopLogProb {
    /// The token
    pub token: String,
    
    /// Log probability
    pub logprob: f64,
    
    /// UTF-8 bytes representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}

/// Usage statistics
#[derive(Deserialize, Debug, Clone)]
pub struct Usage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,
    
    /// Tokens in the completion
    pub completion_tokens: u32,
    
    /// Total tokens used
    pub total_tokens: u32,
    
    /// Prompt cache hit tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_hit_tokens: Option<u32>,
    
    /// Prompt cache miss tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_miss_tokens: Option<u32>,
    
    /// Completion tokens details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Completion tokens breakdown
#[derive(Deserialize, Debug, Clone)]
pub struct CompletionTokensDetails {
    /// Reasoning tokens (for deepseek-reasoner)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

/// API error response structure
#[derive(Deserialize, Debug, Clone)]
pub struct ApiErrorResponse {
    /// Error details
    pub error: ErrorDetails,
}

/// Error details from API
#[derive(Deserialize, Debug, Clone)]
pub struct ErrorDetails {
    /// Error message
    pub message: String,
    
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    
    /// Error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    
    /// Additional error parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

impl ChatCompletionRequest {
    /// Create a new chat completion request with minimal parameters
    pub fn new(model: String, messages: Vec<ChatMessage>) -> Self {
        Self {
            model,
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: None,
            response_format: None,
            tools: None,
            tool_choice: None,
        }
    }
    
    /// Set temperature for the request
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    /// Set max tokens for the request
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// Enable JSON output mode
    pub fn with_json_output(mut self) -> Self {
        self.response_format = Some(ResponseFormat {
            format_type: "json_object".to_string(),
        });
        self
    }
}

impl ChatMessage {
    /// Create a system message
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
    
    /// Create a user message
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
    
    /// Create an assistant message
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
}
