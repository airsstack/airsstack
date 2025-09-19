#!/usr/bin/env python3
"""
Automated OAuth2 Flow Integration Test

This script automates the complete OAuth2 + MCP integration testing without
relying on terminal command coordination. It starts all necessary servers,
runs the client test, and validates the complete flow.
"""

import asyncio
import json
import logging
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import urllib.parse
import requests

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class ServerManager:
    """Manages OAuth2 and MCP server processes"""
    
    def __init__(self, project_dir: Path):
        self.project_dir = project_dir
        self.processes: Dict[str, subprocess.Popen] = {}
        self.server_configs = {
            'oauth2_server': {
                'command': ['cargo', 'run', '--bin', 'http-oauth2-mock-server', '--', 
                           '--host', '127.0.0.1', '--port', '3002'],
                'port': 3002,
                'health_path': '/health'
            },
            'mcp_server': {
                'command': ['cargo', 'run', '--bin', 'http-mcp-mock-server', '--', 
                           '--host', '127.0.0.1', '--port', '3003'],
                'port': 3003,
                'health_path': '/health'
            }
        }
    
    def start_server(self, server_name: str) -> bool:
        """Start a specific server"""
        config = self.server_configs[server_name]
        
        logger.info(f"🚀 Starting {server_name}...")
        
        try:
            process = subprocess.Popen(
                config['command'],
                cwd=self.project_dir,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
                universal_newlines=True
            )
            
            self.processes[server_name] = process
            
            # Wait for server to start and become healthy
            if self.wait_for_server_health(server_name, timeout=30):
                logger.info(f"✅ {server_name} started successfully on port {config['port']}")
                return True
            else:
                logger.error(f"❌ {server_name} failed to start or become healthy")
                self.stop_server(server_name)
                return False
                
        except Exception as e:
            logger.error(f"❌ Failed to start {server_name}: {e}")
            return False
    
    def wait_for_server_health(self, server_name: str, timeout: int = 30) -> bool:
        """Wait for server to become healthy"""
        config = self.server_configs[server_name]
        url = f"http://127.0.0.1:{config['port']}{config['health_path']}"
        
        start_time = time.time()
        while time.time() - start_time < timeout:
            try:
                response = requests.get(url, timeout=2)
                if response.status_code == 200:
                    return True
            except requests.exceptions.RequestException:
                pass
            
            # Check if process is still running
            process = self.processes.get(server_name)
            if process and process.poll() is not None:
                logger.error(f"❌ {server_name} process exited unexpectedly")
                self.log_process_output(server_name)
                return False
            
            time.sleep(1)
        
        return False
    
    def start_all_servers(self) -> bool:
        """Start all required servers"""
        logger.info("🏗️  Starting all servers...")
        
        # Kill any existing processes on our ports
        self.kill_processes_on_ports([3002, 3003])
        
        for server_name in self.server_configs:
            if not self.start_server(server_name):
                logger.error(f"❌ Failed to start {server_name}")
                self.stop_all_servers()
                return False
        
        logger.info("✅ All servers started successfully")
        return True
    
    def stop_server(self, server_name: str):
        """Stop a specific server"""
        process = self.processes.get(server_name)
        if process:
            logger.info(f"🛑 Stopping {server_name}...")
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            del self.processes[server_name]
    
    def stop_all_servers(self):
        """Stop all managed servers"""
        logger.info("🛑 Stopping all servers...")
        for server_name in list(self.processes.keys()):
            self.stop_server(server_name)
    
    def kill_processes_on_ports(self, ports: List[int]):
        """Kill any processes running on specified ports"""
        for port in ports:
            try:
                # Find processes using the port
                result = subprocess.run(['lsof', '-ti', f':{port}'], 
                                      capture_output=True, text=True)
                if result.stdout.strip():
                    pids = result.stdout.strip().split('\n')
                    for pid in pids:
                        subprocess.run(['kill', '-9', pid], 
                                     capture_output=True)
                    logger.info(f"🔪 Killed existing processes on port {port}")
            except Exception as e:
                logger.debug(f"No processes to kill on port {port}: {e}")
    
    def log_process_output(self, server_name: str):
        """Log the output of a failed process"""
        process = self.processes.get(server_name)
        if process:
            stdout, stderr = process.communicate()
            if stdout:
                logger.error(f"{server_name} stdout: {stdout}")
            if stderr:
                logger.error(f"{server_name} stderr: {stderr}")

class OAuth2FlowTester:
    """Tests the complete OAuth2 flow"""
    
    def __init__(self, project_dir: Path):
        self.project_dir = project_dir
        self.auth_server_url = "http://127.0.0.1:3002"
        self.mcp_server_url = "http://127.0.0.1:3003"
    
    async def test_oauth2_flow(self) -> bool:
        """Test the complete OAuth2 flow"""
        logger.info("🔐 Starting OAuth2 flow test...")
        
        # Test 1: Verify servers are responding
        if not self.verify_server_endpoints():
            return False
        
        # Test 2: Run the OAuth2 client
        if not await self.run_oauth2_client():
            return False
        
        logger.info("✅ OAuth2 flow test completed successfully!")
        return True
    
    def verify_server_endpoints(self) -> bool:
        """Verify that all required server endpoints are responding"""
        logger.info("🌐 Verifying server endpoints...")
        
        endpoints_to_test = [
            (f"{self.auth_server_url}/health", "OAuth2 server health"),
            (f"{self.auth_server_url}/.well-known/openid-configuration", "OAuth2 OIDC discovery"),
            (f"{self.auth_server_url}/jwks", "OAuth2 JWKS"),
            (f"{self.mcp_server_url}/health", "MCP server health"),
        ]
        
        for url, description in endpoints_to_test:
            try:
                response = requests.get(url, timeout=5)
                if response.status_code == 200:
                    logger.info(f"✅ {description}: OK")
                else:
                    logger.error(f"❌ {description}: HTTP {response.status_code}")
                    return False
            except Exception as e:
                logger.error(f"❌ {description}: {e}")
                return False
        
        return True
    
    async def run_oauth2_client(self) -> bool:
        """Run the OAuth2 client and validate output"""
        logger.info("🔌 Running OAuth2 client...")
        
        command = [
            'cargo', 'run', '--bin', 'http-oauth2-client', '--',
            '--auth-server', self.auth_server_url,
            '--mcp-server', self.mcp_server_url
        ]
        
        try:
            process = await asyncio.create_subprocess_exec(
                *command,
                cwd=self.project_dir,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                text=True
            )
            
            # Wait for completion with timeout
            try:
                stdout, stderr = await asyncio.wait_for(
                    process.communicate(), timeout=60
                )
            except asyncio.TimeoutError:
                logger.error("❌ OAuth2 client timed out")
                process.kill()
                await process.wait()
                return False
            
            # Analyze the output
            success = self.analyze_client_output(stdout, stderr, process.returncode)
            
            if success:
                logger.info("✅ OAuth2 client completed successfully")
            else:
                logger.error("❌ OAuth2 client failed")
                logger.error(f"Exit code: {process.returncode}")
                if stdout:
                    logger.error(f"STDOUT:\n{stdout}")
                if stderr:
                    logger.error(f"STDERR:\n{stderr}")
            
            return success
            
        except Exception as e:
            logger.error(f"❌ Failed to run OAuth2 client: {e}")
            return False
    
    def analyze_client_output(self, stdout: str, stderr: str, exit_code: int) -> bool:
        """Analyze client output for success indicators"""
        
        # Success indicators we're looking for
        success_indicators = [
            "Authorization URL:",
            "Authorization code generated",
            "Token exchange successful",
            "MCP operations completed successfully"
        ]
        
        # Error indicators
        error_indicators = [
            "AuthorizationFailed",
            "400 Bad Request",
            "Configuration error",
            "Connection refused"
        ]
        
        combined_output = f"{stdout}\n{stderr}"
        
        # Check for error indicators first
        for error in error_indicators:
            if error in combined_output:
                logger.error(f"❌ Found error indicator: {error}")
                return False
        
        # Check for success indicators
        success_count = 0
        for success in success_indicators:
            if success in combined_output:
                logger.info(f"✅ Found success indicator: {success}")
                success_count += 1
        
        # We need at least some success indicators and exit code 0
        return exit_code == 0 and success_count >= 2

async def main():
    """Main test execution function"""
    
    # Setup signal handler for cleanup
    def signal_handler(signum, frame):
        logger.info("🛑 Received interrupt signal, cleaning up...")
        sys.exit(1)
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Determine project directory
    script_dir = Path(__file__).parent
    project_dir = script_dir
    
    logger.info("🧪 Starting OAuth2 Integration Test Suite")
    logger.info(f"📁 Project directory: {project_dir}")
    
    server_manager = ServerManager(project_dir)
    flow_tester = OAuth2FlowTester(project_dir)
    
    try:
        # Step 1: Start all servers
        if not server_manager.start_all_servers():
            logger.error("❌ Failed to start servers")
            return 1
        
        # Step 2: Wait a moment for servers to fully initialize
        logger.info("⏳ Waiting for servers to fully initialize...")
        await asyncio.sleep(3)
        
        # Step 3: Test the OAuth2 flow
        if not await flow_tester.test_oauth2_flow():
            logger.error("❌ OAuth2 flow test failed")
            return 1
        
        logger.info("🎉 All tests passed successfully!")
        return 0
        
    except Exception as e:
        logger.error(f"❌ Unexpected error: {e}")
        return 1
        
    finally:
        # Cleanup
        server_manager.stop_all_servers()
        logger.info("🧹 Cleanup completed")

if __name__ == "__main__":
    # Install required packages if needed
    try:
        import requests
    except ImportError:
        logger.info("📦 Installing required Python packages...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", "requests"])
        import requests
    
    # Run the test suite
    exit_code = asyncio.run(main())
    sys.exit(exit_code)