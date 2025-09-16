# OAuth2 Integration Logs Directory

This directory contains log files generated during OAuth2 MCP integration testing and development.

## Log Files

### Server Logs
- **`server.log`** - Main server output from basic tests (`test_oauth2_basic.py`)
- **`server_comprehensive.log`** - Server output from comprehensive tests (`test_oauth2_comprehensive.py`)
- **`server_integration.log`** - Server output from integration tests (`test_oauth2_integration.py`)

### Test Logs
- **`test_basic_*.log`** - Basic test execution logs with timestamps
- **`test_comprehensive_*.log`** - Comprehensive test execution logs with timestamps
- **`test_integration_*.log`** - Integration test execution logs with timestamps

### Development Logs
- **`debug_*.log`** - Debug output when tests are run with debug mode
- **`error_*.log`** - Error logs for failed test runs
- **`performance_*.log`** - Performance metrics and timing data

## Log File Management

### Automatic Cleanup
- Logs are rotated when they exceed 10MB
- Older logs are compressed and archived
- Only the last 5 log files are kept per test type

### Git Ignore
All log files are ignored by git as specified in the `.gitignore` file:
```
# Log files
logs/
*.log
```

### Manual Cleanup
To clean up all logs:
```bash
rm -rf logs/*.log
```

## Troubleshooting

When debugging issues:
1. Check the appropriate server log file for server-side errors
2. Check test log files for test execution details
3. Use debug mode for verbose logging: `python run_tests.py --debug`
4. Review error logs for specific failure details

## Log Rotation

Log files automatically rotate based on:
- **Size**: When files exceed 10MB
- **Time**: Daily rotation for long-running servers
- **Count**: Maximum 5 files per log type