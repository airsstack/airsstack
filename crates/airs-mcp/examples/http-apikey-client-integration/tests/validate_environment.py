#!/usr/bin/env python3
"""
Quick test validation script to check if the test environment is properly set up.

This script performs basic validation of the test environment without running
full integration tests.
"""

import os
import subprocess
import sys
from pathlib import Path


def check_environment():
    """Check if the test environment is properly set up."""
    print("🔍 Validating test environment...")
    
    project_dir = Path(__file__).parent.parent
    test_dir = Path(__file__).parent
    
    # Check if we're in the right directory
    print(f"Project directory: {project_dir}")
    print(f"Test directory: {test_dir}")
    
    # Check if virtual environment exists
    venv_python = test_dir / "venv" / "bin" / "python"
    if venv_python.exists():
        print("✅ Virtual environment found")
    else:
        print("❌ Virtual environment not found")
        return False
    
    # Check if requirements.txt exists
    requirements_file = test_dir / "requirements.txt"
    if requirements_file.exists():
        print("✅ Requirements file found")
    else:
        print("❌ Requirements file not found")
        return False
    
    # Check if test files exist
    test_files = [
        "test_http_client_mock_integration.py",
        "test_http_client_production_integration.py", 
        "test_stress_validation.py",
        "run_comprehensive_tests.py"
    ]
    
    for test_file in test_files:
        if (test_dir / test_file).exists():
            print(f"✅ Test file found: {test_file}")
        else:
            print(f"❌ Test file missing: {test_file}")
            return False
    
    # Check if main scripts exist
    main_scripts = ["run_tests.sh"]
    for script in main_scripts:
        script_path = test_dir / script
        if script_path.exists():
            print(f"✅ Script found: {script}")
            if os.access(script_path, os.X_OK):
                print(f"✅ Script is executable: {script}")
            else:
                print(f"⚠️  Script not executable: {script}")
        else:
            print(f"❌ Script missing: {script}")
            return False
    
    # Try to activate virtual environment and check packages
    try:
        result = subprocess.run([
            str(venv_python), "-c", 
            "import pytest, requests, psutil, aiohttp, httpx; print('All required packages available')"
        ], capture_output=True, text=True, timeout=10)
        
        if result.returncode == 0:
            print("✅ All required Python packages available")
        else:
            print(f"❌ Package check failed: {result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Failed to check packages: {e}")
        return False
    
    # Check if Rust binaries can be built
    print("🔨 Checking Rust binary compilation...")
    try:
        result = subprocess.run([
            "cargo", "check", "--bin", "http-apikey-client"
        ], cwd=project_dir, capture_output=True, text=True, timeout=30)
        
        if result.returncode == 0:
            print("✅ HTTP client binary compiles successfully")
        else:
            print(f"❌ HTTP client binary compilation failed: {result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Failed to check binary compilation: {e}")
        return False
    
    return True


def run_simple_test():
    """Run a simple test to verify the environment works."""
    print("\n🧪 Running simple environment test...")
    
    test_dir = Path(__file__).parent
    venv_python = test_dir / "venv" / "bin" / "python"
    
    # Simple pytest execution test
    test_code = '''
import pytest

def test_simple():
    """Simple test to verify pytest works."""
    assert 1 + 1 == 2
    print("✅ Simple test passed")

if __name__ == "__main__":
    test_simple()
    print("✅ Environment test successful")
'''
    
    try:
        result = subprocess.run([
            str(venv_python), "-c", test_code
        ], capture_output=True, text=True, timeout=10)
        
        if result.returncode == 0:
            print("✅ Simple test execution successful")
            print(result.stdout)
            return True
        else:
            print(f"❌ Simple test failed: {result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Simple test execution failed: {e}")
        return False


def main():
    """Main validation function."""
    print("🚀 HTTP API Key Client Integration Test Environment Validation")
    print("=" * 70)
    
    # Check environment
    env_ok = check_environment()
    
    if not env_ok:
        print("\n❌ Environment validation failed!")
        print("Please run ./run_tests.sh to set up the environment properly.")
        sys.exit(1)
    
    # Run simple test
    test_ok = run_simple_test()
    
    if not test_ok:
        print("\n❌ Simple test failed!")
        sys.exit(1)
    
    print("\n" + "=" * 70)
    print("✅ Environment validation successful!")
    print("")
    print("Your test environment is ready. You can now run:")
    print("  ./run_tests.sh                    # Run all tests")
    print("  ./run_tests.sh --comprehensive    # Run with detailed reporting")
    print("  ./run_tests.sh --suite mock       # Run only mock server tests")
    print("  ./run_tests.sh --help            # Show all options")
    print("")


if __name__ == "__main__":
    main()