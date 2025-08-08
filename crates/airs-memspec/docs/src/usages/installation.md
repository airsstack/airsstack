# Installation & Setup

This guide covers everything you need to know about installing and setting up airs-memspec for Multi-Project Memory Bank management.

## Installation Options

### Option 1: From Cargo (Recommended)

```bash
# Install from crates.io (when published)
cargo install airs-memspec

# Verify installation
airs-memspec --version
```

### Option 2: From Source

```bash
# Clone the AIRS workspace
git clone https://github.com/rstlix0x0/airs.git
cd airs

# Build and install
cargo build --release --bin airs-memspec
cargo install --path crates/airs-memspec

# Verify installation
airs-memspec --help
```

### Option 3: Development Build

```bash
# For development and testing
cd /path/to/airs/workspace
cargo run --bin airs-memspec -- --help

# Create an alias for convenience
alias airs-memspec="cargo run --bin airs-memspec --"
```

## Initial Setup

### 1. Verify Installation

```bash
# Check version and basic functionality
airs-memspec --version
airs-memspec --help

# Expected output:
# airs-memspec 0.1.0
# AI-focused memory bank management tool
```

### 2. Choose Your Workspace

Navigate to your development workspace root directory:

```bash
# Example workspace structures
cd /path/to/your/workspace      # General workspace
cd /path/to/your/monorepo       # Monorepo structure  
cd /path/to/your/project        # Single project
```

### 3. Install GitHub Copilot Instructions

```bash
# Basic installation (recommended)
airs-memspec install

# Custom path installation
airs-memspec install --path /path/to/copilot/config

# Force overwrite existing files
airs-memspec install --force

# Install specific template
airs-memspec install --template multi-project
```

**Expected Results:**
- Creates `.copilot/instructions/` directory
- Installs Multi-Project Memory Bank instruction templates
- Ready for GitHub Copilot integration

### 4. Verify Setup

```bash
# Check installation status
airs-memspec status

# Expected output if no memory bank exists yet:
# ❌ No memory bank structure found
# 💡 Use GitHub Copilot with custom instructions to create it
```

## GitHub Copilot Configuration

### VS Code Setup

1. **Open VS Code Settings**
   - Press `Cmd/Ctrl + ,` to open settings
   - Search for "copilot"

2. **Configure Custom Instructions Path**
   ```json
   // In settings.json
   {
     "github.copilot.chat.localeOverride": "en",
     "github.copilot.enable": {
       "*": true,
       "yaml": true,
       "plaintext": true,
       "markdown": true
     }
   }
   ```

3. **Restart VS Code**
   - Completely restart VS Code to apply changes
   - Verify Copilot can access the instructions

### Alternative Editors

For other editors supporting GitHub Copilot:

```bash
# Ensure instructions are in accessible location
ls -la .copilot/instructions/

# Verify file permissions
chmod 644 .copilot/instructions/*.md
```

## Creating Your First Memory Bank

### Using GitHub Copilot

With the instructions installed, interact with GitHub Copilot:

```
You: "Create a memory bank structure for this workspace"

Copilot will:
1. Analyze your workspace structure
2. Create .copilot/memory_bank/ directory
3. Set up workspace and sub-project files
4. Initialize task tracking structures
```

### Manual Setup (Optional)

If you prefer manual setup:

```bash
# Create memory bank structure
mkdir -p .copilot/memory_bank/{workspace,sub_projects,context_snapshots}

# Create workspace files
touch .copilot/memory_bank/current_context.md
touch .copilot/memory_bank/workspace/project_brief.md
touch .copilot/memory_bank/workspace/shared_patterns.md
```

### Verify Memory Bank Creation

```bash
# Check memory bank status
airs-memspec status --workspace

# Expected output:
# 🏢 Your Workspace
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Status        Memory Bank Active
# Projects      X discovered
# Last Updated  Just now
```

## Troubleshooting Installation

### Common Issues

#### Permission Errors
```bash
# Fix permission issues
chmod +x $(which airs-memspec)
sudo chown -R $USER:$USER ~/.cargo/bin/
```

#### PATH Issues
```bash
# Add Cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify PATH
which airs-memspec
```

#### Build Errors
```bash
# Update Rust toolchain
rustup update

# Clean build cache
cargo clean
cargo build --release --bin airs-memspec
```

### Verification Commands

```bash
# Comprehensive installation check
airs-memspec --version                    # Version info
airs-memspec --help                       # Command help
airs-memspec status                       # Basic functionality
ls -la .copilot/instructions/             # Instruction files
ls -la .copilot/memory_bank/              # Memory bank structure
```

## Next Steps

After successful installation:

1. **Learn Essential Workflows**: Continue to [Essential Workflows](./workflows.md)
2. **Explore Commands**: Review [Command Reference](./commands.md)  
3. **Set Up Integration**: Configure [Integration Patterns](./integration.md)

## Advanced Installation

### System-Wide Installation

```bash
# Install for all users (requires sudo)
sudo cargo install --root /usr/local airs-memspec

# Verify system installation
which airs-memspec
/usr/local/bin/airs-memspec --version
```

### Docker Usage

```bash
# Use within Docker environment
FROM rust:latest
RUN cargo install airs-memspec
WORKDIR /workspace
CMD ["airs-memspec", "status"]
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Install airs-memspec
  run: cargo install airs-memspec

- name: Verify memory bank
  run: airs-memspec status --workspace
```
