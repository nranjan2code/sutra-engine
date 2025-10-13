#!/usr/bin/env python3
"""
🥧 PI-OPTIMIZED BIOLOGICAL INTELLIGENCE SERVICE
Enhanced biological service with Pi-specific optimizations and thermal management.

This service automatically:
- Detects Pi hardware and adjusts performance
- Monitors CPU temperature and throttles when hot
- Uses external HDD for persistence when available
- Optimizes memory usage for 8GB Pi 5
- Provides thermal emergency shutdown
"""

import asyncio
import time
import logging
from pathlib import Path
from datetime import datetime

# Import Pi configuration
try:
    from pi_config import PiConfig, PI_BIOLOGICAL_CONFIG
    PI_MODE = True
except ImportError:
    PI_MODE = False
    PiConfig = None

# Import base biological service
from biological_service import BiologicalIntelligenceService

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('pi_biological_intelligence.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger('PiBiologicalService')


class PiBiologicalService(BiologicalIntelligenceService):
    """
    Pi-optimized biological intelligence service.
    Extends the base service with Pi-specific features.
    """
    
    def __init__(self, workspace_path: str = None):
        # Use Pi-optimized workspace path if not specified
        if workspace_path is None and PI_MODE:
            workspace_path = str(Path(PiConfig.HDD_WORKSPACE) / "biological_workspace")
        elif workspace_path is None:
            workspace_path = "./biological_workspace"
            
        super().__init__(workspace_path)
        
        # Pi-specific settings
        self.pi_mode = PI_MODE
        self.last_thermal_check = 0
        self.thermal_throttled = False
        self.emergency_shutdown_temp = 85.0  # Critical temperature
        
        # Adjust intervals for Pi hardware
        if PI_MODE:
            self.dream_interval = PiConfig.DREAM_INTERVAL
            self.consolidation_interval = PiConfig.MAINTENANCE_INTERVAL
            logger.info(f"🥧 Pi mode enabled - adjusted intervals (dream: {self.dream_interval}s, maintenance: {self.consolidation_interval}s)")
        
        logger.info(f"🧬 Pi Biological Service initialized at {self.workspace}")
    
    def _initialize_trainer(self):
        """Initialize trainer with Pi-optimized settings."""
        try:
            from src.biological_trainer import BiologicalTrainer
            # Use core architecture directly
            
            if PI_MODE:
                # Use core architecture with Pi-specific workspace
                self.trainer = BiologicalTrainer(
                    base_path=str(self.workspace),
                    workspace_id="pi_core",
                    use_full_swarm=True
                )
                logger.info("🥧 Using Pi-optimized biological trainer with core architecture")
            else:
                # Fall back to standard core configuration
                self.trainer = BiologicalTrainer(
                    base_path=str(self.workspace),
                    workspace_id="core",
                    use_full_swarm=True
                )
                logger.info("🖥️ Using standard biological trainer with core architecture")
            
            # Try to load existing memory
            if self.memory_path.exists():
                logger.info("Loading existing biological memory...")
                self.trainer.load_memory()
                stats = self.trainer.memory_system.get_stats()
                logger.info(f"Restored {stats['total_concepts']} concepts, {stats['total_associations']} associations")
            else:
                logger.info("Starting with fresh biological memory")
                
        except Exception as e:
            logger.error(f"Failed to initialize Pi trainer: {e}")
            self.trainer = None
    
    async def thermal_monitor(self):
        """Monitor Pi temperature and manage thermal throttling."""
        if not PI_MODE:
            return
            
        logger.info("🌡️ Starting thermal monitoring...")
        
        while self.is_running:
            try:
                current_temp = PiConfig.get_cpu_temp()
                current_time = time.time()
                
                # Check temperature every 30 seconds
                if current_time - self.last_thermal_check > 30:
                    self.last_thermal_check = current_time
                    
                    if current_temp > self.emergency_shutdown_temp:
                        logger.critical(f"🔥 EMERGENCY THERMAL SHUTDOWN! Temperature: {current_temp:.1f}°C")
                        await self.emergency_thermal_shutdown()
                        break
                    
                    elif current_temp > PiConfig.TEMP_CRITICAL_C:
                        if not self.thermal_throttled:
                            logger.warning(f"🔥 Thermal throttling activated at {current_temp:.1f}°C")
                            await self.activate_thermal_throttling()
                        
                    elif current_temp < PiConfig.TEMP_WARNING_C and self.thermal_throttled:
                        logger.info(f"❄️ Temperature normal ({current_temp:.1f}°C), removing throttling")
                        await self.deactivate_thermal_throttling()
                    
                    # Log temperature periodically
                    if current_time % 300 < 30:  # Every 5 minutes
                        memory_info = PiConfig.get_memory_info()
                        logger.info(f"🥧 Pi Status - Temp: {current_temp:.1f}°C, RAM: {memory_info['percent']:.1f}%")
                
                await asyncio.sleep(30)
                
            except Exception as e:
                logger.error(f"Thermal monitor error: {e}")
                await asyncio.sleep(60)
    
    async def activate_thermal_throttling(self):
        """Activate thermal throttling to reduce CPU load."""
        self.thermal_throttled = True
        
        if self.trainer:
            # Reduce batch sizes
            original_batch = getattr(self.trainer, 'training_batch_size', 5)
            new_batch = max(1, original_batch // 2)
            self.trainer.training_batch_size = new_batch
            
            # Increase intervals
            self.dream_interval = int(self.dream_interval * 1.5)
            self.consolidation_interval = int(self.consolidation_interval * 1.5)
            
            logger.warning(f"🐌 Thermal throttling: batch_size={new_batch}, intervals increased")
    
    async def deactivate_thermal_throttling(self):
        """Deactivate thermal throttling and return to normal performance."""
        self.thermal_throttled = False
        
        if self.trainer and PI_MODE:
            # Restore normal batch sizes and intervals
            self.trainer.training_batch_size = PiConfig.TRAINING_BATCH_SIZE
            self.dream_interval = PiConfig.DREAM_INTERVAL
            self.consolidation_interval = PiConfig.MAINTENANCE_INTERVAL
            
            logger.info("🚀 Thermal throttling deactivated, normal performance restored")
    
    async def emergency_thermal_shutdown(self):
        """Emergency shutdown due to critical temperature."""
        logger.critical("💥 EMERGENCY THERMAL SHUTDOWN INITIATED!")
        
        # Stop all learning immediately
        self.is_running = False
        
        # Save current state immediately
        if self.trainer:
            try:
                logger.info("💾 Emergency save in progress...")
                self.trainer.save_memory()
                self._save_metrics()
                self._save_state()
                self._save_training_queue()
                logger.info("✅ Emergency save completed")
            except Exception as e:
                logger.error(f"❌ Emergency save failed: {e}")
        
        # Wait for system to cool down
        logger.info("❄️ Waiting for system to cool down...")
        await asyncio.sleep(300)  # Wait 5 minutes
        
        # Check if temperature has dropped
        if PI_MODE:
            temp = PiConfig.get_cpu_temp()
            if temp < PiConfig.TEMP_WARNING_C:
                logger.info(f"🌡️ Temperature normalized ({temp:.1f}°C), service can be restarted")
            else:
                logger.warning(f"🔥 Temperature still high ({temp:.1f}°C), manual intervention required")
    
    async def pi_hardware_monitor(self):
        """Monitor Pi hardware resources."""
        if not PI_MODE:
            return
            
        logger.info("🔧 Starting Pi hardware monitoring...")
        
        while self.is_running:
            try:
                # Get comprehensive Pi status
                pi_status = PiConfig.get_system_status()
                
                # Check memory usage
                memory_percent = pi_status['memory']['percent']
                if memory_percent > 90:
                    logger.warning(f"⚠️ High memory usage: {memory_percent:.1f}%")
                    # Force garbage collection
                    import gc
                    gc.collect()
                
                # Check HDD space
                disk_info = pi_status['disk']
                if 'hdd' in disk_info and 'percent' in disk_info['hdd']:
                    hdd_percent = disk_info['hdd']['percent']
                    if hdd_percent > 90:
                        logger.warning(f"⚠️ HDD space low: {hdd_percent:.1f}% used")
                        # Trigger backup cleanup
                        await self.cleanup_old_backups()
                
                # Update metrics with Pi-specific data
                self.metrics.update({
                    'pi_cpu_temp': pi_status['hardware']['cpu_temp'],
                    'pi_memory_percent': memory_percent,
                    'pi_thermal_throttled': self.thermal_throttled
                })
                
                await asyncio.sleep(60)  # Check every minute
                
            except Exception as e:
                logger.error(f"Pi hardware monitor error: {e}")
                await asyncio.sleep(120)
    
    async def cleanup_old_backups(self):
        """Clean up old backup files to free space."""
        if not PI_MODE:
            return
            
        backup_dir = Path(PiConfig.HDD_BACKUPS)
        if backup_dir.exists():
            try:
                # Remove backups older than 14 days
                import time
                cutoff = time.time() - (14 * 24 * 60 * 60)  # 14 days ago
                
                cleaned = 0
                for backup_file in backup_dir.glob("**/*.tar.gz"):
                    if backup_file.stat().st_mtime < cutoff:
                        backup_file.unlink()
                        cleaned += 1
                
                if cleaned > 0:
                    logger.info(f"🧹 Cleaned up {cleaned} old backup files")
                    
            except Exception as e:
                logger.error(f"Backup cleanup error: {e}")
    
    async def run(self):
        """Run the Pi-optimized biological intelligence service."""
        logger.info("🧬 Pi Biological Intelligence Service starting...")
        
        # Start base service tasks
        base_tasks = [
            asyncio.create_task(self.training_loop()),
            asyncio.create_task(self.dream_loop()),
            asyncio.create_task(self.maintenance_loop())
        ]
        
        # Add Pi-specific monitoring tasks
        pi_tasks = []
        if PI_MODE:
            pi_tasks = [
                asyncio.create_task(self.thermal_monitor()),
                asyncio.create_task(self.pi_hardware_monitor())
            ]
        
        all_tasks = base_tasks + pi_tasks
        
        # Setup signal handlers
        def signal_handler(sig, frame):
            logger.info("🛑 Shutdown signal received...")
            self.is_running = False
            
        import signal
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        try:
            # Run all tasks
            await asyncio.gather(*all_tasks)
        except asyncio.CancelledError:
            pass
        except Exception as e:
            logger.error(f"Service error: {e}")
        finally:
            # Final save before shutdown
            self.state = self.ServiceState.STOPPING
            logger.info("💾 Performing final save...")
            
            if self.trainer:
                self.trainer.save_memory()
            
            self._save_metrics()
            self._save_state()
            self._save_training_queue()
            
            logger.info("🛑 Pi Biological Intelligence Service stopped gracefully")


async def main():
    """Main entry point for Pi biological service."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Pi Biological Intelligence Service")
    parser.add_argument("--workspace", default=None,
                       help="Workspace directory for the service")
    parser.add_argument("--english", action="store_true",
                       help="Start in English learning mode")
    
    args = parser.parse_args()
    
    # Determine workspace
    if args.english:
        if PI_MODE:
            workspace = str(Path(PiConfig.HDD_WORKSPACE) / "english_biological_workspace")
        else:
            workspace = "./english_biological_workspace"
    else:
        workspace = args.workspace
    
    # Create and run service
    service = PiBiologicalService(workspace_path=workspace)
    
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
            "This system has no parameters, no gradients, and infinite capacity.",
            "Pi hardware can support full consciousness emergence."
        ]
    
    for knowledge in initial_knowledge:
        await service.feed_data(knowledge)
    
    # Run the service
    await service.run()


if __name__ == "__main__":
    asyncio.run(main())