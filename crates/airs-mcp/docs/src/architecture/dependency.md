# Dependency Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     Server      │    │     Client      │    │    Security     │
│    Features     │    │    Features     │    │  (Cross-cutting)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                     ┌─────────────────┐
                     │   Lifecycle     │
                     │   Management    │
                     └─────────────────┘
                                 │
                     ┌─────────────────┐
                     │  Base Protocol  │
                     │ (JSON-RPC 2.0)  │
                     └─────────────────┘
                                 │
                     ┌─────────────────┐
                     │ Shared Types &  │
                     │    Utilities    │
                     └─────────────────┘
```

Dependency Rules:

- No Circular Dependencies: Enforced through module structure
- Inward Dependencies Only: Higher layers depend on lower layers
- Interface-Based Coupling: Domains communicate through traits
- Shared Types at Bottom: Common data structures in shared module
