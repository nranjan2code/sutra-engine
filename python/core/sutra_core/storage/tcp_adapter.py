"""
TCP-based storage adapter for distributed storage.

Simple wrapper around sutra-storage-client-tcp providing the interface
expected by ReasoningEngine and learning components.
"""

import logging
import time
import uuid
from typing import Dict, List, Optional, Tuple, Any

from ..graph.concepts import Association, Concept, AssociationType
from ..config.system import (
    association_type_to_int,
    int_to_association_type,
    SYSTEM_CONFIG,
)

logger = logging.getLogger(__name__)


class TcpStorageAdapter:
    """
    TCP-based storage adapter for distributed deployments.
    
    Simple wrapper around StorageClient from sutra-storage-client-tcp.
    """

    def __init__(self, server_address: str, vector_dimension: int = 768):
        """
        Initialize TCP storage adapter.

        Args:
            server_address: Storage server address (host:port)
            vector_dimension: Vector dimension for HNSW
        """
        self.server_address = server_address
        self.vector_dimension = vector_dimension
        self.client = None
        
        # Import the client class
        try:
            from sutra_storage_client import StorageClient
            self.StorageClient = StorageClient
        except ImportError as e:
            logger.error(f"Failed to import TCP storage client: {e}")
            raise RuntimeError(f"TCP storage client not available: {e}")
        
        # Initialize connection
        self._connect()
    
    def _connect(self):
        """Establish or re-establish TCP connection."""
        try:
            if self.client:
                self.client.close()
            self.client = self.StorageClient(self.server_address)
            logger.info(f"TCP storage client connected to {self.server_address}")
        except Exception as e:
            logger.error(f"Failed to connect TCP storage client: {e}")
            raise RuntimeError(f"TCP storage connection failed: {e}")
    
    def _execute_with_retry(self, operation, *args, **kwargs):
        """Execute operation with automatic connection recovery."""
        max_retries = SYSTEM_CONFIG.TCP_MAX_RETRIES
        for attempt in range(max_retries):
            try:
                return operation(*args, **kwargs)
            except (ConnectionError, BrokenPipeError, OSError) as e:
                logger.warning(f"TCP connection lost (attempt {attempt + 1}/{max_retries}): {e}")
                if attempt < max_retries - 1:
                    # Exponential backoff using system config
                    delay = SYSTEM_CONFIG.TCP_RETRY_BACKOFF_BASE * (2 ** attempt)
                    logger.info(f"Waiting {delay}s before reconnection attempt...")
                    time.sleep(delay)
                    
                    logger.info("Attempting to reconnect...")
                    try:
                        self._connect()
                    except Exception as reconnect_error:
                        logger.error(f"Reconnection failed: {reconnect_error}")
                        continue
                else:
                    logger.error(f"All {max_retries} connection attempts failed")
                    raise
            except Exception as e:
                # Non-connection errors should be raised immediately
                raise

    def learn_concept(
        self,
        content: str,
        source: Optional[str] = None,
        category: Optional[str] = None,
        confidence: float = 1.0,
        strength: float = 1.0,
        generate_embedding: bool = True,
        extract_associations: bool = True,
        embedding_model: Optional[str] = None,
        min_association_confidence: float = 0.5,
        max_associations_per_concept: int = 10,
    ) -> str:
        """Learn a concept via TCP storage server (unified pipeline)."""
        options = {
            "generate_embedding": bool(generate_embedding),
            "embedding_model": embedding_model,
            "extract_associations": bool(extract_associations),
            "min_association_confidence": float(min_association_confidence),
            "max_associations_per_concept": int(max_associations_per_concept),
            "strength": float(strength),
            "confidence": float(confidence),
            # Note: 'source' and 'category' can be added to storage later
        }
        
        def _operation():
            return self.client.learn_concept_v2(
                content=content,
                options=options,
            )
        
        try:
            concept_id = self._execute_with_retry(_operation)
            logger.debug(f"Learned concept {concept_id[:8]} via TCP (unified)")
            return concept_id
            
        except Exception as e:
            logger.error(f"Failed to learn concept via TCP (unified): {e}")
            raise

    def add_concept(self, concept: Concept, embedding) -> None:
        """Add concept with its embedding via TCP storage server."""
        # Convert embedding to list if it's numpy array
        if hasattr(embedding, 'tolist'):
            embedding_list = embedding.tolist()
        elif isinstance(embedding, list):
            embedding_list = embedding
        else:
            embedding_list = []
            
        def _operation():
            return self.client.learn_concept(
                concept_id=concept.id,
                content=concept.content,
                embedding=embedding_list,
                strength=concept.strength,
                confidence=concept.confidence,
            )
        
        try:
            sequence = self._execute_with_retry(_operation)
            logger.debug(f"Added concept {concept.id[:8]} via TCP (seq={sequence})")
            
        except Exception as e:
            logger.error(f"Failed to add concept via TCP: {e}")
            raise

    def learn_association(
        self,
        source_id: str,
        target_id: str,
        association_type: AssociationType = AssociationType.SEMANTIC,
        confidence: float = 1.0,
    ) -> None:
        """Learn an association via TCP storage server."""
        # Convert AssociationType to int (centralized mapping)
        type_int = association_type_to_int(association_type)
        
        def _operation():
            return self.client.learn_association(
                source_id=source_id,
                target_id=target_id,
                assoc_type=type_int,
                confidence=confidence,
            )
        
        try:
            sequence = self._execute_with_retry(_operation)
            logger.debug(f"Learned association {source_id[:8]} -> {target_id[:8]} via TCP (seq={sequence})")
            
        except Exception as e:
            logger.error(f"Failed to learn association via TCP: {e}")
            raise

    def add_association(self, association: Association) -> None:
        """Add association via TCP storage server."""
        # Convert AssociationType to int (centralized mapping)
        type_int = association_type_to_int(association.assoc_type)
        
        def _operation():
            return self.client.learn_association(
                source_id=association.source_id,
                target_id=association.target_id,
                assoc_type=type_int,
                confidence=association.confidence,
            )
        
        try:
            sequence = self._execute_with_retry(_operation)
            logger.debug(f"Added association {association.source_id[:8]} -> {association.target_id[:8]} via TCP (seq={sequence})")
            
        except Exception as e:
            logger.error(f"Failed to add association via TCP: {e}")
            raise

    def get_concept(self, concept_id: str) -> Optional[Concept]:
        """Get concept by ID via TCP storage server."""
        def _operation():
            return self.client.query_concept(concept_id)
            
        try:
            result = self._execute_with_retry(_operation)
            
            if not result:
                return None
            
            return Concept(
                id=result["id"],
                content=result["content"],
                confidence=result.get("confidence", 1.0),
                strength=result.get("strength", 1.0),
                source=result.get("source"),
                category=result.get("category"),
                created=result.get("created", 0.0),
                last_accessed=result.get("last_accessed", 0.0),
                access_count=result.get("access_count", 0),
            )
            
        except Exception as e:
            logger.error(f"Failed to get concept {concept_id[:8]} via TCP: {e}")
            return None

    def get_neighbors(self, concept_id: str) -> List[str]:
        """Get neighboring concept IDs via TCP storage server."""
        def _operation():
            result = self.client.get_neighbors(concept_id)
            # Ensure result is a list of strings
            if isinstance(result, list):
                return result
            elif isinstance(result, dict) and 'neighbors' in result:
                return result['neighbors']
            return []
            
        try:
            return self._execute_with_retry(_operation)
            
        except Exception as e:
            logger.error(f"Failed to get neighbors for {concept_id[:8]} via TCP: {e}")
            return []

    def find_path(
        self,
        start_id: str,
        end_id: str,
        max_depth: int = 6,
    ) -> Optional[List[str]]:
        """Find path between concepts via TCP storage server."""
        def _operation():
            return self.client.find_path(start_id, end_id, max_depth)
            
        try:
            return self._execute_with_retry(_operation)
            
        except Exception as e:
            logger.error(f"Failed to find path via TCP: {e}")
            return None
    
    def find_paths(
        self,
        start_concepts: List[str],
        target_concepts: List[str],
        max_depth: int = 6,
        num_paths: int = 5,
        query: str = "",
    ) -> List[Any]:
        """Find multiple reasoning paths via TCP storage server."""
        # For now, fallback to simple pathfinding between first concepts
        # TODO: Implement full multi-path reasoning in storage server
        paths = []
        
        for start in start_concepts[:2]:  # Limit starts
            for target in target_concepts[:3]:  # Limit targets
                if start == target:
                    continue
                    
                path = self.find_path(start, target, max_depth)
                if path:
                    # Convert to ReasoningPath object
                    from ..graph.concepts import ReasoningPath, ReasoningStep
                    
                    steps = []
                    for i in range(len(path) - 1):
                        assoc = self.get_association(path[i], path[i + 1])
                        if assoc:
                            step = ReasoningStep(
                                step_number=i + 1,
                                source_concept=path[i],
                                relation=assoc.assoc_type.value,
                                target_concept=path[i + 1],
                                confidence=assoc.confidence,
                            )
                            steps.append(step)
                    
                    if steps:
                        reasoning_path = ReasoningPath(
                            steps=steps,
                            confidence=min(s.confidence for s in steps),
                        )
                        paths.append(reasoning_path)
                        
                        if len(paths) >= num_paths:
                            return paths
        
        return paths
    
    def get_association(
        self, source_id: str, target_id: str
    ) -> Optional[Association]:
        """Get association between two concepts via TCP storage server."""
        def _operation():
            return self.client.get_association(source_id, target_id)

        try:
            result = self._execute_with_retry(_operation)

            if not result:
                return None

            # Convert type int back to AssociationType (centralized mapping)
            assoc_type = int_to_association_type(result.get("type", 0))
            
            return Association(
                source_id=result.get("source_id", source_id),
                target_id=result.get("target_id", target_id),
                assoc_type=assoc_type,
                confidence=result.get("confidence", 1.0),
            )
            
        except Exception as e:
            logger.debug(f"No association found {source_id[:8]} -> {target_id[:8]}: {e}")
            return None
    
    def get_all_concept_ids(self) -> List[str]:
        """Get all concept IDs via TCP storage server."""
        def _operation():
            return self.client.get_all_concept_ids()
            
        try:
            result = self._execute_with_retry(_operation)
            if isinstance(result, list):
                return result
            elif isinstance(result, dict) and 'concept_ids' in result:
                return result['concept_ids']
            return []
            
        except Exception as e:
            logger.error(f"Failed to get all concept IDs via TCP: {e}")
            return []

    def vector_search(
        self,
        query_vector: List[float],
        k: int = 10,
    ) -> List[Tuple[str, float]]:
        """Vector similarity search via TCP storage server."""
        def _operation():
            return self.client.vector_search(query_vector, k)
            
        try:
            return self._execute_with_retry(_operation)
            
        except Exception as e:
            logger.error(f"Failed vector search via TCP: {e}")
            return []

    def contains(self, concept_id: str) -> bool:
        """Check if concept exists via TCP storage server."""
        def _operation():
            return self.client.contains(concept_id)
            
        try:
            return self._execute_with_retry(_operation)
            
        except Exception as e:
            logger.error(f"Failed to check existence for {concept_id[:8]} via TCP: {e}")
            return False

    def stats(self) -> Dict[str, Any]:
        """Get storage statistics via TCP storage server."""
        def _operation():
            return self.client.stats()
            
        try:
            return self._execute_with_retry(_operation)
            
        except Exception as e:
            logger.error(f"Failed to get stats via TCP: {e}")
            return {}

    def save(self) -> None:
        """Force flush to disk via TCP storage server."""
        def _operation():
            return self.client.flush()
            
        try:
            self._execute_with_retry(_operation)
            logger.debug("Flushed storage via TCP")
            
        except Exception as e:
            logger.warning(f"Failed to flush via TCP: {e}")

    def health_check(self) -> Dict[str, Any]:
        """Check server health via TCP."""
        def _operation():
            return self.client.health_check()
            
        try:
            return self._execute_with_retry(_operation)
            
        except Exception as e:
            logger.error(f"TCP health check failed: {e}")
            return {"status": "unhealthy", "error": str(e)}

    def close(self) -> None:
        """Close TCP connection."""
        try:
            if hasattr(self, 'client') and self.client:
                self.client.close()
            logger.debug("TCP storage connection closed")
            
        except Exception as e:
            logger.warning(f"Error closing TCP connection: {e}")