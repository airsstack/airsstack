# Interface Design & API Patterns

## Builder Pattern Implementation

```rust,ignore
// Server builder with type-safe configuration
pub struct McpServerBuilder {
    capabilities: ServerCapabilities,
    resource_providers: Vec<Box<dyn ResourceProvider>>,
    tool_executors: Vec<Box<dyn ToolExecutor>>,
    prompt_managers: Vec<Box<dyn PromptManager>>,
    security_policy: Option<SecurityPolicy>,
    transport_config: Option<TransportConfig>,
}

impl McpServerBuilder {
    pub fn new() -> Self {
        Self {
            capabilities: ServerCapabilities::default(),
            resource_providers: Vec::new(),
            tool_executors: Vec::new(),
            prompt_managers: Vec::new(),
            security_policy: None,
            transport_config: None,
        }
    }
    
    pub fn add_resource_provider<T>(mut self, provider: T) -> Self 
    where
        T: ResourceProvider + 'static,
    {
        self.resource_providers.push(Box::new(provider));
        self.capabilities.resources = Some(ResourceCapabilities {
            subscribe: provider.supports_subscriptions(),
            list_changed: provider.supports_change_notifications(),
        });
        self
    }
    
    pub fn add_tool_executor<T>(mut self, executor: T) -> Self
    where
        T: ToolExecutor + 'static,
    {
        self.tool_executors.push(Box::new(executor));
        self.capabilities.tools = Some(ToolCapabilities {});
        self
    }
    
    pub fn with_security_policy(mut self, policy: SecurityPolicy) -> Self {
        self.security_policy = Some(policy);
        self
    }
    
    pub fn with_transport_config(mut self, config: TransportConfig) -> Self {
        self.transport_config = Some(config);
        self
    }
    
    pub fn build(self) -> Result<McpServer, BuildError> {
        let security_policy = self.security_policy.unwrap_or_default();
        let transport_config = self.transport_config.unwrap_or_default();
        
        Ok(McpServer {
            capabilities: self.capabilities,
            resource_providers: self.resource_providers,
            tool_executors: self.tool_executors,
            prompt_managers: self.prompt_managers,
            security_policy,
            transport_config,
        })
    }
}

// Usage example
let server = McpServerBuilder::new()
    .add_resource_provider(FilesystemProvider::new("/home/user/docs"))
    .add_tool_executor(CalculatorTool::new())
    .add_prompt_manager(TemplateManager::new())
    .with_security_policy(SecurityPolicy::strict())
    .with_transport_config(TransportConfig::stdio())
    .build()?;
```

## Trait-Based Extension System

```rust,ignore
// Resource provider trait with async support
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    async fn list_resources(&self) -> Result<Vec<Resource>, ResourceError>;
    async fn read_resource(&self, uri: &str) -> Result<ResourceContent, ResourceError>;
    
    // Optional subscription support
    fn supports_subscriptions(&self) -> bool { false }
    async fn subscribe_to_resource(
        &self, 
        uri: &str, 
        callback: ResourceCallback
    ) -> Result<SubscriptionId, ResourceError> {
        Err(ResourceError::SubscriptionsNotSupported)
    }
    
    // Optional template support
    fn supports_templates(&self) -> bool { false }
    async fn list_resource_templates(&self) -> Result<Vec<ResourceTemplate>, ResourceError> {
        Ok(Vec::new())
    }
}

// Tool executor trait with safety integration
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<Tool>, ToolError>;
    async fn execute_tool(
        &self, 
        call: ToolCall, 
        approval: Option<UserApproval>
    ) -> Result<ToolResult, ToolError>;
    
    // Safety assessment
    fn get_safety_level(&self, tool_name: &str) -> SafetyLevel;
    fn requires_approval(&self, tool_name: &str) -> bool {
        matches!(self.get_safety_level(tool_name), SafetyLevel::Moderate | SafetyLevel::Dangerous)
    }
}

// Prompt manager trait with completion support
#[async_trait]
pub trait PromptManager: Send + Sync {
    async fn list_prompts(&self) -> Result<Vec<Prompt>, PromptError>;
    async fn get_prompt(
        &self, 
        name: &str, 
        arguments: Option<HashMap<String, serde_json::Value>>
    ) -> Result<PromptGetResult, PromptError>;
    
    // Optional completion support
    fn supports_completion(&self) -> bool { false }
    async fn complete_prompt(
        &self,
        ref_: CompletionRef,
        argument: CompletionArgument,
    ) -> Result<CompletionResult, PromptError> {
        Err(PromptError::CompletionNotSupported)
    }
}
```

## Error Handling Strategy

```rust,ignore
// Hierarchical error design with context preservation
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Server feature error: {0}")]
    ServerFeature(#[from] ServerFeatureError),
    
    #[error("Client feature error: {0}")]
    ClientFeature(#[from] ClientFeatureError),
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Invalid message format: {reason}")]
    InvalidMessage { reason: String },
    
    #[error("Protocol phase violation: cannot use method '{method}' in phase '{phase:?}'")]
    PhaseViolation { method: String, phase: ConnectionPhase },
    
    #[error("Capability not supported: {capability}")]
    CapabilityNotSupported { capability: String },
    
    #[error("Request correlation failed: {request_id}")]
    CorrelationFailed { request_id: RequestId },
    
    #[error("Request timeout after {timeout:?}")]
    RequestTimeout { timeout: Duration },
}

// Error context preservation
impl From<ValidationError> for ProtocolError {
    fn from(err: ValidationError) -> Self {
        match err {
            ValidationError::InvalidFormat { field, expected, actual } => {
                ProtocolError::InvalidMessage {
                    reason: format!("Field '{}': expected {}, got {}", field, expected, actual),
                }
            }
            ValidationError::PhaseConstraint { method, current_phase } => {
                ProtocolError::PhaseViolation {
                    method,
                    phase: current_phase,
                }
            }
        }
    }
}

// Result type aliases for ergonomic error handling
pub type McpResult<T> = Result<T, McpError>;
pub type ProtocolResult<T> = Result<T, ProtocolError>;
pub type TransportResult<T> = Result<T, TransportError>;
```
