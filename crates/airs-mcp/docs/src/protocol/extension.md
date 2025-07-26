# Protocol Extensions & Advanced Features

## Progress Tracking

```rust,ignore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressToken {
    pub progress: f64,        // 0.0 to 1.0
    pub total: Option<u64>,   // Total work units
    pub completed: Option<u64>, // Completed work units
}

// Long-running operations support progress updates
"tools/call" with progress_token → periodic progress notifications
```

## Cancellation Support

```rust,ignore
// Operation cancellation through notifications
"notifications/cancelled" { request_id: "operation-123" }

// Implementation requirement: All long-running operations must be cancellable
#[async_trait]
pub trait CancellableOperation {
    async fn execute(&self, cancellation_token: CancellationToken) -> Result<T, OperationError>;
}
```

## Pagination System

```rust,ignore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}
```
