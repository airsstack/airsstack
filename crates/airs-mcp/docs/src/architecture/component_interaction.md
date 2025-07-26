# Component Interaction Patterns

## Capability Negotiation Sequence

```rust,ignore
// Capability negotiation orchestration
pub struct CapabilityNegotiator {
    server_capabilities: ServerCapabilities,
    client_requirements: ClientRequirements,
}

impl CapabilityNegotiator {
    pub async fn negotiate(
        &self,
        transport: &dyn BidirectionalTransport,
    ) -> Result<NegotiatedCapabilities, NegotiationError> {
        // 1. Client sends initialize request
        let init_request = JsonRpcRequest {
            method: "initialize".to_string(),
            params: InitializeParams {
                protocol_version: PROTOCOL_VERSION,
                capabilities: self.client_requirements.to_capabilities(),
                client_info: self.get_client_info(),
            },
            id: RequestId::generate(),
        };
        
        // 2. Server responds with capabilities
        let response = self.send_request(init_request, transport).await?;
        let server_caps: InitializeResult = response.extract_result()?;
        
        // 3. Compute intersection of capabilities
        let negotiated = self.compute_capability_intersection(
            &self.client_requirements,
            &server_caps.capabilities,
        )?;
        
        // 4. Send initialized notification
        let initialized = JsonRpcNotification {
            method: "notifications/initialized".to_string(),
            params: serde_json::Value::Null,
        };
        transport.send_message(JsonRpcMessage::from(initialized)).await?;
        
        Ok(negotiated)
    }
}
```

## Security Integration Pattern

```rust,ignore
// Security concerns integrated at message processing level
pub struct SecureMessageProcessor {
    base_processor: JsonRpcProcessor,
    authenticator: Box<dyn Authenticator>,
    authorizer: Box<dyn Authorizer>,
    audit_logger: Box<dyn AuditLogger>,
}

impl SecureMessageProcessor {
    pub async fn process_message(
        &self,
        message: JsonRpcMessage,
        security_context: &SecurityContext,
    ) -> Result<Option<JsonRpcMessage>, ProcessingError> {
        // 1. Authentication check
        self.authenticator.verify_message(&message, security_context).await?;
        
        // 2. Authorization check
        self.authorizer.check_permission(&message, security_context).await?;
        
        // 3. Audit logging (pre-execution)
        self.audit_logger.log_message_received(&message, security_context).await?;
        
        // 4. Process message
        let result = self.base_processor.process_message(message, &security_context.into()).await;
        
        // 5. Audit logging (post-execution)
        self.audit_logger.log_message_processed(&result, security_context).await?;
        
        result
    }
}
```

## Resource Subscription Management

```rust,ignore
// Real-time subscription management with cleanup
pub struct SubscriptionManager {
    subscriptions: DashMap<SubscriptionId, Subscription>,
    resource_providers: Vec<Box<dyn ResourceProvider>>,
    notification_sender: mpsc::UnboundedSender<ResourceNotification>,
}

impl SubscriptionManager {
    pub async fn subscribe_to_resource(
        &self,
        uri: &str,
        connection_id: &str,
    ) -> Result<SubscriptionId, SubscriptionError> {
        let subscription_id = SubscriptionId::generate();
        let subscription = Subscription {
            id: subscription_id.clone(),
            uri: uri.to_string(),
            connection_id: connection_id.to_string(),
            created_at: Utc::now(),
        };
        
        // Register subscription
        self.subscriptions.insert(subscription_id.clone(), subscription);
        
        // Setup resource watching
        for provider in &self.resource_providers {
            if provider.supports_uri(uri) {
                provider.watch_resource(uri, subscription_id.clone()).await?;
                break;
            }
        }
        
        Ok(subscription_id)
    }
    
    pub async fn handle_resource_change(
        &self,
        uri: &str,
        change: ResourceChange,
    ) -> Result<(), SubscriptionError> {
        // Find all subscriptions for this resource
        let affected_subscriptions: Vec<_> = self.subscriptions
            .iter()
            .filter(|entry| entry.value().uri == uri)
            .map(|entry| entry.key().clone())
            .collect();
        
        // Send notifications
        for subscription_id in affected_subscriptions {
            let notification = ResourceNotification {
                subscription_id,
                uri: uri.to_string(),
                change: change.clone(),
            };
            
            self.notification_sender.send(notification)?;
        }
        
        Ok(())
    }
}
```
