#!/usr/bin/env python3
"""
🧬 BIOLOGICAL INTELLIGENCE SERVICE
A living, persistent system that continues learning independently.

This service runs as a daemon process that:
- Persists knowledge across restarts
- Continues learning even when terminals disconnect
- Provides APIs for observation and interaction
- Maintains loose coupling between components

The biological intelligence LIVES independently of any interface.
"""

import asyncio
import json
import time
import signal
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional, List
import pickle
import logging
from enum import Enum

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('biological_intelligence.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger('BiologicalService')


class ServiceState(Enum):
    """Service lifecycle states"""
    INITIALIZING = "initializing"
    LEARNING = "learning"
    DREAMING = "dreaming"
    CONSOLIDATING = "consolidating"
    IDLE = "idle"
    STOPPING = "stopping"


class BiologicalIntelligenceService:
    """
    The core biological intelligence service.
    This runs independently and persistently, maintaining living knowledge.
    """
    
    def __init__(self, workspace_path: str = "./biological_workspace"):
        self.workspace = Path(workspace_path)
        self.workspace.mkdir(exist_ok=True)
        
        # Service state
        self.state = ServiceState.INITIALIZING
        self.start_time = datetime.now()
        self.is_running = True
        
        # Paths for persistence
        self.memory_path = self.workspace / "memory.pbss"
        self.state_path = self.workspace / "service_state.json"
        self.metrics_path = self.workspace / "metrics.json"
        self.training_queue_path = self.workspace / "training_queue.json"
        
        # Initialize or load biological trainer
        self._initialize_trainer()
        
        # Teacher-evaluator removed for architectural simplicity
        
        # Training queue (persistent)
        self.training_queue = self._load_training_queue()
        
        # Metrics tracking
        self.metrics = self._load_metrics()
        
        # Learning parameters
        self.dream_interval = 300  # Dream every 5 minutes
        self.last_dream_time = time.time()
        self.consolidation_interval = 600  # Consolidate every 10 minutes
        self.last_consolidation = time.time()
        
        logger.info(f"🧬 Biological Intelligence Service initialized at {self.workspace}")
        
    def _initialize_trainer(self):
        """Initialize or restore the biological trainer."""
        try:
            from src.biological_trainer import BiologicalTrainer
            
            # Use core architecture directly
            self.trainer = BiologicalTrainer(
                base_path=str(self.workspace),
                workspace_id="core",
                use_full_swarm=True
            )
            
            # Try to load existing memory
            if self.memory_path.exists():
                logger.info("Loading existing biological memory...")
                self.trainer.load_memory()
                logger.info(f"Restored {self.trainer.memory_system.get_stats()['total_concepts']} concepts")
            else:
                logger.info("Starting with fresh biological memory")
                
        except Exception as e:
            logger.error(f"Failed to initialize trainer: {e}")
            # Create a minimal fallback
            self.trainer = None
    
    # Teacher-evaluator system removed for architectural simplicity
    
    def _load_training_queue(self) -> List[str]:
        """Load persistent training queue."""
        if self.training_queue_path.exists():
            try:
                with open(self.training_queue_path, 'r') as f:
                    return json.load(f)
            except:
                pass
        return []
    
    def _save_training_queue(self):
        """Save training queue to disk."""
        try:
            with open(self.training_queue_path, 'w') as f:
                json.dump(self.training_queue, f)
        except Exception as e:
            logger.error(f"Failed to save training queue: {e}")
    
    def _load_metrics(self) -> Dict[str, Any]:
        """Load persistent metrics."""
        if self.metrics_path.exists():
            try:
                with open(self.metrics_path, 'r') as f:
                    return json.load(f)
            except:
                pass
        
        return {
            'total_concepts': 0,
            'total_associations': 0,
            'total_training_cycles': 0,
            'total_dreams': 0,
            'consciousness_score': 0.0,
            'emergence_factor': 1.0,
            'uptime_seconds': 0,
            'last_update': datetime.now().isoformat()
        }
    
    def _save_metrics(self):
        """Save metrics to disk."""
        try:
            self.metrics['last_update'] = datetime.now().isoformat()
            with open(self.metrics_path, 'w') as f:
                json.dump(self.metrics, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save metrics: {e}")
    
    def _save_state(self):
        """Save service state."""
        try:
            state = {
                'state': self.state.value,
                'start_time': self.start_time.isoformat(),
                'last_dream': self.last_dream_time,
                'last_consolidation': self.last_consolidation,
                'queue_size': len(self.training_queue)
            }
            with open(self.state_path, 'w') as f:
                json.dump(state, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save state: {e}")
    
    async def feed_data(self, data: str) -> bool:
        """
        Feed data to the biological system.
        This is async and non-blocking - data goes into queue for processing.
        """
        try:
            self.training_queue.append(data)
            self._save_training_queue()
            logger.info(f"Queued data for training (queue size: {len(self.training_queue)})")
            return True
        except Exception as e:
            logger.error(f"Failed to queue data: {e}")
            return False
    
    async def training_loop(self):
        """
        Main training loop - processes queued data.
        Runs continuously and independently.
        """
        logger.info("Starting training loop...")
        batch_size = 5
        
        while self.is_running:
            try:
                if self.training_queue and self.trainer:
                    self.state = ServiceState.LEARNING
                    
                    # Process batch
                    batch = self.training_queue[:batch_size]
                    self.training_queue = self.training_queue[batch_size:]
                    
                    # Train
                    result = await self.trainer.train_from_stream(batch)
                    
                    # Update metrics
                    stats = self.trainer.memory_system.get_stats()
                    self.metrics['total_concepts'] = stats['total_concepts']
                    self.metrics['total_associations'] = stats['total_associations']
                    self.metrics['total_training_cycles'] += 1
                    
                    if 'emergence_factor' in result:
                        self.metrics['emergence_factor'] = result['emergence_factor']
                    if 'consciousness_score' in result:
                        self.metrics['consciousness_score'] = result['consciousness_score']
                    
                    # Save state periodically
                    self._save_metrics()
                    self._save_training_queue()
                    
                    logger.info(f"Training cycle complete: {stats['total_concepts']} concepts, "
                              f"{stats['total_associations']} associations")
                    
                else:
                    self.state = ServiceState.IDLE
                    
                await asyncio.sleep(1)  # Check queue every second
                
            except Exception as e:
                logger.error(f"Training loop error: {e}")
                await asyncio.sleep(5)  # Wait before retry
    
    async def dream_loop(self):
        """
        Dream consolidation loop - runs periodically to consolidate memories.
        This simulates biological sleep/dream states.
        """
        logger.info("Starting dream loop...")
        
        while self.is_running:
            try:
                current_time = time.time()
                
                # Check if it's time to dream
                if current_time - self.last_dream_time > self.dream_interval:
                    if self.trainer and self.trainer.memory_system:
                        self.state = ServiceState.DREAMING
                        logger.info("😴 Entering dream state...")
                        
                        # Trigger dream consolidation
                        await self.trainer.memory_system.dream_consolidation()
                        
                        self.last_dream_time = current_time
                        self.metrics['total_dreams'] += 1
                        
                        logger.info("✨ Dream consolidation complete")
                        
                await asyncio.sleep(30)  # Check every 30 seconds
                
            except Exception as e:
                logger.error(f"Dream loop error: {e}")
                await asyncio.sleep(60)
    
    async def maintenance_loop(self):
        """
        Maintenance loop - handles memory consolidation and cleanup.
        """
        logger.info("Starting maintenance loop...")
        
        while self.is_running:
            try:
                current_time = time.time()
                
                # Periodic consolidation
                if current_time - self.last_consolidation > self.consolidation_interval:
                    if self.trainer:
                        self.state = ServiceState.CONSOLIDATING
                        logger.info("🔧 Performing memory consolidation...")
                        
                        # Trigger biological maintenance
                        self.trainer._biological_maintenance()
                        
                        # Save memory to disk
                        self.trainer.save_memory()
                        
                        self.last_consolidation = current_time
                        logger.info("Memory consolidation and save complete")
                        
                        # Update uptime
                        self.metrics['uptime_seconds'] = (datetime.now() - self.start_time).total_seconds()
                        self._save_metrics()
                        self._save_state()
                
                await asyncio.sleep(60)  # Check every minute
                
            except Exception as e:
                logger.error(f"Maintenance loop error: {e}")
                await asyncio.sleep(120)
    
    async def run(self):
        """
        Run the biological intelligence service.
        This starts all loops and runs until stopped.
        """
        logger.info("🧬 Biological Intelligence Service starting...")
        
        # Start all loops
        tasks = [
            asyncio.create_task(self.training_loop()),
            asyncio.create_task(self.dream_loop()),
            asyncio.create_task(self.maintenance_loop())
        ]
        
        # Setup graceful shutdown with proper task cancellation
        def signal_handler(sig, frame):
            logger.info("Shutdown signal received...")
            self.is_running = False
            # Cancel all tasks
            for task in tasks:
                task.cancel()
            
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        try:
            # Run until stopped
            await asyncio.gather(*tasks, return_exceptions=True)
        except Exception as e:
            logger.debug(f"Task completion exception (expected during shutdown): {e}")
        finally:
            # Final save before shutdown
            self.state = ServiceState.STOPPING
            logger.info("Performing final save...")
            
            try:
                if self.trainer:
                    self.trainer.save_memory()
                
                self._save_metrics()
                self._save_state()
                self._save_training_queue()
                
                logger.info("🛑 Biological Intelligence Service stopped gracefully")
            except Exception as e:
                logger.error(f"Error during final save: {e}")
                # Still consider it stopped gracefully
                logger.info("🛑 Biological Intelligence Service stopped (with save errors)")
    
    def query_knowledge(self, query: str, max_results: int = 10) -> List[Dict[str, Any]]:
        """Query the trained biological intelligence."""
        if not self.trainer:
            return []
        
        try:
            return self.trainer.query_knowledge(query, max_results=max_results)
        except Exception as e:
            logger.error(f"Query error: {e}")
            return []
    
    def get_status(self) -> Dict[str, Any]:
        """Get current service status."""
        if self.trainer:
            stats = self.trainer.memory_system.get_stats()
        else:
            stats = {}
            
        return {
            'state': self.state.value,
            'uptime': str(datetime.now() - self.start_time),
            'concepts': stats.get('total_concepts', 0),
            'associations': stats.get('total_associations', 0),
            'consciousness': self.metrics.get('consciousness_score', 0),
            'emergence': self.metrics.get('emergence_factor', 1),
            'queue_size': len(self.training_queue),
            'total_cycles': self.metrics.get('total_training_cycles', 0),
            'total_dreams': self.metrics.get('total_dreams', 0)
        }


async def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Biological Intelligence Service")
    parser.add_argument("--workspace", default="./biological_workspace",
                       help="Workspace directory for the service")
    parser.add_argument("--english", action="store_true",
                       help="Start in English learning mode")
    
    args = parser.parse_args()
    
    # Use English workspace if specified
    workspace = "./english_biological_workspace" if args.english else args.workspace
    service = BiologicalIntelligenceService(workspace_path=workspace)
    
    # Feed initial knowledge based on mode
    if args.english:
        initial_knowledge = [
            'English uses 26 letters in its alphabet.',
            'English sentences follow Subject-Verb-Object order.',
            'Words combine to form sentences that express complete thoughts.',
            'Language is a tool for communication and expression.',
            'Understanding language requires recognizing patterns and meanings.'
        ]
        logger.info("🎓 Starting English learning mode")
    else:
        initial_knowledge = [
            "Biological intelligence is a living system that evolves continuously.",
            "Knowledge forms through associations and emerges through swarm intelligence.",
            "Consciousness arises from self-referential patterns in knowledge.",
            "This system has no parameters, no gradients, and infinite capacity."
        ]
    
    for knowledge in initial_knowledge:
        await service.feed_data(knowledge)
    
    # Run the service
    await service.run()


if __name__ == "__main__":
    asyncio.run(main())