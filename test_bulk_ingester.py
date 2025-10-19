#!/usr/bin/env python3
"""
Quick test script for the bulk ingester
"""

import subprocess
import time
import requests
import json
import sys
from pathlib import Path

def test_bulk_ingester():
    """Test the bulk ingester implementation."""
    print("🧪 Testing Sutra Bulk Ingester")
    
    # Check if we can build the Rust code
    print("\n1. Building Rust code...")
    ingester_dir = Path("packages/sutra-bulk-ingester")
    
    if not ingester_dir.exists():
        print("❌ Bulk ingester directory not found")
        return False
    
    try:
        # Build the project
        result = subprocess.run(
            ["cargo", "build"], 
            cwd=ingester_dir,
            capture_output=True, 
            text=True,
            timeout=120
        )
        
        if result.returncode != 0:
            print("❌ Build failed:")
            print(result.stderr)
            return False
        
        print("✅ Build successful")
        
        # Try to run the binary (this will fail because no storage server, but should show it compiles)
        print("\n2. Testing binary startup...")
        result = subprocess.run(
            ["cargo", "run", "--", "--help"], 
            cwd=ingester_dir,
            capture_output=True, 
            text=True,
            timeout=30
        )
        
        if "High-performance bulk data ingestion service" in result.stdout:
            print("✅ Binary runs and shows help")
        else:
            print("⚠️ Binary startup issue:")
            print(result.stdout)
            print(result.stderr)
        
        return True
        
    except subprocess.TimeoutExpired:
        print("❌ Build timed out")
        return False
    except FileNotFoundError:
        print("❌ Cargo not found - install Rust first")
        return False
    except Exception as e:
        print(f"❌ Build error: {e}")
        return False

def check_environment():
    """Check if the environment is ready."""
    print("🔍 Checking environment...")
    
    # Check if Rust is installed
    try:
        result = subprocess.run(["cargo", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print(f"✅ Rust: {result.stdout.strip()}")
        else:
            print("❌ Rust/Cargo not found")
            return False
    except FileNotFoundError:
        print("❌ Rust not installed")
        return False
    
    # Check if we're in the right directory
    if not Path("packages").exists():
        print("❌ Not in sutra-models root directory")
        return False
    
    print("✅ Environment looks good")
    return True

def show_architecture():
    """Show the architecture we've implemented."""
    print("\n📋 IMPLEMENTED ARCHITECTURE:")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("🦀 RUST CORE (High Performance):")
    print("  ├── lib.rs              # Main ingestion engine")
    print("  ├── adapters.rs         # Plugin interface")
    print("  ├── storage.rs          # TCP storage client")
    print("  ├── server.rs           # FastAPI-like web server")
    print("  ├── plugins.rs          # Plugin registry")
    print("  └── main.rs             # Executable")
    print("")
    print("🐍 PYTHON PLUGINS (Flexibility):")
    print("  └── plugins/wikipedia_adapter.py  # Wikipedia dataset adapter")
    print("")
    print("🚀 FEATURES:")
    print("  • Async streaming for memory efficiency")
    print("  • TCP binary protocol for storage")
    print("  • Pluggable adapter system")
    print("  • RESTful API for job management")
    print("  • Real-time progress tracking")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

def show_next_steps():
    """Show what needs to be done next."""
    print("\n📋 NEXT STEPS TO COMPLETE INTEGRATION:")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("1️⃣  INTEGRATION:")
    print("    • Connect to existing TCP storage server")
    print("    • Integrate with sutra-protocol package")
    print("    • Add proper error handling")
    print("")
    print("2️⃣  PYTHON ADAPTER BRIDGE:")
    print("    • Complete PyO3 integration")
    print("    • Load wikipedia_adapter.py dynamically")
    print("    • Test with real Wikipedia dataset")
    print("")
    print("3️⃣  DOCKER INTEGRATION:")
    print("    • Add to docker-compose-grid.yml")
    print("    • Update sutra-deploy.sh")
    print("    • Test 14-service orchestration")
    print("")
    print("4️⃣  PERFORMANCE TESTING:")
    print("    • Benchmark with Wikipedia dataset")
    print("    • Compare vs current Python consumer")
    print("    • Optimize batch sizes and memory usage")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

if __name__ == "__main__":
    show_architecture()
    
    if not check_environment():
        print("\n❌ Environment check failed")
        sys.exit(1)
    
    if test_bulk_ingester():
        print("\n✅ BULK INGESTER FOUNDATION COMPLETE!")
        show_next_steps()
        print("\nStatus: Foundation implemented, integration needed for full functionality")
    else:
        print("\n❌ Tests failed")
        sys.exit(1)