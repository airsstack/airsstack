# Troubleshooting

Comprehensive problem resolution guide for airs-memspec issues, error scenarios, and debugging techniques.

## Quick Diagnosis

### Health Check Commands

Run these commands first to identify the problem scope:

```bash
# 1. Basic functionality test
airs-memspec --version

# 2. Current workspace status
airs-memspec status --workspace 2>&1

# 3. Memory bank structure check
ls -la .copilot/memory_bank/ 2>&1

# 4. GitHub Copilot instructions verification  
ls -la .copilot/instructions/ 2>&1

# 5. Task system health
airs-memspec tasks list --filter all --quiet 2>&1
```

### Error Pattern Recognition

| Error Pattern | Likely Cause | Quick Fix |
|---------------|--------------|-----------|
| `No memory bank structure found` | Missing or incomplete installation | Run `airs-memspec install` |
| `Permission denied` | File system permissions | Check directory permissions |
| `Project 'X' not found` | Invalid project name or missing context | Verify with `airs-memspec status --workspace` |
| `Command not found: airs-memspec` | Installation issue | Reinstall with `cargo install airs-memspec` |
| `Failed to parse current_context.md` | Corrupted memory bank file | Restore from backup or recreate |

## Installation Issues

### Command Not Found

**Problem:** `airs-memspec: command not found`

**Diagnosis:**
```bash
# Check if Rust/Cargo is installed
cargo --version

# Check if airs-memspec is in PATH
echo $PATH | grep -q "$HOME/.cargo/bin" && echo "Cargo bin in PATH" || echo "Cargo bin NOT in PATH"

# Check if binary exists
ls -la ~/.cargo/bin/airs-memspec
```

**Solutions:**

1. **Install airs-memspec:**
   ```bash
   cargo install airs-memspec
   ```

2. **Add Cargo bin to PATH:**
   ```bash
   # For bash/zsh
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   
   # For fish
   set -U fish_user_paths $HOME/.cargo/bin $fish_user_paths
   ```

3. **Use full path temporarily:**
   ```bash
   ~/.cargo/bin/airs-memspec --version
   ```

### Permission Errors

**Problem:** `Permission denied to write to .copilot/instructions/`

**Diagnosis:**
```bash
# Check directory permissions
ls -ld .copilot/
ls -ld .copilot/instructions/ 2>/dev/null

# Check if directory exists and is writable
test -w .copilot/ && echo "Writable" || echo "Not writable"

# Check for readonly file system
mount | grep "$(df . | tail -1 | awk '{print $1}')"
```

**Solutions:**

1. **Fix directory permissions:**
   ```bash
   chmod 755 .copilot/
   chmod 755 .copilot/instructions/ 2>/dev/null || true
   ```

2. **Create missing directories:**
   ```bash
   mkdir -p .copilot/instructions/
   ```

3. **Change ownership (if needed):**
   ```bash
   sudo chown -R $USER:$USER .copilot/
   ```

4. **Use different installation path:**
   ```bash
   airs-memspec install --path ~/memspec-config
   ```

### Cargo Installation Failures

**Problem:** `cargo install airs-memspec` fails

**Common Errors and Solutions:**

1. **Network connectivity:**
   ```bash
   # Test network access
   curl -I https://crates.io
   
   # Use alternative registry
   cargo install --registry crates-io airs-memspec
   ```

2. **Compilation errors:**
   ```bash
   # Update Rust toolchain
   rustup update
   
   # Clear Cargo cache
   cargo clean
   rm -rf ~/.cargo/registry/cache/
   
   # Install with verbose output
   cargo install airs-memspec --verbose
   ```

3. **Disk space issues:**
   ```bash
   # Check available space
   df -h ~/.cargo/
   
   # Clean up Cargo cache
   cargo cache --autoclean
   ```

4. **Dependency conflicts:**
   ```bash
   # Force reinstall
   cargo install airs-memspec --force
   
   # Install specific version
   cargo install airs-memspec --version 0.1.0
   ```

## Memory Bank Issues

### Missing Memory Bank Structure

**Problem:** `No memory bank structure found in current directory`

**Diagnosis:**
```bash
# Check current directory
pwd

# Look for memory bank
find . -name "current_context.md" -type f

# Check if in subdirectory
find ../.. -name "current_context.md" -type f 2>/dev/null | head -5
```

**Solutions:**

1. **Initialize memory bank:**
   ```bash
   airs-memspec install
   ```

2. **Navigate to correct directory:**
   ```bash
   # Find workspace root
   while [ ! -f "current_context.md" ] && [ "$PWD" != "/" ]; do
     cd ..
     if [ -f ".copilot/memory_bank/current_context.md" ]; then
       echo "Found workspace at: $PWD"
       break
     fi
   done
   ```

3. **Specify workspace path:**
   ```bash
   airs-memspec status --path /path/to/workspace
   ```

### Corrupted Memory Bank Files

**Problem:** Memory bank files are unreadable or malformed

**Diagnosis:**
```bash
# Check file integrity
file .copilot/memory_bank/current_context.md

# Look for binary data in text files
grep -P "[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\xFF]" .copilot/memory_bank/current_context.md

# Check file sizes
find .copilot/memory_bank -name "*.md" -size 0
find .copilot/memory_bank -name "*.md" -size +1M
```

**Recovery Steps:**

1. **Create backup of current state:**
   ```bash
   cp -r .copilot/memory_bank .copilot/memory_bank.corrupted.$(date +%Y%m%d_%H%M%S)
   ```

2. **Restore from git history:**
   ```bash
   git log --oneline --follow .copilot/memory_bank/current_context.md
   git checkout HEAD~1 -- .copilot/memory_bank/current_context.md
   ```

3. **Recreate core files:**
   ```bash
   # Recreate current_context.md
   echo "# Current Context" > .copilot/memory_bank/current_context.md
   echo "" >> .copilot/memory_bank/current_context.md
   echo "**Active Sub-Project:** main" >> .copilot/memory_bank/current_context.md
   
   # Reinstall instructions
   airs-memspec install --force
   ```

4. **Rebuild from fragments:**
   ```bash
   # Look for recoverable data
   grep -r "^# \[TASK" .copilot/memory_bank.corrupted.*/ | head -10
   
   # Manually recreate critical files
   # Use GitHub Copilot to help rebuild from context
   ```

### Task Index Inconsistencies

**Problem:** Task indices don't match actual task files

**Diagnosis:**
```bash
# Compare index entries with actual files
for index_file in $(find .copilot/memory_bank -name "_index.md"); do
  project_dir=$(dirname "$index_file")
  echo "=== $(basename "$project_dir") ==="
  
  # Count tasks in index
  index_tasks=$(grep -c "^\- \[TASK" "$index_file" 2>/dev/null || echo 0)
  
  # Count actual task files
  actual_tasks=$(find "$project_dir" -name "task_*.md" | wc -l)
  
  echo "Index: $index_tasks, Actual: $actual_tasks"
  
  if [ "$index_tasks" -ne "$actual_tasks" ]; then
    echo "❌ Inconsistency detected"
  fi
done
```

**Rebuilding Task Indices:**

1. **Automated rebuild:**
   ```bash
   # Remove existing indices
   find .copilot/memory_bank -name "_index.md" -delete
   
   # Use GitHub Copilot to rebuild
   # In GitHub Copilot chat: "update memory bank"
   ```

2. **Manual rebuild script:**
   ```bash
   #!/bin/bash
   # rebuild-task-indices.sh
   
   for project_dir in .copilot/memory_bank/sub_projects/*/; do
     if [ -d "$project_dir/tasks" ]; then
       index_file="$project_dir/tasks/_index.md"
       
       echo "# Tasks Index" > "$index_file"
       echo "" >> "$index_file"
       
       # Group by status
       for status in "in_progress" "pending" "completed" "abandoned"; do
         echo "## $(echo "$status" | tr '_' ' ' | awk '{for(i=1;i<=NF;i++)sub(/./,toupper(substr($i,1,1)),$i)}1')" >> "$index_file"
         
         for task_file in "$project_dir/tasks/task_"*.md; do
           if [ -f "$task_file" ] && grep -q "**Status:** $status" "$task_file"; then
             task_id=$(basename "$task_file" .md | sed 's/^task_//')
             title=$(grep "^# \[" "$task_file" | head -1 | sed 's/^# \[.*\] - //')
             echo "- [$task_id] $title" >> "$index_file"
           fi
         done
         echo "" >> "$index_file"
       done
       
       echo "✅ Rebuilt: $(basename "$project_dir")/tasks/_index.md"
     fi
   done
   ```

## GitHub Copilot Integration Issues

### Custom Instructions Not Working

**Problem:** GitHub Copilot doesn't respond to memory bank commands

**Diagnosis:**
```bash
# Check instructions installation
ls -la .copilot/instructions/

# Verify instruction content
head -20 .copilot/instructions/multi_project_memory_bank.instructions.md

# Check VS Code settings (if using VS Code)
code --list-extensions | grep copilot
```

**Solutions:**

1. **Verify VS Code configuration:**
   ```json
   // Check settings.json
   {
     "github.copilot.customInstructions": ".copilot/instructions/"
   }
   ```

2. **Reinstall instructions:**
   ```bash
   airs-memspec install --force
   ```

3. **Use global instructions:**
   ```bash
   # macOS
   airs-memspec install --path ~/Library/Application\ Support/Code/User/copilot
   
   # Linux
   airs-memspec install --path ~/.config/Code/User/copilot
   
   # Windows
   airs-memspec install --path %APPDATA%\Code\User\copilot
   ```

4. **Test Copilot functionality:**
   ```markdown
   # In GitHub Copilot chat:
   "Hi, can you see custom instructions?"
   "update memory bank"
   "show memory bank summary"
   ```

### Copilot Memory Commands Failing

**Problem:** Commands like "update memory bank" don't work as expected

**Troubleshooting Steps:**

1. **Check instruction file syntax:**
   ```bash
   # Look for YAML front matter errors
   head -10 .copilot/instructions/multi_project_memory_bank.instructions.md
   
   # Verify markdown formatting
   markdown-lint .copilot/instructions/multi_project_memory_bank.instructions.md
   ```

2. **Test with simpler commands:**
   ```markdown
   # Start with basic commands
   "show current context"
   "list active tasks"
   ```

3. **Verify workspace state:**
   ```bash
   airs-memspec status --workspace
   ```

4. **Check instruction file permissions:**
   ```bash
   ls -la .copilot/instructions/
   chmod 644 .copilot/instructions/*.md
   ```

## Command Execution Problems

### Task Commands Failing

**Problem:** `airs-memspec tasks list` produces errors or unexpected output

**Common Issues:**

1. **Malformed task files:**
   ```bash
   # Find tasks with missing required fields
   for task_file in $(find .copilot/memory_bank -name "task_*.md"); do
     if ! grep -q "^**Status:**" "$task_file"; then
       echo "Missing status: $task_file"
     fi
   done
   ```

2. **Permission issues:**
   ```bash
   # Check task directory permissions
   find .copilot/memory_bank -name "tasks" -type d -exec ls -ld {} \;
   ```

3. **File encoding issues:**
   ```bash
   # Check for non-UTF8 files
   find .copilot/memory_bank -name "*.md" -exec file {} \; | grep -v UTF-8
   ```

**Resolution:**

1. **Fix malformed files:**
   ```bash
   # Add missing status fields
   for task_file in $(find .copilot/memory_bank -name "task_*.md"); do
     if ! grep -q "^**Status:**" "$task_file"; then
       echo "" >> "$task_file"
       echo "**Status:** pending" >> "$task_file"
     fi
   done
   ```

2. **Reset task permissions:**
   ```bash
   find .copilot/memory_bank -name "tasks" -type d -exec chmod 755 {} \;
   find .copilot/memory_bank -name "*.md" -exec chmod 644 {} \;
   ```

### Status Command Issues

**Problem:** `airs-memspec status` shows incorrect or missing information

**Debugging:**

1. **Check current_context.md:**
   ```bash
   cat .copilot/memory_bank/current_context.md
   ```

2. **Verify project structure:**
   ```bash
   find .copilot/memory_bank/sub_projects -type d -name "*" | head -10
   ```

3. **Test with verbose output:**
   ```bash
   airs-memspec status --workspace --verbose
   ```

**Fixes:**

1. **Recreate current_context.md:**
   ```bash
   cat > .copilot/memory_bank/current_context.md << 'EOF'
   # Current Context
   
   **Active Sub-Project:** main
   **Last Updated:** $(date +%Y-%m-%d)
   
   ## Context Overview
   Active development workspace
   EOF
   ```

2. **Validate project structure:**
   ```bash
   # Ensure each project has required files
   for project_dir in .copilot/memory_bank/sub_projects/*/; do
     project_name=$(basename "$project_dir")
     
     for required_file in project_brief.md active_context.md progress.md; do
       if [ ! -f "$project_dir/$required_file" ]; then
         echo "Creating missing file: $project_dir/$required_file"
         touch "$project_dir/$required_file"
       fi
     done
   done
   ```

## Performance Issues

### Slow Command Execution

**Problem:** airs-memspec commands take a long time to execute

**Diagnosis:**
```bash
# Time command execution
time airs-memspec status --workspace

# Check memory bank size
du -sh .copilot/memory_bank/

# Count total files
find .copilot/memory_bank -type f | wc -l

# Check for large files
find .copilot/memory_bank -size +10M
```

**Optimization Steps:**

1. **Compress old files:**
   ```bash
   # Compress files older than 90 days
   find .copilot/memory_bank -name "*.md" -mtime +90 -exec gzip {} \;
   ```

2. **Clean up empty files:**
   ```bash
   # Remove empty task files
   find .copilot/memory_bank -name "task_*.md" -size 0 -delete
   ```

3. **Archive old snapshots:**
   ```bash
   # Move old snapshots to archive
   mkdir -p .copilot/memory_bank/archive/
   find .copilot/memory_bank/context_snapshots -name "*.md" -mtime +180 \
     -exec mv {} .copilot/memory_bank/archive/ \;
   ```

4. **Split large projects:**
   ```bash
   # Consider splitting into separate repositories
   # if project has > 1000 task files
   find .copilot/memory_bank/sub_projects -name "task_*.md" | wc -l
   ```

### Memory Usage Issues

**Problem:** High memory consumption during operation

**Monitoring:**
```bash
# Monitor memory usage during command execution
/usr/bin/time -v airs-memspec status --workspace 2>&1 | grep -E "(Maximum|Average)"

# Check system memory
free -h  # Linux
vm_stat  # macOS
```

**Solutions:**

1. **Use streaming operations:**
   ```bash
   # Process files in batches
   find .copilot/memory_bank -name "task_*.md" | head -100 | xargs grep "Status:"
   ```

2. **Reduce verbosity:**
   ```bash
   # Use quiet mode for scripts
   airs-memspec status --quiet
   ```

## Network and Connectivity Issues

### GitHub Integration Problems

**Problem:** Issues with GitHub integration or remote operations

**Diagnosis:**
```bash
# Test GitHub connectivity
ssh -T git@github.com

# Check repository status
git remote -v
git status

# Verify GitHub CLI (if used)
gh auth status
```

**Solutions:**

1. **Fix SSH keys:**
   ```bash
   # Generate new SSH key
   ssh-keygen -t ed25519 -C "your-email@example.com"
   
   # Add to SSH agent
   ssh-add ~/.ssh/id_ed25519
   
   # Add to GitHub account
   cat ~/.ssh/id_ed25519.pub
   ```

2. **Update Git configuration:**
   ```bash
   git config --global user.name "Your Name"
   git config --global user.email "your-email@example.com"
   ```

3. **Fix remote URLs:**
   ```bash
   # Switch to SSH URLs
   git remote set-url origin git@github.com:username/repository.git
   ```

## Environment-Specific Issues

### Windows Specific Problems

**Path separator issues:**
```bash
# Use forward slashes or escape backslashes
airs-memspec status --path "C:/workspace" 
airs-memspec status --path "C:\\workspace"
```

**PowerShell execution policy:**
```powershell
# Allow script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Line ending problems:**
```bash
# Convert line endings
git config --global core.autocrlf true
```

### macOS Specific Problems

**Quarantine issues:**
```bash
# Remove quarantine attribute
xattr -rd com.apple.quarantine ~/.cargo/bin/airs-memspec
```

**PATH issues in GUI applications:**
```bash
# Add to ~/.zshrc for macOS Catalina+
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
```

### Linux Distribution Issues

**Missing dependencies:**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL/Fedora  
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel

# Arch Linux
sudo pacman -S base-devel openssl
```

## Data Recovery

### Backup and Restore

**Creating Backups:**
```bash
# Full memory bank backup
tar -czf "membank-backup-$(date +%Y%m%d).tar.gz" .copilot/memory_bank/

# Git-based backup
git add .copilot/memory_bank/
git commit -m "Memory bank backup - $(date)"
git tag "membank-$(date +%Y%m%d)"
```

**Restoring from Backup:**
```bash
# From tar archive
tar -xzf membank-backup-20250115.tar.gz

# From Git
git checkout membank-20250115 -- .copilot/memory_bank/

# Partial restore (tasks only)
git checkout HEAD~5 -- .copilot/memory_bank/sub_projects/*/tasks/
```

### Emergency Recovery

**Minimal Working State:**
```bash
#!/bin/bash
# emergency-restore.sh - Create minimal working memory bank

mkdir -p .copilot/memory_bank/workspace
mkdir -p .copilot/memory_bank/sub_projects/main/tasks

# Create minimal current_context.md
cat > .copilot/memory_bank/current_context.md << 'EOF'
# Current Context

**Active Sub-Project:** main
**Last Updated:** $(date +%Y-%m-%d)

## Recovery Note
Memory bank restored to minimal working state.
Previous state may be available in git history.
EOF

# Create minimal project files
cat > .copilot/memory_bank/sub_projects/main/project_brief.md << 'EOF'
# Main Project Brief

Emergency restored project structure.
EOF

cat > .copilot/memory_bank/sub_projects/main/active_context.md << 'EOF'
# Active Context

Recovery mode - please update with current project status.
EOF

# Create empty task index
cat > .copilot/memory_bank/sub_projects/main/tasks/_index.md << 'EOF'
# Tasks Index

No tasks currently tracked.
EOF

# Reinstall instructions
airs-memspec install --force

echo "✅ Emergency memory bank restoration complete"
echo "🔄 Use 'update memory bank' in GitHub Copilot to rebuild state"
```

## Getting Help

### Diagnostic Information Collection

**System Information Script:**
```bash
#!/bin/bash
# collect-diagnostics.sh - Gather system information for support

echo "=== airs-memspec Diagnostic Information ==="
echo "Date: $(date)"
echo "User: $USER"
echo "PWD: $PWD"
echo ""

echo "=== System Information ==="
uname -a
echo ""

echo "=== Rust/Cargo Version ==="
rustc --version
cargo --version
echo ""

echo "=== airs-memspec Version ==="
airs-memspec --version
echo ""

echo "=== Memory Bank Structure ==="
find .copilot/memory_bank -type f -name "*.md" | head -20
echo ""

echo "=== Recent Errors ==="
tail -50 ~/.memspec-health.log 2>/dev/null || echo "No health log found"
echo ""

echo "=== Environment Variables ==="
env | grep -E "(PATH|CARGO|RUST)" | sort
echo ""

echo "=== File Permissions ==="
ls -la .copilot/memory_bank/ 2>/dev/null || echo "Memory bank not found"
```

### Community Support

**Issue Reporting:**
1. Run diagnostic script above
2. Include output in GitHub issue
3. Provide minimal reproduction steps
4. Specify expected vs actual behavior

**Documentation:**
- Check [GitHub repository](https://github.com/your-org/airs-memspec) for latest docs
- Review usage examples in `/docs/src/usages/`
- Search existing issues for similar problems

**Development:**
- Clone repository for local development
- Run test suite: `cargo test --workspace`
- Check contributing guidelines

---

*This troubleshooting guide covers common issues and resolution strategies. For complex problems, collect diagnostic information and seek community support.*
