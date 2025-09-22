# Configuration & Deployment

## **Configuration Management**

### **Hierarchical Configuration System**
```toml
# Global configuration: ~/.config/airs-mcpserver-fs/config.toml
[server]
name = "airs-mcpserver-fs"
version = "1.0.0"
transport = "stdio"

[performance]
max_concurrent_operations = 10
cache_size_mb = 50
temp_directory = "/tmp/airs-mcpserver-fs"

[logging]
level = "info"
file = "~/.config/airs-mcpserver-fs/logs/airs-mcpserver-fs.log"
max_size_mb = 100
max_files = 10

# Project-specific configuration: ./.airs-mcpserver-fs.toml
[project]
name = "my-awesome-project"
root_path = "./"
exclude_patterns = ["node_modules/**", ".git/**", "target/**"]

[security]
# Override global security settings for this project
allowed_write_paths = ["./src/**", "./docs/**", "./tests/**"]
require_approval_for_writes = false  # Relaxed for development
```

### **Environment-Based Configuration**
```rust
#[derive(Deserialize)]
pub struct FsConfig {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
    pub project: Option<ProjectConfig>,
}

impl FsConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::load_global_config()?;
        
        // Override with project-specific config if present
        if let Ok(project_config) = Self::load_project_config() {
            config.merge_project_config(project_config)?;
        }
        
        // Override with environment variables
        config.apply_env_overrides()?;
        
        Ok(config)
    }
    
    fn load_global_config() -> Result<Self, ConfigError> {
        let config_path = dirs::config_dir()
            .ok_or(ConfigError::NoConfigDir)?
            .join("airs-mcpserver-fs")
            .join("config.toml");
            
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
}
```

## **Deployment Strategies**

### **Development Deployment**
```bash
# Install from source for development
git clone https://github.com/airsstack/airsstack.git
cd airs
cargo build --release --bin airs-mcpserver-fs

# Configure for Claude Desktop
# Add to Claude Desktop MCP configuration:
{
  "mcpServers": {
    "airs-mcpserver-fs": {
      "command": "path/to/airs-mcpserver-fs",
      "args": ["--config", "./.airs-mcpserver-fs.toml"]
    }
  }
}
```

### **Production Deployment**
```bash
# Install via cargo
cargo install airs-mcp-fs

# System-wide configuration
sudo mkdir -p /etc/airs-mcp-fs
sudo cp config.toml /etc/airs-mcp-fs/

# User configuration
mkdir -p ~/.config/airs-mcp-fs
cp user-config.toml ~/.config/airs-mcp-fs/config.toml

# Systemd service for enterprise environments
[Unit]
Description=AIRS MCP-FS Server
After=network.target

[Service]
Type=simple
User=mcp-fs
ExecStart=/usr/local/bin/airs-mcp-fs --daemon
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

