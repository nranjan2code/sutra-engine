"""
Persistence module for hybrid AI system.

Handles saving and loading of:
- Concepts and associations (from sutra-core)
- Embeddings and embedding metadata
"""

import json
import logging
from pathlib import Path
from typing import Dict, Optional

import numpy as np
from sutra_core import Concept

logger = logging.getLogger(__name__)


class HybridStorage:
    """Storage handler for hybrid AI system."""

    def __init__(self, storage_path: Path):
        """
        Initialize storage handler.

        Args:
            storage_path: Directory for storing knowledge
        """
        self.storage_path = storage_path
        self.storage_path.mkdir(parents=True, exist_ok=True)

    def save(
        self,
        concepts: Dict[str, Concept],
        associations: dict,
        embeddings: Dict[str, np.ndarray],
        embedding_provider_name: str,
        vocabulary: dict = None,
        vectorizer_state: Optional[bytes] = None,
    ) -> None:
        """
        Save all hybrid AI data to disk.

        Args:
            concepts: Dictionary of concepts
            associations: Dictionary of associations
            embeddings: Dictionary of concept embeddings
            embedding_provider_name: Name of embedding provider
            vocabulary: DEPRECATED - Legacy vocabulary dict
            vectorizer_state: Pickled TF-IDF vectorizer state (optional)
        """
        # Save concepts
        concepts_data = {cid: concept.to_dict() for cid, concept in concepts.items()}
        concepts_file = self.storage_path / "concepts.json"
        with open(concepts_file, "w") as f:
            json.dump(concepts_data, f, indent=2)

        # Save associations
        associations_data = {}
        for key, assoc in associations.items():
            # Convert tuple key to string
            str_key = f"{key[0]}:{key[1]}"
            associations_data[str_key] = assoc.to_dict()

        associations_file = self.storage_path / "associations.json"
        with open(associations_file, "w") as f:
            json.dump(associations_data, f, indent=2)

        # Save embeddings
        embeddings_data = {
            "provider": embedding_provider_name,
            "embeddings": {cid: emb.tolist() for cid, emb in embeddings.items()},
            "vocabulary": vocabulary if vocabulary else {},  # Legacy support
        }
        embeddings_file = self.storage_path / "embeddings.json"
        with open(embeddings_file, "w") as f:
            json.dump(embeddings_data, f, indent=2)

        # Save TF-IDF vectorizer state (binary pickle file)
        if vectorizer_state is not None:
            vectorizer_file = self.storage_path / "tfidf_vectorizer.pkl"
            with open(vectorizer_file, "wb") as f:
                f.write(vectorizer_state)
            logger.info(
                f"Saved TF-IDF vectorizer state " f"({len(vectorizer_state)} bytes)"
            )

        logger.info(
            f"Saved {len(concepts)} concepts, "
            f"{len(associations)} associations, "
            f"{len(embeddings)} embeddings to {self.storage_path}"
        )

    def load(self) -> tuple:
        """
        Load hybrid AI data from disk.

        Returns:
            Tuple of (concepts, associations, embeddings,
                provider_name, vocabulary, vectorizer_state)
        """
        concepts = {}
        associations = {}
        embeddings = {}
        provider_name = "unknown"
        vocabulary = {}
        vectorizer_state = None

        # Load concepts
        concepts_file = self.storage_path / "concepts.json"
        if concepts_file.exists():
            with open(concepts_file, "r") as f:
                concepts_data = json.load(f)
                concepts = {
                    cid: Concept.from_dict(data) for cid, data in concepts_data.items()
                }

        # Load associations
        associations_file = self.storage_path / "associations.json"
        if associations_file.exists():
            with open(associations_file, "r") as f:
                from sutra_core import Association

                associations_data = json.load(f)
                for str_key, assoc_data in associations_data.items():
                    # Convert string key back to tuple
                    if ":" in str_key:
                        source_id, target_id = str_key.split(":", 1)
                    elif "|" in str_key:
                        source_id, target_id = str_key.split("|", 1)
                    else:
                        # Unknown format; skip
                        continue
                    key = (source_id, target_id)
                    associations[key] = Association.from_dict(assoc_data)

        # Load embeddings
        embeddings_file = self.storage_path / "embeddings.json"
        if embeddings_file.exists():
            with open(embeddings_file, "r") as f:
                embeddings_data = json.load(f)
                provider_name = embeddings_data.get("provider", "unknown")
                embeddings = {
                    cid: np.array(emb)
                    for cid, emb in embeddings_data.get("embeddings", {}).items()
                }
                vocabulary = embeddings_data.get("vocabulary", {})  # Legacy

        # Load TF-IDF vectorizer state (binary pickle file)
        vectorizer_file = self.storage_path / "tfidf_vectorizer.pkl"
        if vectorizer_file.exists():
            with open(vectorizer_file, "rb") as f:
                vectorizer_state = f.read()
            logger.info(
                f"Loaded TF-IDF vectorizer state " f"({len(vectorizer_state)} bytes)"
            )

        logger.info(
            f"Loaded {len(concepts)} concepts, "
            f"{len(associations)} associations, "
            f"{len(embeddings)} embeddings from {self.storage_path}"
        )

        return (
            concepts,
            associations,
            embeddings,
            provider_name,
            vocabulary,
            vectorizer_state,
        )

    def exists(self) -> bool:
        """
        Check if saved data exists.

        Returns:
            True if any save files exist
        """
        return (
            (self.storage_path / "concepts.json").exists()
            or (self.storage_path / "associations.json").exists()
            or (self.storage_path / "embeddings.json").exists()
        )
