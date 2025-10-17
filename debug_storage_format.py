#!/usr/bin/env python3
"""
Debug the storage file format to identify persistence issues.
"""

import os
import struct
import hashlib
import time
import sys
import logging

# Add packages to path
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-core'))
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-hybrid'))

from sutra_hybrid.engine import SutraAI

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def analyze_storage_file(file_path):
    """Analyze the binary storage file format."""
    print(f"🔍 ANALYZING STORAGE FILE: {file_path}")
    print("=" * 60)
    
    if not os.path.exists(file_path):
        print("❌ Storage file does not exist")
        return
    
    file_size = os.path.getsize(file_path)
    print(f"📁 File size: {file_size:,} bytes ({file_size/1024/1024:.2f} MB)")
    
    with open(file_path, 'rb') as f:
        # Read header
        header = f.read(64)
        if len(header) < 64:
            print("❌ File too small - invalid format")
            return
        
        # Parse header
        magic = header[:8]
        version, concept_count, edge_count = struct.unpack('<III', header[8:20])
        timestamp = struct.unpack('<Q', header[20:28])[0]
        
        print(f"🏷️ Magic bytes: {magic}")
        print(f"📊 Format version: {version}")
        print(f"📦 Concept count: {concept_count}")
        print(f"🔗 Edge count: {edge_count}")
        print(f"⏰ Timestamp: {timestamp}")
        
        if magic != b'SUTRADAT':
            print("⚠️ WARNING: Magic bytes don't match expected format")
        
        # Read and analyze concepts
        print(f"\n📖 ANALYZING {concept_count} CONCEPTS:")
        print("-" * 40)
        
        concepts_analyzed = 0
        for i in range(min(concept_count, 5)):  # Analyze first 5 concepts
            try:
                # Read concept header (36 bytes)
                concept_header = f.read(36)
                if len(concept_header) != 36:
                    print(f"❌ Concept {i}: Incomplete header")
                    break
                
                # Parse header
                id_bytes = concept_header[:16]
                content_len, strength, confidence, access_count, created = struct.unpack('<IffiI', concept_header[16:36])
                
                # Read content
                content = f.read(content_len)
                if len(content) != content_len:
                    print(f"❌ Concept {i}: Incomplete content")
                    break
                
                content_str = content.decode('utf-8', errors='replace')
                id_hex = id_bytes.hex()
                
                print(f"  Concept {i}:")
                print(f"    ID: {id_hex[:16]}...")
                print(f"    Content: '{content_str[:50]}...'")
                print(f"    Length: {content_len} bytes")
                print(f"    Strength: {strength:.3f}")
                print(f"    Confidence: {confidence:.3f}")
                print(f"    Access count: {access_count}")
                
                concepts_analyzed += 1
                
            except Exception as e:
                print(f"❌ Error analyzing concept {i}: {e}")
                break
        
        print(f"\n✅ Successfully analyzed {concepts_analyzed} concepts")
        
        # Analyze edges
        print(f"\n🔗 ANALYZING {edge_count} EDGES:")
        print("-" * 40)
        
        edges_analyzed = 0
        for i in range(min(edge_count, 5)):  # Analyze first 5 edges
            try:
                edge_data = f.read(36)  # source_id(16) + target_id(16) + confidence(4)
                if len(edge_data) != 36:
                    print(f"❌ Edge {i}: Incomplete data")
                    break
                
                source_id = edge_data[:16].hex()
                target_id = edge_data[16:32].hex()
                confidence = struct.unpack('<f', edge_data[32:36])[0]
                
                print(f"  Edge {i}: {source_id[:8]}... -> {target_id[:8]}... (conf: {confidence:.3f})")
                edges_analyzed += 1
                
            except Exception as e:
                print(f"❌ Error analyzing edge {i}: {e}")
                break
        
        print(f"✅ Successfully analyzed {edges_analyzed} edges")

def test_storage_write_read_cycle():
    """Test the complete write-read cycle."""
    print("\n🔄 TESTING WRITE-READ CYCLE")
    print("=" * 60)
    
    # Clean storage
    storage_path = "./knowledge/storage.dat"
    if os.path.exists(storage_path):
        os.remove(storage_path)
        print("🗑️ Cleaned existing storage")
    
    # Create engine and learn something
    print("📝 Learning test concept...")
    engine = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    test_content = f"Storage Debug Test - {int(time.time())}"
    result = engine.learn(test_content)
    print(f"✅ Learned: {test_content}")
    print(f"   Concept ID: {result.concept_id}")
    
    # Test immediate query
    immediate_response = engine.ask(f"What is {test_content}?")
    print(f"🔍 Immediate query confidence: {immediate_response.confidence:.3f}")
    
    # Force save
    engine.save()
    print("💾 Storage saved")
    
    # Analyze the written file
    analyze_storage_file(storage_path)
    
    # Close and recreate engine
    engine.close()
    del engine
    print("🗑️ Engine destroyed")
    
    # Create new engine
    print("🔄 Creating new engine...")
    engine2 = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    # Test persistence query
    persistence_response = engine2.ask(f"What is {test_content}?")
    print(f"🔍 Persistence query confidence: {persistence_response.confidence:.3f}")
    
    # Debug: Check if concept exists by ID
    concept_details = engine2.get_concept(result.concept_id)
    if concept_details:
        print(f"✅ Concept found by ID: {concept_details['content'][:50]}...")
    else:
        print("❌ Concept NOT found by ID")
    
    # Compare content
    if immediate_response.confidence > 0.5 and persistence_response.confidence == 0.0:
        print("\n💥 PERSISTENCE ISSUE CONFIRMED:")
        print("   - Immediate query works (confidence > 0.5)")
        print("   - Persistence query fails (confidence = 0.0)")
        print("   - This indicates content/association mismatch")
    elif persistence_response.confidence > 0.5:
        print("\n✅ PERSISTENCE WORKS CORRECTLY")
    else:
        print("\n⚠️ BOTH QUERIES FAILED - Different issue")
    
    return {
        'immediate_confidence': immediate_response.confidence,
        'persistence_confidence': persistence_response.confidence,
        'concept_found_by_id': concept_details is not None
    }

if __name__ == "__main__":
    print("🔍 STORAGE FORMAT DEBUG UTILITY")
    print("=" * 60)
    
    # First, analyze existing storage if it exists
    storage_path = "./knowledge/storage.dat"
    if os.path.exists(storage_path):
        analyze_storage_file(storage_path)
    else:
        print("ℹ️ No existing storage file found")
    
    # Test complete cycle
    results = test_storage_write_read_cycle()
    
    print("\n📋 FINAL ANALYSIS")
    print("=" * 60)
    for key, value in results.items():
        print(f"  {key}: {value}")
    
    if results['persistence_confidence'] == 0.0 and results['concept_found_by_id']:
        print("\n🔍 DIAGNOSIS: Content stored but associations/queries failing")
        print("   Likely cause: Query processing not finding stored content")
    elif not results['concept_found_by_id']:
        print("\n🔍 DIAGNOSIS: Concept not being loaded properly from storage")
        print("   Likely cause: Binary format parser issue")
    else:
        print("\n✅ Storage and persistence working correctly")