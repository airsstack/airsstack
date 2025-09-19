#!/usr/bin/env python3
"""
Comprehensive test runner for HTTP API Key MCP client integration tests.

This script runs all test suites with different configurations and generates
a comprehensive test report.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any


class TestRunner:
    """Comprehensive test runner for HTTP client integration tests."""
    
    def __init__(self, project_dir: Path):
        self.project_dir = project_dir
        self.test_dir = project_dir / "tests"
        self.results = []
        
    def setup_environment(self):
        """Set up the test environment."""
        print("Setting up test environment...")
        
        # Ensure we're in the virtual environment
        venv_python = self.test_dir / "venv" / "bin" / "python"
        if not venv_python.exists():
            print("Virtual environment not found. Please run ./run_tests.sh first.")
            sys.exit(1)
        
        # Set environment variables
        os.environ["PYTHONPATH"] = str(self.test_dir)
        os.environ["RUST_LOG"] = "info,airs_mcp=debug"
        
        print("✅ Environment setup complete")
    
    def run_test_suite(self, test_file: str, suite_name: str, env_vars: Dict[str, str] = None) -> Dict[str, Any]:
        """Run a specific test suite and return results."""
        print(f"\n{'='*60}")
        print(f"Running {suite_name}")
        print(f"Test file: {test_file}")
        print(f"{'='*60}")
        
        # Prepare environment
        env = os.environ.copy()
        if env_vars:
            env.update(env_vars)
            print(f"Environment overrides: {env_vars}")
        
        # Run the test
        cmd = [
            str(self.test_dir / "venv" / "bin" / "python"), 
            "-m", "pytest", 
            str(self.test_dir / test_file),
            "-v",
            "--tb=short",
            "--capture=no"
        ]
        
        start_time = time.time()
        result = subprocess.run(cmd, env=env, capture_output=True, text=True, cwd=self.test_dir)
        end_time = time.time()
        
        # Parse results
        duration = end_time - start_time
        
        test_result = {
            "suite_name": suite_name,
            "test_file": test_file,
            "duration": duration,
            "exit_code": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "passed": result.returncode == 0,
            "timestamp": datetime.now().isoformat(),
            "env_vars": env_vars or {}
        }
        
        # Print summary
        if test_result["passed"]:
            print(f"✅ {suite_name} PASSED in {duration:.2f}s")
        else:
            print(f"❌ {suite_name} FAILED in {duration:.2f}s")
            print(f"Exit code: {result.returncode}")
            if result.stderr:
                print(f"Errors: {result.stderr[-500:]}")  # Last 500 chars of stderr
        
        self.results.append(test_result)
        return test_result
    
    def run_all_tests(self, test_configs: List[Dict[str, Any]]):
        """Run all configured test suites."""
        print(f"Running {len(test_configs)} test configurations...")
        
        for i, config in enumerate(test_configs, 1):
            print(f"\n[{i}/{len(test_configs)}] {config['name']}")
            
            # Skip if test file doesn't exist
            test_file_path = self.test_dir / config["file"]
            if not test_file_path.exists():
                print(f"⚠️  Skipping {config['name']} - test file not found: {config['file']}")
                continue
            
            try:
                self.run_test_suite(
                    config["file"],
                    config["name"],
                    config.get("env_vars", {})
                )
            except Exception as e:
                print(f"❌ {config['name']} failed with exception: {e}")
                self.results.append({
                    "suite_name": config["name"],
                    "test_file": config["file"],
                    "duration": 0,
                    "exit_code": -1,
                    "stdout": "",
                    "stderr": str(e),
                    "passed": False,
                    "timestamp": datetime.now().isoformat(),
                    "env_vars": config.get("env_vars", {}),
                    "exception": str(e)
                })
        
        self.generate_report()
    
    def generate_report(self):
        """Generate a comprehensive test report."""
        print(f"\n{'='*80}")
        print("TEST REPORT SUMMARY")
        print(f"{'='*80}")
        
        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r["passed"])
        failed_tests = total_tests - passed_tests
        total_duration = sum(r["duration"] for r in self.results)
        
        print(f"Total test suites: {total_tests}")
        print(f"Passed: {passed_tests}")
        print(f"Failed: {failed_tests}")
        print(f"Success rate: {(passed_tests/total_tests)*100:.1f}%")
        print(f"Total duration: {total_duration:.2f}s")
        
        print(f"\n{'='*80}")
        print("DETAILED RESULTS")
        print(f"{'='*80}")
        
        for result in self.results:
            status = "✅ PASS" if result["passed"] else "❌ FAIL"
            print(f"{status} {result['suite_name']:<40} {result['duration']:>8.2f}s")
            
            if not result["passed"]:
                print(f"      Exit code: {result['exit_code']}")
                if result.get("exception"):
                    print(f"      Exception: {result['exception']}")
                elif result["stderr"]:
                    # Show first few lines of stderr
                    stderr_lines = result["stderr"].split('\n')[:3]
                    for line in stderr_lines:
                        if line.strip():
                            print(f"      Error: {line.strip()}")
        
        # Save detailed report to file
        report_file = self.test_dir / "test_report.json"
        with open(report_file, 'w') as f:
            json.dump({
                "summary": {
                    "total_suites": total_tests,
                    "passed": passed_tests,
                    "failed": failed_tests,
                    "success_rate": (passed_tests/total_tests)*100 if total_tests > 0 else 0,
                    "total_duration": total_duration,
                    "timestamp": datetime.now().isoformat()
                },
                "results": self.results
            }, f, indent=2)
        
        print(f"\n📋 Detailed report saved to: {report_file}")
        
        # Return exit code based on results
        return 0 if failed_tests == 0 else 1


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Comprehensive HTTP client integration test runner")
    parser.add_argument("--suite", choices=["mock", "production", "stress", "all"], default="all",
                       help="Test suite to run")
    parser.add_argument("--production-url", default="http://127.0.0.1:3000",
                       help="Production server URL for production tests")
    parser.add_argument("--production-key", default="test-api-key",
                       help="Production server API key")
    parser.add_argument("--start-production", action="store_true",
                       help="Start production server for production tests")
    
    args = parser.parse_args()
    
    # Determine project directory
    project_dir = Path(__file__).parent.parent
    
    # Create test runner
    runner = TestRunner(project_dir)
    runner.setup_environment()
    
    # Define test configurations
    test_configs = []
    
    if args.suite in ["mock", "all"]:
        test_configs.append({
            "name": "Mock Server Integration Tests",
            "file": "test_http_client_mock_integration.py",
        })
    
    if args.suite in ["production", "all"]:
        test_configs.append({
            "name": "Production Server Integration Tests",
            "file": "test_http_client_production_integration.py",
            "env_vars": {
                "PRODUCTION_SERVER_URL": args.production_url,
                "PRODUCTION_API_KEY": args.production_key,
                "START_PRODUCTION_SERVER": "true" if args.start_production else "false"
            }
        })
    
    if args.suite in ["stress", "all"]:
        test_configs.append({
            "name": "Stress and Validation Tests",
            "file": "test_stress_validation.py",
        })
    
    # Authentication method variations for mock tests
    if args.suite in ["mock", "all"]:
        for auth_method in ["XApiKey", "Bearer", "QueryParameter"]:
            test_configs.append({
                "name": f"Mock Server Tests - {auth_method} Auth",
                "file": "test_http_client_mock_integration.py",
                "env_vars": {
                    "AUTH_METHOD": auth_method
                }
            })
    
    # Run tests
    print(f"🚀 Starting comprehensive test run with {len(test_configs)} configurations...")
    exit_code = runner.run_all_tests(test_configs)
    
    print(f"\n🏁 Test run complete!")
    sys.exit(exit_code)


if __name__ == "__main__":
    main()