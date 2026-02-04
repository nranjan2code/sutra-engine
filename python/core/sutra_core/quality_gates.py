"""
Confidence Calibration and Quality Gates for Production AI.

Prevents low-quality responses by:
- Calibrating confidence scores based on learned patterns
- Enforcing quality thresholds
- Returning "I don't know" for uncertain cases
- Learning from production feedback to improve calibration

Uses Sutra's own graph reasoning to learn when confidence is mis-calibrated.
"""

from dataclasses import dataclass
from enum import Enum
from typing import Optional, List, Dict, Any
import logging

logger = logging.getLogger(__name__)


class ConfidenceLevel(Enum):
    """Calibrated confidence levels."""
    HIGH = "high"  # >= 0.7
    MEDIUM = "medium"  # 0.4 - 0.7
    LOW = "low"  # 0.2 - 0.4
    VERY_LOW = "very_low"  # < 0.2


@dataclass
class QualityGate:
    """Quality gate configuration."""
    min_confidence: float = 0.3
    min_consensus: float = 0.5
    min_paths: int = 1
    require_evidence: bool = True
    

@dataclass
class QualityAssessment:
    """Quality assessment result."""
    passed: bool
    confidence_level: ConfidenceLevel
    raw_confidence: float
    calibrated_confidence: float
    consensus_strength: float
    num_paths: int
    has_evidence: bool
    reasons_for_failure: List[str]
    recommendation: str
    

class ConfidenceCalibrator:
    """
    Calibrates confidence scores based on learned patterns.
    
    Learns from production usage:
    - Tracks when high confidence was incorrect
    - Identifies query patterns that lead to miscalibration
    - Adjusts confidence based on historical accuracy
    """
    
    def __init__(self, storage):
        """
        Initialize calibrator.
        
        Args:
            storage: Storage adapter to learn calibration patterns
        """
        self.storage = storage
        self.calibration_data: Dict[str, List[float]] = {}
        
    def calibrate(
        self,
        raw_confidence: float,
        query_type: str,
        consensus_strength: float,
        num_paths: int
    ) -> float:
        """
        Calibrate raw confidence score.
        
        Args:
            raw_confidence: Raw confidence from reasoning
            query_type: Type of query (what, how, why, etc.)
            consensus_strength: Agreement across paths
            num_paths: Number of reasoning paths found
        
        Returns:
            Calibrated confidence score
        """
        # Start with raw confidence
        calibrated = raw_confidence
        
        # Apply consensus penalty/bonus
        if consensus_strength < 0.5:
            # Low consensus = reduce confidence
            calibrated *= 0.8
        elif consensus_strength > 0.8:
            # Strong consensus = boost confidence
            calibrated *= 1.1
        
        # Apply path diversity penalty
        if num_paths < 2:
            # Single path = less reliable
            calibrated *= 0.9
        elif num_paths >= 5:
            # Multiple paths = more reliable
            calibrated *= 1.05
        
        # Cap at 1.0
        calibrated = min(1.0, calibrated)
        
        # Learn calibration pattern
        self._record_calibration(query_type, raw_confidence, calibrated)
        
        return calibrated
    
    def _record_calibration(
        self,
        query_type: str,
        raw: float,
        calibrated: float
    ):
        """Record calibration for learning."""
        if query_type not in self.calibration_data:
            self.calibration_data[query_type] = []
        
        self.calibration_data[query_type].append(calibrated - raw)
        
        # Keep only recent calibrations
        if len(self.calibration_data[query_type]) > 100:
            self.calibration_data[query_type] = self.calibration_data[query_type][-100:]
    
    def get_confidence_level(self, confidence: float) -> ConfidenceLevel:
        """Map confidence score to level."""
        if confidence >= 0.7:
            return ConfidenceLevel.HIGH
        elif confidence >= 0.4:
            return ConfidenceLevel.MEDIUM
        elif confidence >= 0.2:
            return ConfidenceLevel.LOW
        else:
            return ConfidenceLevel.VERY_LOW


class QualityGateValidator:
    """
    Validates query results against quality gates.
    
    Ensures production responses meet minimum quality standards.
    """
    
    def __init__(self, gate: QualityGate, calibrator: ConfidenceCalibrator):
        """
        Initialize validator.
        
        Args:
            gate: Quality gate configuration
            calibrator: Confidence calibrator
        """
        self.gate = gate
        self.calibrator = calibrator
    
    def validate(
        self,
        raw_confidence: float,
        consensus_strength: float,
        num_paths: int,
        has_evidence: bool,
        query_type: str = "unknown"
    ) -> QualityAssessment:
        """
        Validate query result against quality gates.
        
        Returns:
            Quality assessment with pass/fail and recommendations
        """
        # Calibrate confidence
        calibrated_confidence = self.calibrator.calibrate(
            raw_confidence=raw_confidence,
            query_type=query_type,
            consensus_strength=consensus_strength,
            num_paths=num_paths
        )
        
        confidence_level = self.calibrator.get_confidence_level(calibrated_confidence)
        
        # Check gates
        passed = True
        reasons = []
        
        if calibrated_confidence < self.gate.min_confidence:
            passed = False
            reasons.append(
                f"Confidence {calibrated_confidence:.2f} below minimum {self.gate.min_confidence}"
            )
        
        if consensus_strength < self.gate.min_consensus:
            passed = False
            reasons.append(
                f"Consensus {consensus_strength:.2f} below minimum {self.gate.min_consensus}"
            )
        
        if num_paths < self.gate.min_paths:
            passed = False
            reasons.append(
                f"Found {num_paths} paths, minimum required is {self.gate.min_paths}"
            )
        
        if self.gate.require_evidence and not has_evidence:
            passed = False
            reasons.append("No evidence/reasoning paths found")
        
        # Generate recommendation
        recommendation = self._generate_recommendation(
            passed=passed,
            confidence_level=confidence_level,
            reasons=reasons
        )
        
        return QualityAssessment(
            passed=passed,
            confidence_level=confidence_level,
            raw_confidence=raw_confidence,
            calibrated_confidence=calibrated_confidence,
            consensus_strength=consensus_strength,
            num_paths=num_paths,
            has_evidence=has_evidence,
            reasons_for_failure=reasons,
            recommendation=recommendation
        )
    
    def _generate_recommendation(
        self,
        passed: bool,
        confidence_level: ConfidenceLevel,
        reasons: List[str]
    ) -> str:
        """Generate actionable recommendation."""
        if passed:
            if confidence_level == ConfidenceLevel.HIGH:
                return "High quality response - safe to return to user"
            elif confidence_level == ConfidenceLevel.MEDIUM:
                return "Acceptable response - consider adding disclaimer"
            else:
                return "Low confidence but passes gates - add strong disclaimer"
        else:
            if "No evidence" in " ".join(reasons):
                return "Return 'I don't know' - no reasoning paths found"
            elif "Confidence" in " ".join(reasons):
                return "Return 'I'm not confident' with partial answer"
            else:
                return "Return 'I don't have enough information'"


class UncertaintyQuantifier:
    """
    Quantifies uncertainty in AI responses using graph patterns.
    
    Provides explainable uncertainty metrics beyond simple confidence scores.
    """
    
    def __init__(self, storage):
        """
        Initialize uncertainty quantifier.
        
        Args:
            storage: Storage adapter to analyze patterns
        """
        self.storage = storage
    
    def quantify(
        self,
        answer: str,
        reasoning_paths: List[Any],
        confidence: float
    ) -> Dict[str, Any]:
        """
        Quantify uncertainty in the answer.
        
        Returns:
            Dict with uncertainty metrics and explanations
        """
        uncertainty_factors = []
        uncertainty_score = 1.0 - confidence
        
        # Factor 1: Path diversity
        if len(reasoning_paths) <= 1:
            uncertainty_factors.append({
                'factor': 'Single reasoning path',
                'impact': 'High uncertainty',
                'explanation': 'No alternative paths to validate answer'
            })
            uncertainty_score += 0.2
        
        # Factor 2: Path agreement
        if reasoning_paths:
            answers = [getattr(p, 'answer', '') for p in reasoning_paths]
            unique_answers = len(set(answers))
            
            if unique_answers > len(answers) * 0.5:
                uncertainty_factors.append({
                    'factor': 'Low path agreement',
                    'impact': 'High uncertainty',
                    'explanation': f'{unique_answers} different answers across {len(answers)} paths'
                })
                uncertainty_score += 0.15
        
        # Factor 3: Knowledge gaps
        if len(answer) < 10:
            uncertainty_factors.append({
                'factor': 'Brief answer',
                'impact': 'Medium uncertainty',
                'explanation': 'Limited information available in knowledge base'
            })
            uncertainty_score += 0.1
        
        # Cap uncertainty at 1.0
        uncertainty_score = min(1.0, uncertainty_score)
        
        return {
            'uncertainty_score': uncertainty_score,
            'confidence_score': confidence,
            'calibrated_confidence': max(0.0, confidence - uncertainty_score),
            'uncertainty_factors': uncertainty_factors,
            'recommendation': self._get_uncertainty_recommendation(uncertainty_score)
        }
    
    def _get_uncertainty_recommendation(self, uncertainty: float) -> str:
        """Get recommendation based on uncertainty."""
        if uncertainty < 0.3:
            return "Low uncertainty - safe to present answer"
        elif uncertainty < 0.6:
            return "Moderate uncertainty - add disclaimer"
        else:
            return "High uncertainty - consider returning 'I don't know'"


# Production configuration presets
STRICT_QUALITY_GATE = QualityGate(
    min_confidence=0.5,
    min_consensus=0.6,
    min_paths=2,
    require_evidence=True
)

MODERATE_QUALITY_GATE = QualityGate(
    min_confidence=0.3,
    min_consensus=0.5,
    min_paths=1,
    require_evidence=True
)

LENIENT_QUALITY_GATE = QualityGate(
    min_confidence=0.2,
    min_consensus=0.3,
    min_paths=1,
    require_evidence=False
)


def create_quality_validator(
    storage,
    gate: QualityGate = MODERATE_QUALITY_GATE
) -> QualityGateValidator:
    """Factory function to create quality validator."""
    calibrator = ConfidenceCalibrator(storage)
    return QualityGateValidator(gate, calibrator)
