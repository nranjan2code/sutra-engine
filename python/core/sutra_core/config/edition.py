"""
Edition Configuration System - Single Source of Truth

Centralized edition specifications eliminating scattered configuration.
Replaces duplicate definitions in feature_flags.py, config.py, Docker Compose.
"""

import os
import logging
from enum import Enum
from dataclasses import dataclass
from typing import Optional

logger = logging.getLogger(__name__)


class Edition(Enum):
    """Sutra AI editions"""
    SIMPLE = "simple"        # Free - single-node, low limits
    COMMUNITY = "community"  # $99/mo - single-node, higher limits
    ENTERPRISE = "enterprise"  # $999/mo - HA + Grid + highest limits


@dataclass(frozen=True)
class EditionSpec:
    """
    Complete edition specification - single source of truth.

    All edition-related configuration unified here.
    """
    # Identity
    edition: Edition
    price_usd: int

    # API rate limits (per minute)
    learn_per_min: int
    reason_per_min: int

    # Ingestion limits
    ingest_workers: int
    max_dataset_gb: int  # 0 = unlimited

    # Storage limits
    max_concepts: int
    retention_days: int  # Audit log retention

    # Topology (deployment architecture)
    num_shards: int  # Storage sharding
    embedding_replicas: int  # HA embedding service replicas
    nlg_replicas: int  # HA NLG service replicas
    ha_enabled: bool
    grid_enabled: bool

    # Security
    secure_mode_required: bool

    # Support SLA
    support_sla_hours: Optional[int]  # Response time in hours, None = community

    # Models
    embedding_model: str
    nlg_model: str

    def validate(self) -> None:
        """
        Validate edition spec for internal consistency.

        Raises:
            ValueError: If configuration is invalid
        """
        # Enterprise must have sharding
        if self.edition == Edition.ENTERPRISE and self.num_shards < 4:
            raise ValueError(
                f"Enterprise edition requires num_shards >= 4, got {self.num_shards}"
            )

        # HA requires multiple replicas
        if self.ha_enabled and (self.embedding_replicas < 2 or self.nlg_replicas < 2):
            raise ValueError(
                f"HA enabled but insufficient replicas: "
                f"embedding={self.embedding_replicas}, nlg={self.nlg_replicas}"
            )

        # Grid requires enterprise
        if self.grid_enabled and self.edition != Edition.ENTERPRISE:
            raise ValueError(
                f"Grid is enterprise-only feature, got edition={self.edition.value}"
            )

        logger.debug(f"Validated edition spec: {self.edition.value}")


# ============================================================================
# Edition Specifications - Single Source of Truth
# ============================================================================

EDITION_SPECS = {
    Edition.SIMPLE: EditionSpec(
        edition=Edition.SIMPLE,
        price_usd=0,
        learn_per_min=10,
        reason_per_min=50,
        ingest_workers=2,
        max_dataset_gb=1,
        max_concepts=100_000,
        retention_days=7,
        num_shards=1,
        embedding_replicas=1,
        nlg_replicas=1,
        ha_enabled=False,
        grid_enabled=False,
        secure_mode_required=False,
        support_sla_hours=None,  # Community support only
        embedding_model="nomic-embed-text-v1.5",
        nlg_model="sutraworks-rwkv-169m",
    ),
    Edition.COMMUNITY: EditionSpec(
        edition=Edition.COMMUNITY,
        price_usd=99,
        learn_per_min=100,
        reason_per_min=500,
        ingest_workers=4,
        max_dataset_gb=10,
        max_concepts=1_000_000,
        retention_days=30,
        num_shards=1,
        embedding_replicas=3,  # HA embedding
        nlg_replicas=3,  # HA NLG
        ha_enabled=True,
        grid_enabled=False,
        secure_mode_required=False,
        support_sla_hours=48,  # 48-hour email support
        embedding_model="nomic-embed-text-v1.5",
        nlg_model="sutraworks-rwkv-430m",
    ),
    Edition.ENTERPRISE: EditionSpec(
        edition=Edition.ENTERPRISE,
        price_usd=999,
        learn_per_min=1000,
        reason_per_min=5000,
        ingest_workers=16,
        max_dataset_gb=0,  # Unlimited
        max_concepts=10_000_000,
        retention_days=365,
        num_shards=16,  # Sharded storage for scale
        embedding_replicas=3,
        nlg_replicas=3,
        ha_enabled=True,
        grid_enabled=True,
        secure_mode_required=True,
        support_sla_hours=4,  # 4-hour dedicated support
        embedding_model="nomic-embed-text-v1.5",
        nlg_model="sutraworks-rwkv-1.5b",
    ),
}


def get_edition_spec(
    edition_override: Optional[str] = None,
    env_var: str = "SUTRA_EDITION",
) -> EditionSpec:
    """
    Get edition specification from environment or override.

    Args:
        edition_override: Explicit edition string (overrides env var)
        env_var: Environment variable name (default: SUTRA_EDITION)

    Returns:
        EditionSpec for the requested edition

    Raises:
        ValueError: If edition is invalid
    """
    edition_str = edition_override or os.getenv(env_var, "simple")

    try:
        edition = Edition(edition_str.lower())
    except ValueError:
        raise ValueError(
            f"Invalid edition: {edition_str}. "
            f"Must be one of: {', '.join(e.value for e in Edition)}"
        )

    spec = EDITION_SPECS[edition]
    spec.validate()  # Always validate on access

    logger.info(
        f"Edition: {edition.value.upper()} "
        f"(${spec.price_usd}/mo, {spec.max_concepts:,} concepts, "
        f"{spec.num_shards} shards)"
    )

    return spec


def validate_edition_consistency(
    edition: Edition,
    num_shards: Optional[int] = None,
    embedding_replicas: Optional[int] = None,
) -> None:
    """
    Validate that runtime configuration matches edition spec.

    Prevents misconfigurations like:
    - SUTRA_EDITION=enterprise with SUTRA_NUM_SHARDS=1
    - SUTRA_EDITION=simple with 16 shards

    Args:
        edition: Edition enum
        num_shards: Actual shard count (from env or config)
        embedding_replicas: Actual embedding replica count

    Raises:
        ValueError: If configuration doesn't match edition spec
    """
    spec = EDITION_SPECS[edition]

    errors = []

    # Validate shard count
    if num_shards is not None:
        if edition == Edition.ENTERPRISE and num_shards < 4:
            errors.append(
                f"Enterprise edition requires ≥4 shards, got {num_shards}"
            )
        elif edition != Edition.ENTERPRISE and num_shards > 1:
            errors.append(
                f"{edition.value.capitalize()} edition is single-node "
                f"(num_shards=1), got {num_shards}"
            )

    # Validate HA configuration
    if embedding_replicas is not None:
        if spec.ha_enabled and embedding_replicas < 2:
            errors.append(
                f"{edition.value.capitalize()} edition requires HA "
                f"(≥2 replicas), got {embedding_replicas}"
            )

    if errors:
        raise ValueError(
            f"Edition configuration mismatch:\n" + "\n".join(f"  • {e}" for e in errors)
        )

    logger.debug(f"Edition consistency validated: {edition.value}")
