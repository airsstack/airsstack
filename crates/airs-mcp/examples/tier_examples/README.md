# Progressive Developer Experience Tiers

This directory contains examples demonstrating the four-tier progressive disclosure pattern for HTTP Transport configuration in airs-mcp. Each tier provides increasing levels of control and complexity, allowing developers to choose the appropriate level for their needs.

## 📚 Tier Overview

### Tier 1: Zero Configuration
- **File**: `tier1_zero_configuration.rs`
- **Target Audience**: Beginners, prototyping, quick starts
- **Key Features**: 
  - Minimal code required
  - Uses all defaults
  - Perfect for learning and testing
- **Example**:
  ```rust
  let transport = HttpTransportBuilder::<AxumHttpServer>::with_default()?
      .bind("127.0.0.1:8080".parse()?).await?
      .build().await?;
  ```

### Tier 2: Basic Configuration
- **File**: `tier2_basic_configuration.rs`
- **Target Audience**: Production apps with standard requirements
- **Key Features**:
  - Pre-configured engines for common patterns
  - Simple authentication setup
  - Proven configurations
- **Example**:
  ```rust
  let engine = AxumHttpServer::with_auth(auth_config)?;
  let transport = HttpTransportBuilder::with_engine(engine)?
      .bind("127.0.0.1:8080".parse()?).await?
      .build().await?;
  ```

### Tier 3: Advanced Configuration
- **File**: `tier3_advanced_configuration.rs`
- **Target Audience**: Advanced users, complex requirements
- **Key Features**:
  - Full builder pattern control
  - Custom middleware support
  - Complex authentication setups
  - Performance tuning
- **Example**:
  ```rust
  let transport = HttpTransportBuilder::with_configured_engine(|| {
      AxumHttpServer::builder()
          .with_oauth2_authorization(oauth2_config)
          .with_custom_middleware(middleware)
          .build()
  })?
  .configure_engine(|engine| { /* post-config */ })
  .bind("127.0.0.1:8080".parse()?).await?
  .build().await?;
  ```

### Tier 4: Expert Async
- **File**: `tier4_expert_async.rs`
- **Target Audience**: Expert users, distributed systems
- **Key Features**:
  - Async initialization patterns
  - Dynamic configuration loading
  - Service discovery integration
  - Multi-tenant support
- **Example**:
  ```rust
  let transport = HttpTransportBuilder::with_configured_engine_async(|| async {
      let oauth2_config = load_oauth2_config_from_db().await?;
      AxumHttpServer::builder()
          .with_oauth2_authorization(oauth2_config)
          .build()
  }).await?
  .bind("127.0.0.1:8080".parse()?).await?
  .build().await?;
  ```

## 🎯 Choosing the Right Tier

### Use Tier 1 When:
- ✅ Learning MCP fundamentals
- ✅ Quick prototyping
- ✅ Development environments
- ✅ Testing scenarios
- ✅ No authentication needed

### Use Tier 2 When:
- ✅ Production applications with standard auth
- ✅ Teams want proven configurations
- ✅ Clear security requirements
- ✅ OAuth2, API keys, or basic auth

### Use Tier 3 When:
- ✅ Need fine-grained control
- ✅ Complex middleware requirements
- ✅ Performance optimization needed
- ✅ Enterprise security policies
- ✅ Custom authentication flows

### Use Tier 4 When:
- ✅ Microservices architectures
- ✅ Database-driven configuration
- ✅ Service discovery required
- ✅ Multi-tenant applications
- ✅ Cloud-native deployments

## 🚀 Running the Examples

Each tier example can be run independently:

```bash
# Tier 1: Zero Configuration
cargo run --example tier1_zero_configuration

# Tier 2: Basic Configuration  
cargo run --example tier2_basic_configuration

# Tier 3: Advanced Configuration
cargo run --example tier3_advanced_configuration

# Tier 4: Expert Async
cargo run --example tier4_expert_async
```

## 🧪 Testing the Examples

All examples include comprehensive test suites:

```bash
# Test all tier examples
cargo test --example tier1_zero_configuration
cargo test --example tier2_basic_configuration
cargo test --example tier3_advanced_configuration
cargo test --example tier4_expert_async

# Or test all examples at once
cargo test tier_examples
```

## 📈 Progressive Learning Path

1. **Start with Tier 1** - Get familiar with basic concepts
2. **Move to Tier 2** - Learn authentication patterns
3. **Explore Tier 3** - Understand advanced configuration
4. **Master Tier 4** - Build distributed systems

Each tier builds upon the previous one, providing a natural learning progression while allowing developers to stop at the level that meets their needs.

## 🔧 Implementation Status

- ✅ **Tier 1**: Fully implemented with `with_default()` method
- ✅ **Tier 2**: Builder patterns ready, auth adapters available
- ✅ **Tier 3**: `with_configured_engine()` method implemented
- ✅ **Tier 4**: `with_configured_engine_async()` method implemented

## 🎓 Best Practices

### Code Organization
- Keep tier progression clear in documentation
- Provide migration paths between tiers
- Include error handling examples for each tier

### Performance Considerations
- Tier 1 optimized for developer experience
- Tier 2 optimized for standard production use
- Tier 3 optimized for specific performance requirements
- Tier 4 optimized for distributed system patterns

### Security Guidelines
- Tier 1: Development only (no auth)
- Tier 2: Production-ready authentication
- Tier 3: Enterprise security policies
- Tier 4: Zero-trust distributed security

---

*This tier system implements Phase 5.3 of TASK-030, providing progressive developer experience patterns that scale from simple to expert usage while maintaining consistent APIs and clear upgrade paths.*