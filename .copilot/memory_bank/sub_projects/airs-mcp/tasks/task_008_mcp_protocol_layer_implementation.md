# [TASK008] - MCP Protocol Layer Implementation

**Status:** pending  
**Added:** 2025-08-06  
**Updated:** 2025-08-06  
**Priority:** CRITICAL  
**Type:** core_functionality  
**Category:** mcp_protocol_implementation

## Original Request
Implement high-level MCP protocol abstractions on top of the existing JSON-RPC foundation to enable real MCP tool development. This is the critical missing layer that transforms the library from infrastructure into a usable MCP toolkit.

## Thought Process
The current airs-mcp library has exceptional foundational components (JSON-RPC, correlation, transport, integration) but lacks the MCP-specific protocol layer that developers need to build real MCP tools. While the foundation is production-ready with outstanding performance, users still need to manually construct MCP message formats.

**Critical Gap Analysis:**
- ✅ **JSON-RPC 2.0 Foundation** - Complete and exceptional
- ✅ **Transport & Correlation** - Production-ready with enterprise performance
- ✅ **Integration Layer** - High-level client API working
- ❌ **MCP Protocol Layer** - Missing but critically needed
- ❌ **MCP Message Types** - Missing resource/tool/prompt abstractions
- ❌ **MCP Client/Server APIs** - Missing high-level MCP operations

## Implementation Plan

### Phase 1: Core MCP Message Types (Week 1)
- Implement MCP-specific message structures
- Resource management messages (`resources/list`, `resources/read`, `resources/subscribe`)
- Tool execution messages (`tools/list`, `tools/call`)  
- Prompt messages (`prompts/list`, `prompts/get`)
- Capability negotiation messages (`initialize`, capability structs)
- Logging and progress messages

### Phase 2: MCP Client API (Week 2)
- High-level MCP client trait and implementation
- Resource discovery and subscription management
- Tool discovery and execution with safety controls
- Prompt template management
- Connection lifecycle with capability negotiation
- Error handling with MCP-specific error types

### Phase 3: MCP Server API (Week 3)
- High-level MCP server trait and implementation
- Resource provider abstractions
- Tool executor abstractions with safety frameworks
- Prompt template provider abstractions
- Server-initiated client communication (sampling)
- Request routing and handler registration

### Phase 4: Integration & Testing (Week 4)
- Complete integration testing with real MCP scenarios
- Example implementations (file system server, simple tools)
- Documentation and usage guides
- Performance validation of full MCP stack

## Progress Tracking

**Overall Status:** planning_complete - 5%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 8.1 | MCP message type definitions | planning_complete | 2025-08-06 | Core protocol messages - Technical plan documented |
| 8.2 | Resource management API | pending | 2025-08-06 | Resource providers and clients |
| 8.3 | Tool execution framework | pending | 2025-08-06 | Tool discovery, execution, safety |
| 8.4 | Prompt template system | pending | 2025-08-06 | Template management and completion |
| 8.5 | MCP client high-level API | pending | 2025-08-06 | Client-side MCP operations |
| 8.6 | MCP server high-level API | pending | 2025-08-06 | Server-side MCP operations |
| 8.7 | Capability negotiation | pending | 2025-08-06 | Runtime capability management |
| 8.8 | Integration testing | pending | 2025-08-06 | End-to-end MCP scenarios |

## Progress Log

### 2025-08-06 - Task Creation & Phase 1 Technical Planning
- **Critical Need Identified**: High-level MCP abstractions missing despite excellent foundation
- **Strategic Importance**: This is the key blocker preventing real MCP tool development
- **Foundation Ready**: Outstanding JSON-RPC infrastructure (8.5+ GiB/s performance) ready to support MCP layer
- **User Impact**: Without this layer, developers must manually construct MCP messages
- **Implementation Readiness**: All dependencies complete, architecture documented, ready for immediate development
- **Phase 1 Planning Complete**: Detailed technical implementation plan documented with 7-day timeline
- **Architecture Decision**: Implement in `src/shared/protocol/` leveraging existing JsonRpcMessage trait
- **Quality Strategy**: 30+ tests, specification compliance, performance validation
- **Integration Plan**: Seamless integration with existing correlation and transport systems

## Expected API Preview

Based on our documentation planning, the target API should look like:

```rust
// MCP Client API (Target)
use airs_mcp::prelude::*;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create MCP client with STDIO transport
    let transport = StdioTransport::new().await?;
    let mut client = McpClient::new(transport).await?;
    
    // Connect with capability negotiation
    client.connect().await?;
    
    // High-level MCP operations
    let resources = client.list_resources().await?;
    let tools = client.list_tools().await?;
    let prompts = client.list_prompts().await?;
    
    // Execute tool with safety controls
    let result = client.execute_tool("file_read", json!({
        "path": "/path/to/file"
    })).await?;
    
    // Subscribe to resource changes
    client.subscribe_to_resource("file://config").await?;
    
    client.disconnect().await?;
    Ok(())
}

// MCP Server API (Target)
#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServerBuilder::new()
        .add_resource_provider(FileSystemProvider::new("/docs"))
        .add_tool_executor(ShellToolExecutor::new())
        .add_prompt_provider(TemplateProvider::new())
        .build()?;
    
    let transport = StdioTransport::new().await?;
    server.serve(transport).await?;
    Ok(())
}
```

## Technical Foundation Assessment

**Strengths (Ready for MCP Layer):**
- ✅ **Enterprise-Grade Performance**: 8.5+ GiB/s throughput, sub-microsecond latencies
- ✅ **Production Reliability**: 195+ tests, zero warnings, comprehensive error handling
- ✅ **Excellent Architecture**: Clean layered design with proper abstractions
- ✅ **Complete JSON-RPC**: Full bidirectional communication support
- ✅ **Transport Ready**: STDIO transport working (primary MCP transport)

**Implementation Requirements:**
- 🎯 **MCP Message Types**: Structured representations of MCP protocol messages
- 🎯 **High-Level APIs**: Developer-friendly interfaces for MCP operations
- 🎯 **Capability System**: Runtime feature negotiation and validation
- 🎯 **Safety Framework**: Tool execution with approval workflows
- 🎯 **Resource Management**: Discovery, subscription, and access patterns

## Success Criteria

1. **Developer Experience**: Simple, intuitive API for common MCP operations
2. **Protocol Compliance**: 100% adherence to MCP specification 
3. **Performance**: Maintain exceptional performance characteristics of foundation
4. **Safety**: Robust error handling and security controls
5. **Real Tool Development**: Enable immediate creation of production MCP tools

## Strategic Impact

This task is **CRITICAL** because:
- It's the final missing piece for production MCP tool development
- The foundation is exceptional and ready to support the MCP layer
- Users need this layer to avoid manual message construction
- It transforms the library from "infrastructure" to "complete toolkit"
- Multiple users have indicated need for high-level MCP abstractions

**Recommendation**: Prioritize this task immediately as the highest impact development effort.

## Phase 1 Detailed Implementation Plan (Week 1)

### Strategic Foundation
**Location & Module Structure:**
```
crates/airs-mcp/src/
├── base/           # ✅ JSON-RPC 2.0 Foundation (Complete)
├── correlation/    # ✅ Request correlation (Complete)  
├── transport/      # ✅ Transport abstraction (Complete)
├── integration/    # ✅ High-level client (Complete)
└── shared/
    └── protocol/   # 🎯 NEW: MCP Protocol Layer
        ├── mod.rs
        ├── messages/
        │   ├── mod.rs
        │   ├── initialization.rs
        │   ├── resources.rs
        │   ├── tools.rs
        │   ├── prompts.rs
        │   └── capabilities.rs
        ├── types/
        │   ├── mod.rs
        │   ├── common.rs
        │   └── content.rs
        └── errors.rs
```

### Technical Design Decisions
**Leverage Existing Excellence:**
- **Reuse JsonRpcMessage trait**: All MCP messages implement existing trait for consistent serialization
- **Integrate with correlation system**: MCP request/response correlation uses proven CorrelationManager
- **Follow established patterns**: Same error handling, testing, and documentation standards

**MCP-Specific Enhancements:**
- **Type-safe message construction**: Prevent invalid MCP protocol messages at compile time
- **Capability-driven features**: Runtime feature availability based on negotiated capabilities
- **Bidirectional support**: Both client→server and server→client message flows

### Implementation Timeline (Week 1)

#### Day 1-2: Foundation & Core Types
**Core Protocol Types** (`src/shared/protocol/types/common.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion(pub String);

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self("2025-06-18".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}
```

**Content System** (`src/shared/protocol/types/content.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { 
        data: String,  // base64 encoded
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { 
        resource: String,  // URI
        text: Option<String>,
        mime_type: Option<String>,
    },
}
```

#### Day 3-4: Capability System
**Capability Definitions** (`src/shared/protocol/messages/capabilities.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    pub experimental: Option<serde_json::Value>,
    pub sampling: Option<SamplingCapabilities>,
    pub roots: Option<RootsCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    pub experimental: Option<serde_json::Value>,
    pub logging: Option<LoggingCapabilities>,
    pub prompts: Option<PromptCapabilities>,
    pub resources: Option<ResourceCapabilities>,
    pub tools: Option<ToolCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}
```

#### Day 5: Initialization Messages
**Protocol Initialization** (`src/shared/protocol/messages/initialization.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    pub protocol_version: ProtocolVersion,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    pub protocol_version: ProtocolVersion,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
    pub instructions: Option<String>,
}

impl InitializeRequest {
    pub fn to_jsonrpc_request(&self, id: RequestId) -> Result<JsonRpcRequest, serde_json::Error> {
        JsonRpcRequest::new(
            "initialize",
            Some(serde_json::to_value(self)?),
            id,
        )
    }
}

impl JsonRpcMessage for InitializeRequest {}
impl JsonRpcMessage for InitializeResponse {}
```

#### Day 6-7: Resource Messages
**Resource Protocol Messages** (`src/shared/protocol/messages/resources.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResponse {
    pub resources: Vec<Resource>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceRequest {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResponse {
    pub contents: Vec<Content>,
}

impl JsonRpcMessage for ListResourcesRequest {}
impl JsonRpcMessage for ListResourcesResponse {}
```

### Integration with Existing Architecture
**Leverage Current Excellence:**
```rust
// Example integration with existing JsonRpcClient
impl JsonRpcClient {
    pub async fn mcp_initialize(
        &self,
        capabilities: ClientCapabilities,
        client_info: ClientInfo,
    ) -> Result<InitializeResponse, crate::integration::error::IntegrationError> {
        let request = InitializeRequest::new(capabilities, client_info);
        let id = RequestId::new_string("mcp-init");
        let jsonrpc_request = request.to_jsonrpc_request(id)?;
        
        let response = self.call(jsonrpc_request).await?;
        let init_response: InitializeResponse = serde_json::from_value(
            response.result.ok_or(IntegrationError::MissingResult)?
        )?;
        
        Ok(init_response)
    }
}
```

### Quality Assurance Strategy
**Testing Approach:**
- **Unit Tests**: Each message type with round-trip serialization validation
- **Integration Tests**: JSON-RPC integration with existing correlation system
- **Specification Compliance**: Validate against MCP protocol specification
- **Performance Tests**: Ensure message serialization maintains 8.5+ GiB/s performance

**Error Handling:**
- **Reuse Existing Patterns**: Leverage proven structured error system
- **MCP-Specific Errors**: Add protocol-specific error variants
- **Graceful Degradation**: Handle capability mismatches and version incompatibilities

### Expected Week 1 Deliverables
- ✅ **Complete MCP message type system** (initialization, resources, tools, prompts)
- ✅ **Capability negotiation framework** with type-safe capability definitions
- ✅ **Seamless JSON-RPC integration** leveraging existing foundation
- ✅ **Comprehensive test suite** (30+ tests) with specification compliance
- ✅ **Performance validation** maintaining exceptional throughput characteristics

### Strategic Impact
- **Developer Experience**: Simple, type-safe MCP message construction
- **Protocol Compliance**: 100% MCP specification adherence
- **Foundation for Phase 2**: Ready for high-level client/server API implementation
- **Production Readiness**: Enterprise-grade quality matching existing components
