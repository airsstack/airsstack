# MCP Protocol Deep Dive

## Protocol Fundamentals

### JSON-RPC 2.0 Foundation

MCP is built on JSON-RPC 2.0 with specific extensions and constraints:

```rust,ignore
// Core message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc")]
pub enum JsonRpcMessage {
    #[serde(rename = "2.0")]
    V2_0(JsonRpcV2Message),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcV2Message {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
    // MCP extension: batch operations (2025-03-26)
    BatchRequest(Vec<JsonRpcRequest>),
    BatchResponse(Vec<JsonRpcResponse>),
}
```

### Bidirectional Communication Model

Unlike traditional JSON-RPC implementations, MCP requires true bidirectional communication:

```rust,ignore
// Client → Server (traditional)
client.send_request("resources/list", params).await?;

// Server → Client (MCP-specific)  
server.send_request("sampling/createMessage", sampling_request).await?;
```

Implementation Challenge: Both client and server must handle:

- Outgoing requests (with response correlation)
- Incoming requests (with request processing)
- Concurrent request streams in both directions
