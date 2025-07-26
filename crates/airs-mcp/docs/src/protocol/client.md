# Client Feature Specifications

## Sampling: Server-Initiated AI Interactions

```rust,ignore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRequest {
    pub messages: Vec<SamplingMessage>,
    pub model_preferences: Option<ModelPreferences>,
    pub system_prompt: Option<String>,
    pub include_context: Option<ContextInclusion>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    pub hints: Option<ModelHints>,
    pub cost_priority: Option<f64>,      // 0.0-1.0
    pub speed_priority: Option<f64>,     // 0.0-1.0  
    pub intelligence_priority: Option<f64>, // 0.0-1.0
}
```

Double Approval Workflow:

```rust,ignore
// Step 1: Server requests sampling permission
server.request_sampling_approval(request).await?;

// Step 2: User approves the AI interaction  
if user_approves_sampling(request) {
    let response = client.create_message(request).await?;
    
    // Step 3: User approves the AI response
    if user_approves_response(response) {
        return Ok(response);
    }
}
```

Context Inclusion Controls:

```rust,ignore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextInclusion {
    None,           // No additional context
    ThisServer,     // Context from requesting server only
    AllServers,     // Context from all connected servers
}
```
