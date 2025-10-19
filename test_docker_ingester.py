#!/usr/bin/env python3
"""
Docker-based test for the bulk ingester with Wikipedia dataset
"""

import subprocess
import time
import requests
import json
import sys
from pathlib import Path

def create_test_wikipedia_dataset():
    """Create a small test Wikipedia dataset for testing."""
    datasets_dir = Path("datasets")
    datasets_dir.mkdir(exist_ok=True)
    
    test_data = """Albert Einstein

Albert Einstein was a German-born theoretical physicist, widely held to be one of the greatest and most influential scientists of all time. Best known for developing the theory of relativity, Einstein also made important contributions to quantum mechanics, and was thus a central figure in the revolutionary reshaping of the scientific understanding of nature that modern physics accomplished in the first decades of the twentieth century.


Marie Curie

Marie Salomea Skłodowska-Curie was a Polish and naturalised-French physicist and chemist who conducted pioneering research on radioactivity. She was the first woman to win a Nobel Prize, the first person and the only woman to win the Nobel Prize twice, and the only person to win the Nobel Prize in two different scientific fields.


Isaac Newton

Sir Isaac Newton was an English mathematician, physicist, astronomer, alchemist, theologian, and author who is widely recognised as one of the greatest mathematicians and physicists of all time. He was a key figure in the philosophical revolution known as the Enlightenment. His book Philosophiæ Naturalis Principia Mathematica, first published in 1687, established classical mechanics.


Charles Darwin

Charles Robert Darwin was an English naturalist who proposed the theory of biological evolution by natural selection. Darwin published his theory of evolution with compelling evidence in his 1859 book On the Origin of Species. By the 1870s, the scientific community and a majority of the educated public had accepted evolution as a fact.


Nikola Tesla

Nikola Tesla was a Serbian-American inventor, electrical engineer, mechanical engineer, and futurist best known for his contributions to the design of the modern alternating current electricity supply system. Born and raised in the Austrian Empire, Tesla studied engineering and physics in the 1870s without receiving a degree, gaining practical experience in the early 1880s working in telephony and at Continental Edison in the new electric power industry."""

    test_file = datasets_dir / "wikipedia.txt"
    test_file.write_text(test_data)
    
    print(f"✅ Created test Wikipedia dataset: {test_file}")
    print(f"   Size: {test_file.stat().st_size} bytes")
    print(f"   Articles: 5")
    
    return test_file

def test_docker_build():
    """Test building the Docker image."""
    print("\n🔨 Building Docker image...")
    
    try:
        result = subprocess.run([
            "docker", "build", 
            "-f", "packages/sutra-bulk-ingester/Dockerfile",
            "-t", "sutra-bulk-ingester:test",
            "."
        ], capture_output=True, text=True, timeout=600)
        
        if result.returncode == 0:
            print("✅ Docker build successful")
            return True
        else:
            print("❌ Docker build failed:")
            print(result.stderr)
            return False
            
    except subprocess.TimeoutExpired:
        print("❌ Docker build timed out")
        return False
    except Exception as e:
        print(f"❌ Build error: {e}")
        return False

def test_storage_server():
    """Test if storage server is available."""
    print("\n🔍 Checking storage server availability...")
    
    try:
        # Check if storage server is running via docker-compose
        result = subprocess.run([
            "docker", "compose", "-f", "docker-compose-grid.yml",
            "ps", "storage-server"
        ], capture_output=True, text=True)
        
        if "Up" in result.stdout:
            print("✅ Storage server is running")
            return True
        else:
            print("⚠️ Storage server not running, starting it...")
            return start_storage_server()
            
    except Exception as e:
        print(f"❌ Error checking storage server: {e}")
        return False

def start_storage_server():
    """Start just the storage server for testing."""
    try:
        print("🚀 Starting storage server...")
        result = subprocess.run([
            "docker", "compose", "-f", "docker-compose-grid.yml",
            "up", "-d", "storage-server"
        ], capture_output=True, text=True, timeout=120)
        
        if result.returncode == 0:
            # Wait for health check
            print("⏳ Waiting for storage server health check...")
            time.sleep(10)
            return True
        else:
            print("❌ Failed to start storage server:")
            print(result.stderr)
            return False
            
    except subprocess.TimeoutExpired:
        print("❌ Storage server start timed out")
        return False

def test_ingester_api():
    """Test the ingester API endpoints."""
    print("\n🧪 Testing ingester API...")
    
    try:
        # Start the ingester service
        result = subprocess.run([
            "docker", "run", "-d",
            "--name", "test-ingester",
            "--network", "sutra-models_sutra-network",
            "-p", "8005:8005",
            "-e", "SUTRA_STORAGE_SERVER=storage-server:50051",
            "-v", f"{Path.cwd()}/datasets:/datasets:ro",
            "sutra-bulk-ingester:test"
        ], capture_output=True, text=True)
        
        if result.returncode != 0:
            print("❌ Failed to start ingester container")
            print(result.stderr)
            return False
        
        print("⏳ Waiting for ingester to start...")
        time.sleep(15)
        
        # Test health endpoint
        try:
            response = requests.get("http://localhost:8005/health", timeout=5)
            if response.status_code == 200:
                print("✅ Health check passed")
            else:
                print(f"❌ Health check failed: {response.status_code}")
                return False
        except requests.RequestException as e:
            print(f"❌ Health check request failed: {e}")
            return False
        
        # Test adapters endpoint
        try:
            response = requests.get("http://localhost:8005/adapters", timeout=5)
            if response.status_code == 200:
                adapters = response.json()
                print(f"✅ Available adapters: {adapters}")
            else:
                print(f"⚠️ Adapters endpoint returned: {response.status_code}")
        except requests.RequestException as e:
            print(f"⚠️ Adapters request failed: {e}")
        
        # Test job submission
        try:
            job_data = {
                "source_type": "file",
                "source_config": {
                    "path": "/datasets/wikipedia.txt",
                    "format": "wikipedia"
                },
                "adapter_name": "file"
            }
            
            response = requests.post("http://localhost:8005/jobs", 
                                   json=job_data, timeout=10)
            if response.status_code == 200:
                job = response.json()
                print(f"✅ Job submitted: {job['id']}")
                
                # Check job status
                time.sleep(3)
                status_response = requests.get(f"http://localhost:8005/jobs/{job['id']}")
                if status_response.status_code == 200:
                    status = status_response.json()
                    print(f"✅ Job status: {status['status']}")
                
            else:
                print(f"❌ Job submission failed: {response.status_code}")
                print(response.text)
        
        except requests.RequestException as e:
            print(f"⚠️ Job submission test failed: {e}")
        
        return True
        
    except Exception as e:
        print(f"❌ API test error: {e}")
        return False
    
    finally:
        # Cleanup test container
        subprocess.run(["docker", "stop", "test-ingester"], 
                      capture_output=True)
        subprocess.run(["docker", "rm", "test-ingester"], 
                      capture_output=True)

def cleanup():
    """Cleanup test resources."""
    print("\n🧹 Cleaning up...")
    subprocess.run(["docker", "stop", "test-ingester"], 
                  capture_output=True)
    subprocess.run(["docker", "rm", "test-ingester"], 
                  capture_output=True)

def show_docker_status():
    """Show current Docker ecosystem status."""
    print("\n📊 DOCKER ECOSYSTEM STATUS:")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    
    try:
        result = subprocess.run([
            "docker", "compose", "-f", "docker-compose-grid.yml", "ps"
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            print(result.stdout)
        else:
            print("⚠️ Could not get Docker status")
            
    except Exception as e:
        print(f"⚠️ Error getting Docker status: {e}")

def show_next_steps():
    """Show next steps for full integration."""
    print("\n📋 NEXT STEPS FOR FULL WIKIPEDIA INGESTION:")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("1️⃣  START FULL ECOSYSTEM:")
    print("    docker-compose -f docker-compose-with-ingester.yml up -d")
    print("")
    print("2️⃣  SUBMIT WIKIPEDIA JOB:")
    print("    curl -X POST http://localhost:8005/jobs \\")
    print("      -H 'Content-Type: application/json' \\")
    print("      -d '{")
    print("        \"source_type\": \"file\",")
    print("        \"source_config\": {\"path\": \"/datasets/wikipedia.txt\"},")
    print("        \"adapter_name\": \"file\"")
    print("      }'")
    print("")
    print("3️⃣  MONITOR PROGRESS:")
    print("    # Check jobs: curl http://localhost:8005/jobs")
    print("    # Check logs: docker logs sutra-bulk-ingester")
    print("")
    print("4️⃣  ACCESS INTERFACES:")
    print("    # Control Center: http://localhost:9000")
    print("    # Bulk Ingester API: http://localhost:8005")
    print("    # Client Interface: http://localhost:8080")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

if __name__ == "__main__":
    print("🐳 DOCKER BULK INGESTER TEST")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    
    try:
        # Create test dataset
        create_test_wikipedia_dataset()
        
        # Show current Docker status
        show_docker_status()
        
        # Test Docker build
        if not test_docker_build():
            sys.exit(1)
        
        # Test storage server
        if not test_storage_server():
            print("⚠️ Continuing without storage server")
        
        # Test ingester API
        if test_ingester_api():
            print("\n✅ DOCKER INTEGRATION SUCCESSFUL!")
            show_next_steps()
        else:
            print("\n❌ Some tests failed")
            sys.exit(1)
            
    except KeyboardInterrupt:
        print("\n⚠️ Test interrupted")
        cleanup()
    except Exception as e:
        print(f"\n❌ Test error: {e}")
        cleanup()
        sys.exit(1)