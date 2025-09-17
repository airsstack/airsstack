#!/usr/bin/env python3
"""
STDIO MCP Integration Test Runner

This script provides a unified interface to run all STDIO MCP integration tests.

Usage:
    python3 run_tests.py [test_type] [options]
    
Test Types:
    basic        Run basic functionality test (default, recommended)
    integration  Run comprehensive integration test
    comprehensive Run advanced comprehensive test with performance metrics
    all          Run all test suites in sequence
    
Options:
    --debug      Enable debug output
    --verbose    Enable verbose output
    --server-path PATH  Specify custom path to stdio-server binary
    --help       Show this help message
"""

import sys
import subprocess
import argparse
import os
from pathlib import Path


def find_server_binary() -> str:
    """Find the STDIO server binary"""
    test_dir = Path(__file__).parent
    possible_paths = [
        test_dir / "../target/debug/stdio-server",
        test_dir / "../target/release/stdio-server",
        test_dir / "../../../../target/debug/examples/stdio-server-integration",
        test_dir / "../../../../target/release/examples/stdio-server-integration"
    ]
    
    for path in possible_paths:
        if Path(path).exists():
            return str(path.resolve())
    
    return None


def run_test(script_name: str, debug: bool = False, verbose: bool = False, server_path: str = None) -> bool:
    """Run a specific test script"""
    script_path = Path(__file__).parent / script_name
    if not script_path.exists():
        print(f"❌ Test script not found: {script_name}")
        return False
    
    # Build command
    cmd = [sys.executable, str(script_path)]
    if debug:
        cmd.append("--debug")
    if verbose:
        cmd.append("--verbose")
    if server_path:
        cmd.extend(["--server-path", server_path])
    
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
        description="STDIO MCP Integration Test Runner",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    python3 run_tests.py                           # Run basic test
    python3 run_tests.py basic --debug             # Run basic test with debug output
    python3 run_tests.py integration               # Run integration test
    python3 run_tests.py comprehensive --verbose   # Run comprehensive test with verbose output
    python3 run_tests.py all                       # Run all tests
    python3 run_tests.py basic --server-path /custom/path/stdio-server  # Use custom server path
        """
    )
    
    parser.add_argument(
        "test_type", 
        nargs="?", 
        default="basic",
        choices=["basic", "integration", "comprehensive", "all"],
        help="Type of test to run (default: basic)"
    )
    parser.add_argument("--debug", action="store_true", help="Enable debug output")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose output")
    parser.add_argument("--server-path", help="Path to stdio-server binary")
    
    args = parser.parse_args()
    
    print("🧪 STDIO MCP Integration Test Runner")
    print("=" * 60)
    
    # Find server binary if not specified
    if not args.server_path:
        args.server_path = find_server_binary()
        if not args.server_path:
            print("❌ STDIO server binary not found!")
            print()
            print("Please build the server first:")
            print("  cd ../ && cargo build")
            print()
            print("Or specify the binary path:")
            print("  python3 run_tests.py basic --server-path /path/to/stdio-server")
            sys.exit(1)
    
    print(f"📍 Using server binary: {args.server_path}")
    print()
    
    # Verify server binary exists
    if not Path(args.server_path).exists():
        print(f"❌ Server binary not found: {args.server_path}")
        sys.exit(1)
    
    # Test script mapping
    test_scripts = {
        "basic": "test_stdio_basic.py",
        "integration": "test_stdio_integration.py",
        "comprehensive": "test_stdio_comprehensive.py"
    }
    
    success_count = 0
    total_tests = 0
    
    if args.test_type == "all":
        # Run all tests in sequence
        for test_name in ["basic", "integration", "comprehensive"]:
            total_tests += 1
            if run_test(test_scripts[test_name], args.debug, args.verbose, args.server_path):
                success_count += 1
            print()  # Add spacing between tests
    else:
        # Run single test
        total_tests = 1
        if run_test(test_scripts[args.test_type], args.debug, args.verbose, args.server_path):
            success_count = 1
    
    # Summary
    print("=" * 60)
    print("📊 Test Runner Summary")
    print("=" * 60)
    print(f"Tests completed: {success_count}/{total_tests}")
    print(f"Server binary: {args.server_path}")
    
    if success_count == total_tests:
        print("🎉 All tests passed!")
        print()
        print("STDIO MCP server is working correctly!")
        print("The server processes JSON-RPC requests via stdin/stdout.")
        sys.exit(0)
    else:
        print(f"💥 {total_tests - success_count} test(s) failed")
        print()
        print("Check the test output above for details.")
        print("Ensure the server binary is built and accessible.")
        sys.exit(1)


if __name__ == "__main__":
    main()