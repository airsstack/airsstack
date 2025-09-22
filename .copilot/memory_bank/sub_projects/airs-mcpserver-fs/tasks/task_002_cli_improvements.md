# [task_002] - CLI Improvements and Internal Renaming

**Status:** pending  
**Added:** 2025-09-23  
**Updated:** 2025-09-23

## Original Request
Improve `airs-mcpserver-fs` with:
1. Add `setup` command to auto-create `$HOME/.airs-mcpserver-fs/{config,logs}` 
2. Add `--config-dir` and `--logs-dir` parameters to `server` command
3. Rename `generate-config` → `config` command
4. Update all internal references from `airs-mcp-fs` → `airs-mcpserver-fs` (env vars, strings, comments)

## Implementation Plan
1. **Add setup command** - Create directory structure automatically
2. **Add CLI parameters** - Custom config/logs directory support
3. **Rename commands** - `generate-config` → `config`
4. **Internal cleanup** - Environment variables and references

## Progress Tracking

**Overall Status:** not_started - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 2.1 | Add setup command with directory creation | not_started | 2025-09-23 | Auto-create ~/.airs-mcpserver-fs/{config,logs} |
| 2.2 | Add --config-dir and --logs-dir to server command | not_started | 2025-09-23 | Override default directories |
| 2.3 | Rename generate-config to config command | not_started | 2025-09-23 | Breaking change with migration note |
| 2.4 | Update environment variables AIRS_MCP_FS → AIRS_MCPSERVER_FS | not_started | 2025-09-23 | Internal consistency cleanup |
| 2.5 | Update internal strings and paths | not_started | 2025-09-23 | Log paths, config names, comments |
| 2.6 | Update documentation and examples | not_started | 2025-09-23 | CONFIGURATION.md and examples |

## Progress Log
### 2025-09-23
- Task created with comprehensive implementation plan
- Ready for implementation phase