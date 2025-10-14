#!/usr/bin/env python3
"""Debug script to test the biological intelligence API flow."""

import requests
import json
import time
from datetime import datetime

def log(message, level="INFO"):
    """Simple logging function."""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{timestamp}] {level}: {message}")

def test_core_service():
    """Test direct connection to core service."""
    log("Testing Core Service (http://localhost:8000)")
    
    try:
        # Test status endpoint
        response = requests.get("http://localhost:8000/api/status")
        if response.status_code == 200:
            data = response.json()
            log(f"✅ Core service is running - Consciousness: {data['consciousness_score']:.2f}", "SUCCESS")
            log(f"   Total concepts: {data['total_concepts']}, Associations: {data['total_associations']}")
            return True
        else:
            log(f"❌ Core service returned status {response.status_code}", "ERROR")
            return False
    except Exception as e:
        log(f"❌ Cannot connect to core service: {e}", "ERROR")
        return False

def test_web_gui():
    """Test connection to web GUI."""
    log("Testing Web GUI (http://localhost:8080)")
    
    try:
        # Test status endpoint
        response = requests.get("http://localhost:8080/api/status")
        if response.status_code == 200:
            data = response.json()
            log(f"✅ Web GUI is running - Service: {'Running' if data.get('service_running') else 'Not Running'}", "SUCCESS")
            return True
        else:
            log(f"❌ Web GUI returned status {response.status_code}", "ERROR")
            return False
    except Exception as e:
        log(f"❌ Cannot connect to web GUI: {e}", "ERROR")
        return False

def test_query_core():
    """Test querying the core service directly."""
    log("\nTesting query to Core Service")
    
    query = "What is consciousness?"
    payload = {
        "query": query,
        "max_results": 5,
        "hops": 3
    }
    
    try:
        log(f"Sending query: '{query}'")
        response = requests.post(
            "http://localhost:8000/api/query",
            json=payload,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            data = response.json()
            log(f"✅ Got {len(data.get('results', []))} results", "SUCCESS")
            log(f"   Consciousness score: {data.get('consciousness_score', 0):.3f}")
            log(f"   Processing time: {data.get('processing_time', 0)*1000:.1f}ms")
            
            # Display first result
            if data.get('results'):
                result = data['results'][0]
                log(f"   Top result: '{result['content']}' (relevance: {result['relevance']:.3f})")
            return True
        else:
            log(f"❌ Query failed with status {response.status_code}", "ERROR")
            log(f"   Response: {response.text}")
            return False
    except Exception as e:
        log(f"❌ Query error: {e}", "ERROR")
        return False

def test_query_webgui():
    """Test querying through the web GUI."""
    log("\nTesting query through Web GUI")
    
    query = "What is consciousness?"
    
    # Web GUI expects form data
    form_data = {
        "question": query,
        "hops": "3",
        "max_results": "10"
    }
    
    try:
        log(f"Sending query: '{query}'")
        response = requests.post(
            "http://localhost:8080/api/query",
            data=form_data  # Note: using data= for form data, not json=
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get('success'):
                log(f"✅ Got {len(data.get('results', []))} results", "SUCCESS")
                log(f"   Consciousness score: {data.get('consciousness_score', 0):.3f}")
                log(f"   Processing time: {data.get('processing_time', 0)*1000:.1f}ms")
                
                # Display first result
                if data.get('results'):
                    result = data['results'][0]
                    log(f"   Top result: '{result['content']}' (relevance: {result['relevance']:.3f})")
                return True
            else:
                log(f"❌ Query unsuccessful: {data.get('error', 'Unknown error')}", "ERROR")
                return False
        else:
            log(f"❌ Query failed with status {response.status_code}", "ERROR")
            log(f"   Response: {response.text}")
            return False
    except Exception as e:
        log(f"❌ Query error: {e}", "ERROR")
        return False

def test_feed_knowledge():
    """Test feeding knowledge through web GUI."""
    log("\nTesting knowledge feeding through Web GUI")
    
    knowledge = "Test knowledge: The biological intelligence system uses consciousness emergence."
    
    form_data = {
        "knowledge": knowledge
    }
    
    try:
        log(f"Feeding knowledge: '{knowledge[:50]}...'")
        response = requests.post(
            "http://localhost:8080/api/knowledge/feed",
            data=form_data
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get('success'):
                log(f"✅ Knowledge fed successfully: {data.get('message', 'No message')}", "SUCCESS")
                return True
            else:
                log(f"❌ Feed unsuccessful: {data.get('error', 'Unknown error')}", "ERROR")
                return False
        else:
            log(f"❌ Feed failed with status {response.status_code}", "ERROR")
            return False
    except Exception as e:
        log(f"❌ Feed error: {e}", "ERROR")
        return False

def main():
    """Run all tests."""
    log("=== Biological Intelligence API Debug ===\n")
    
    # Test connections
    core_ok = test_core_service()
    gui_ok = test_web_gui()
    
    if not core_ok:
        log("\n⚠️  Core service is not running. Start it with:", "WARNING")
        log("   docker-compose -f docker-compose.webgui.yml up -d")
        return
    
    if not gui_ok:
        log("\n⚠️  Web GUI is not running properly", "WARNING")
        return
    
    # Test queries
    time.sleep(1)
    test_query_core()
    
    time.sleep(1)
    test_query_webgui()
    
    # Test feeding
    time.sleep(1)
    test_feed_knowledge()
    
    log("\n=== Debug Complete ===")
    log("If all tests passed, the chat interface should work correctly!")
    log("Open http://localhost:8080/intelligence in your browser")

if __name__ == "__main__":
    main()