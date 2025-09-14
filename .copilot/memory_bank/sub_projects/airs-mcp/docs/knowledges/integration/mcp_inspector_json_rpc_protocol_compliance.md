# MCP Inspector JSON-RPC Protocol Compliance - Integration Findings

**Date Created**: 2025-09-14  
**Status**: Complete Success ✅  
**Complexity**: High  
**Priority**: Critical  
**Category**: Integration  
**Tags**: `mcp-inspector`, `json-rpc`, `notifications`, `protocol-compliance`, `oauth2`

## Problem Summary

MCP Inspector was throwing schema validation errors when connecting to our OAuth2-enabled MCP server:

```javascript
ZodError: [
  {
    "code": "unrecognized_keys",
    "keys": ["message", "success"],
    "path": [],
    "message": "Unrecognized key(s) in object: 'message', 'success'"
  }
]
```

## Root Cause Analysis

### 1. **JSON-RPC Notification vs Request Confusion**
- **Issue**: Our server was treating `logging/setLevel` as a request-response method
- **Root Cause**: Inconsistency between internal client expectations and MCP specification
- **Impact**: MCP Inspector expected standard MCP protocol compliance

### 2. **Custom Response Format vs Standard Protocol**
- **Issue**: Server was returning custom `SetLoggingResponse` structure:
  ```json
  {
    "success": true,
    "message": "Logging configuration updated"
  }
  ```
- **Expected**: Empty object per MCP specification: `{}`

### 3. **Protocol Version Mismatch**
- **Previous Issue**: Protocol version was `2024-11-05`, MCP Inspector expected `2025-06-18`
- **Resolution**: Updated to current MCP protocol version

## Solution Implementation

### 1. **Correct JSON-RPC Message Handling**

**File**: `crates/airs-mcp/src/transport/adapters/http/mcp_request_handler.rs`

```rust
// Added JsonRpcMessage enum handling for proper request/notification distinction
use crate::protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, /* ... */};

impl McpRequestHandler for AxumMcpRequestHandler {
    async fn handle_mcp_request(&self, /* ... */) -> Result<HttpResponse, HttpEngineError> {
        let message: JsonRpcMessage = serde_json::from_slice(&request_data)?;
        
        match message {
            JsonRpcMessage::Request(request) => {
                // Handle request and send JSON-RPC response
                let result = self.handle_mcp_method(&session_id, request).await?;
                let response = JsonRpcResponse { /* standard response */ };
                // Return JSON response
            }
            JsonRpcMessage::Notification(notification) => {
                // Handle notification but don't send response
                self.handle_mcp_method(&session_id, converted_request).await?;
                // Return 204 No Content (correct for notifications)
                Ok(HttpResponse {
                    body: vec![],
                    status: 204,
                    headers: std::collections::HashMap::new(),
                    mode: ResponseMode::Json,
                })
            }
            JsonRpcMessage::Response(_) => {
                Err(/* Server shouldn't receive responses */)
            }
        }
    }
}
```

### 2. **MCP Protocol Compliant Logging Response**

```rust
pub async fn handle_set_logging(&self, /* ... */) -> Result<Value, HttpEngineError> {
    if let Some(ref handler) = self.logging_handler {
        // Parse and validate request
        let logging_request: SetLoggingRequest = /* ... */;
        
        match handler.set_logging(LoggingConfig { level: logging_request.level }).await {
            Ok(_success) => {
                // Return empty object per MCP specification for logging/setLevel
                // This follows the pattern from SSE handlers and MCP specification
                Ok(serde_json::json!({}))  // ✅ Standard MCP format
            }
            Err(e) => Err(HttpEngineError::Engine {
                message: format!("Logging handler error: {e}"),
            }),
        }
    }
}
```

### 3. **ServerCapabilities Schema Compliance**

**Fixed experimental field schema validation**:

```rust
// Before (causing Zod validation error)
experimental: None  // Serialized as null

// After (MCP Inspector compatible)
experimental: Some(json!({}))  // Serialized as empty object
```

## Key MCP Protocol Learnings

### 1. **JSON-RPC 2.0 Notification Semantics**
- **Notifications**: Messages without `id` field, NO response expected
- **Requests**: Messages with `id` field, response required
- **HTTP Status**: 204 No Content for notifications, 200 OK for request responses

### 2. **MCP Method Classification**
```rust
// Notification methods (no response)
- "notifications/initialized"
- "logging/setLevel"  // ⚠️ Key finding: This is a notification!

// Request methods (expect response)  
- "initialize"
- "resources/list"
- "tools/list"
- "prompts/list"
```

### 3. **Schema Validation Requirements**
- **Experimental fields**: Must be objects `{}`, not `null`
- **Protocol version**: Must match exactly (`2025-06-18`)
- **Response format**: Must follow JSON-RPC 2.0 specification exactly

## Testing Validation

### Before Fix
```javascript
// MCP Inspector error
ZodError: [
  {
    "code": "unrecognized_keys", 
    "keys": ["message", "success"],
    "path": [],
    "message": "Unrecognized key(s) in object: 'message', 'success'"
  }
]
```

### After Fix
```
✅ OAuth2 authorization flow: SUCCESS
✅ MCP protocol initialization: SUCCESS  
✅ All MCP operations working: resources/list, tools/list, prompts/list
✅ Schema validation: PASSED
✅ No Zod validation errors
```

## Production Implementation Guidelines

### 1. **Always Follow MCP Specification**
- Check official MCP specification for method classification
- Distinguish between notifications and requests properly
- Return correct response formats per specification

### 2. **Schema Compliance**
- Use empty objects `{}` instead of `null` for optional object fields
- Validate protocol versions match exactly
- Test with multiple MCP clients (not just internal client)

### 3. **JSON-RPC 2.0 Compliance**
- Implement proper notification handling (no response)
- Use correct HTTP status codes (204 for notifications, 200 for responses)
- Maintain `jsonrpc: "2.0"` in all messages

## Architecture Impact

### HTTP Transport Layer
```rust
// Clean separation of notification vs request handling
match message {
    JsonRpcMessage::Request(req) => {
        // Process and respond
        Ok(HttpResponse::json(response_body))
    }
    JsonRpcMessage::Notification(notif) => {
        // Process but don't respond  
        Ok(HttpResponse { status: 204, body: vec![], /* ... */ })
    }
}
```

### Internal Client Compatibility
- Our internal `McpClient` still works correctly
- Server now supports both internal clients AND external MCP tools
- Clean separation between internal protocol extensions and standard MCP

## Success Metrics

### Technical Validation
- ✅ **Zero Zod validation errors** from MCP Inspector
- ✅ **Complete OAuth2 + MCP integration** working end-to-end
- ✅ **Protocol compliance** verified with external MCP client
- ✅ **Backward compatibility** maintained with internal clients

### Integration Success  
- ✅ **MCP Inspector connection**: Successful after OAuth2 authentication
- ✅ **All MCP operations**: resources, tools, prompts, logging all functional
- ✅ **Schema validation**: ServerCapabilities and all responses pass validation
- ✅ **Cross-client compatibility**: Works with both internal and external MCP clients

## Future Development Notes

### 1. **Method Classification Reference**
Always check the official MCP specification to determine if a method should be:
- **Notification** (no `id`, no response): `logging/setLevel`, `notifications/initialized`
- **Request** (has `id`, expects response): `initialize`, `resources/list`, etc.

### 2. **Testing Strategy**
- Test with multiple MCP clients (MCP Inspector, Claude Desktop, internal clients)
- Validate schema compliance with external tools
- Verify both notification and request handling paths

### 3. **Schema Evolution**
- Monitor MCP specification updates for protocol version changes
- Keep experimental fields as empty objects, not null
- Maintain strict JSON-RPC 2.0 compliance

---

**Final Result**: Perfect OAuth2 + MCP Inspector integration with full protocol compliance and zero validation errors. The server now supports both internal clients and external MCP tools seamlessly.