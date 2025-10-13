#!/usr/bin/env python3
"""
🎓 Simple English Trainer - Core Architecture Compliant

A straightforward English training system that works directly with the core 
biological intelligence architecture without complex layering.
"""

import asyncio
import json
import sys
from pathlib import Path

# Add src to path for core imports
sys.path.insert(0, str(Path(__file__).parent / "src"))

from src.biological_trainer import BiologicalTrainer
from src.config import settings

def load_enhanced_curriculum():
    """Load the enhanced English curriculum"""
    curriculum_file = Path("enhanced_english_curriculum/optimal_learning_sequence.json")
    
    if not curriculum_file.exists():
        print("❌ Enhanced curriculum not found. Generating...")
        # Generate if not exists
        import subprocess
        result = subprocess.run([sys.executable, "enhanced_english_curriculum.py"], 
                              capture_output=True, text=True)
        if result.returncode != 0:
            print(f"❌ Failed to generate curriculum: {result.stderr}")
            return []
    
    try:
        with open(curriculum_file, 'r') as f:
            data = json.load(f)
            return data['sequence']
    except Exception as e:
        print(f"❌ Failed to load curriculum: {e}")
        return []

async def train_english():
    """Simple English training following core architecture"""
    print("🎓 SIMPLE ENGLISH TRAINING - CORE ARCHITECTURE")
    print("=" * 60)
    
    # Create trainer with explicit English workspace settings
    workspace_path = "./english_biological_workspace"
    workspace_id = "english"
    
    print(f"🧠 Initializing biological trainer...")
    print(f"   Base path: {workspace_path}")
    print(f"   Workspace ID: {workspace_id}")
    
    # Initialize trainer with core architecture
    trainer = BiologicalTrainer(
        base_path=workspace_path,
        workspace_id=workspace_id,
        use_full_swarm=True
    )
    
    # Load curriculum
    print(f"📚 Loading curriculum...")
    lessons = load_enhanced_curriculum()
    
    if not lessons:
        print("❌ No curriculum available")
        return False
    
    print(f"✅ Loaded {len(lessons)} lessons")
    
    # Simple batch training
    batch_size = 5
    total_batches = len(lessons) // batch_size + (1 if len(lessons) % batch_size else 0)
    
    print(f"🚀 Starting training ({total_batches} batches)...")
    
    for batch_num in range(0, len(lessons), batch_size):
        batch_lessons = lessons[batch_num:batch_num + batch_size]
        batch_texts = [lesson['content'] for lesson in batch_lessons]
        
        print(f"   Batch {batch_num//batch_size + 1}/{total_batches}: {len(batch_texts)} lessons")
        
        # Train on batch
        result = await trainer.train_from_stream(batch_texts)
        
        # Show consciousness if emerging
        if 'consciousness_score' in result and result['consciousness_score'] > 0.5:
            print(f"   🧠 CONSCIOUSNESS: {result['consciousness_score']:.2f}")
    
    # Save memory using core persistence
    print(f"💾 Saving memory...")
    trainer.save_memory()
    
    # Show final stats
    stats = trainer.memory_system.get_stats()
    print(f"\n✅ Training Complete!")
    print(f"   Concepts: {stats['total_concepts']}")
    print(f"   Associations: {stats['total_associations']}")
    print(f"   Average Strength: {stats['average_strength']:.3f}")
    
    return True

async def verify_learning():
    """Simple learning verification"""
    print("\n🔍 VERIFYING LEARNING")
    print("=" * 40)
    
    # Load trained system
    trainer = BiologicalTrainer(
        base_path="./english_biological_workspace",
        workspace_id="english",
        use_full_swarm=True
    )
    
    print(f"📥 Loading trained memory...")
    trainer.load_memory()
    
    concepts_count = len(trainer.memory_system.concepts)
    associations_count = len(trainer.memory_system.associations)
    
    print(f"   Concepts loaded: {concepts_count}")
    print(f"   Associations loaded: {associations_count}")
    
    if concepts_count == 0:
        print("❌ No concepts found - training failed")
        return False
    
    # Test simple queries
    test_queries = ["english", "vowels", "alphabet", "words", "sentence"]
    
    print(f"\n🧪 Testing queries...")
    for query in test_queries:
        results = trainer.query_knowledge(query, max_results=3)
        if results:
            print(f"   '{query}': {len(results)} results")
            for i, result in enumerate(results[:2], 1):
                content = result.get('content', '')[:50]
                score = result.get('relevance', 0)
                print(f"      {i}. {content}... ({score:.3f})")
        else:
            print(f"   '{query}': No results")
    
    return concepts_count > 0

if __name__ == "__main__":
    async def main():
        # Train
        success = await train_english()
        if not success:
            sys.exit(1)
        
        # Verify
        verified = await verify_learning()
        if not verified:
            sys.exit(1)
        
        print(f"\n🎉 English training completed successfully!")
    
    # Run training
    asyncio.run(main())