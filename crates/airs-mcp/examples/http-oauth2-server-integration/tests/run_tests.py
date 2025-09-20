#!/usr/bin/env python3
"""
OAuth2 MCP Integration Test Runner

This script provides a unified interface to run all OAuth2 MCP integration tests.

Usage:
    python3 run_tests.py [test_type] [options]
    
Test Types:
    basic        Run basic functionality test (default, recommended)
    comprehensive Run comprehensive integration test
    advanced     Run advanced integration test with all features
    flow         Run OAuth2 authorization flow test with PKCE validation
    edge         Run OAuth2 edge case and security tests
    jsonrpc      Run JSON-RPC protocol edge case tests
    all          Run all test suites in sequence
    
Options:
    --debug      Enable debug output
    --no-cleanup Keep server running after tests
    --help       Show this help message
"""

import sys
import subprocess
import argparse
import os
from pathlib import Path


def run_test(script_name: str, debug: bool = False, no_cleanup: bool = False) -> bool:
    """Run a specific test script"""
    script_path = Path(__file__).parent / script_name
    if not script_path.exists():
        print(f"❌ Test script not found: {script_name}")
        return False
    
    # Build command
    cmd = [sys.executable, str(script_path)]
    if debug:
        cmd.append("--debug")
    if no_cleanup:
        cmd.append("--no-cleanup")
    
    print(f"🚀 Running {script_name}...")
    print("-" * 60)
    
    try:
        result = subprocess.run(cmd, check=False)
        success = result.returncode == 0
        
        if success:
            print(f"✅ {script_name} completed successfully")
        else:
            print(f"❌ {script_name} failed with exit code {result.returncode}")
        
        print("-" * 60)
        return success
        
    except Exception as e:
        print(f"❌ Error running {script_name}: {e}")
        return False


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="OAuth2 MCP Integration Test Runner",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    python3 run_tests.py                    # Run basic test
    python3 run_tests.py basic --debug      # Run basic test with debug output
    python3 run_tests.py comprehensive      # Run comprehensive test
    python3 run_tests.py flow               # Run OAuth2 authorization flow test
    python3 run_tests.py edge               # Run OAuth2 edge case and security tests
    python3 run_tests.py jsonrpc            # Run JSON-RPC protocol edge case tests
    python3 run_tests.py all                # Run all tests
    python3 run_tests.py basic --no-cleanup # Keep server running after test
        """
    )
    
    parser.add_argument(
        "test_type", 
        nargs="?", 
        default="basic",
        choices=["basic", "comprehensive", "advanced", "flow", "edge", "jsonrpc", "all"],
        help="Type of test to run (default: basic)"
    )
    parser.add_argument("--debug", action="store_true", help="Enable debug output")
    parser.add_argument("--no-cleanup", action="store_true", help="Keep server running after tests")
    
    args = parser.parse_args()
    
    print("🧪 OAuth2 MCP Integration Test Runner")
    print("=" * 60)
    
    # Test script mapping
    test_scripts = {
        "basic": "test_oauth2_basic.py",
        "comprehensive": "test_oauth2_comprehensive.py", 
        "advanced": "test_oauth2_integration.py",
        "flow": "test_oauth2_authorization_flow.py",
        "edge": "test_oauth2_edge_cases.py",
        "jsonrpc": "test_jsonrpc_edge_cases.py"
    }
    
    success_count = 0
    total_tests = 0
    
    if args.test_type == "all":
        # Run all tests in sequence
        for test_name in ["basic", "comprehensive", "advanced", "flow", "edge", "jsonrpc"]:
            total_tests += 1
            if run_test(test_scripts[test_name], args.debug, args.no_cleanup):
                success_count += 1
            print()  # Add spacing between tests
    else:
        # Run single test
        total_tests = 1
        if run_test(test_scripts[args.test_type], args.debug, args.no_cleanup):
            success_count = 1
    
    # Summary
    print("=" * 60)
    print("📊 Test Runner Summary")
    print("=" * 60)
    print(f"Tests completed: {success_count}/{total_tests}")
    
    if success_count == total_tests:
        print("🎉 All tests passed!")
        if args.no_cleanup:
            print()
            print("🔄 Server endpoints (still running):")
            print("  • MCP: http://localhost:3001/mcp")
            print("  • Tokens: http://localhost:3002/auth/tokens")
        sys.exit(0)
    else:
        print(f"💥 {total_tests - success_count} test(s) failed")
        sys.exit(1)


if __name__ == "__main__":
    main()