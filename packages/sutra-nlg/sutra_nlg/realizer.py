from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple
import os
import logging

from .templates import select_template

logger = logging.getLogger(__name__)

@dataclass
class NLGConfig:
    tone: str = "friendly"  # friendly | formal | concise | regulatory
    moves: List[str] = None  # e.g., ["define", "evidence"]
    seed: Optional[int] = None
    mode: str = "template"  # "template" or "hybrid" (LLM-based)
    service_url: Optional[str] = None  # NLG service URL for hybrid mode

@dataclass
class GroundedSentence:
    text: str
    grounded_to: List[str]  # list of concept_ids

class GroundingError(Exception):
    pass

class NLGRealizer:
    """Minimal grounded realizer: deterministic, template-driven.

    Inputs:
      - answer: canonical answer text (from storage)
      - reasoning_paths: list of path objects with concept_ids and concepts
    Outputs:
      - final text + grounding map
    """

    def __init__(self, config: Optional[NLGConfig] = None):
        self.config = config or NLGConfig()
        if self.config.moves is None:
            self.config.moves = ["define", "evidence"]

    def realize(self,
                query: str,
                answer: str,
                reasoning_paths: Optional[List[Dict[str, Any]]] = None) -> Tuple[str, List[GroundedSentence], Dict[str, Any]]:
        # Route to hybrid (LLM) or template mode
        if self.config.mode == "hybrid" and self.config.service_url:
            try:
                return self._realize_hybrid(query, answer, reasoning_paths)
            except Exception as e:
                logger.warning(f"Hybrid NLG failed ({e}), falling back to template")
                # Fall through to template mode
        
        return self._realize_template(query, answer, reasoning_paths)
    
    def _realize_template(self,
                         query: str,
                         answer: str,
                         reasoning_paths: Optional[List[Dict[str, Any]]] = None) -> Tuple[str, List[GroundedSentence], Dict[str, Any]]:
        """Template-based realization (original implementation)"""
        template = select_template(self.config.tone, self.config.moves)

        # slots
        lead = self._lead_phrase(self.config.tone)
        because_opt = "Because " if self._has_evidence(reasoning_paths) else ""
        evidence_1_opt, evidence_ids = self._evidence_1(reasoning_paths)

        # render
        rendered = template.pattern
        rendered = rendered.replace("{lead_friendly}", lead)
        rendered = rendered.replace("{lead_formal}", lead)
        rendered = rendered.replace("{answer}", answer.strip())
        rendered = rendered.replace("{because_opt}", because_opt)
        rendered = rendered.replace("{evidence_1_opt}", evidence_1_opt)

        sentences = [s.strip() for s in rendered.split(".") if s.strip()]

        grounded: List[GroundedSentence] = []
        for i, s in enumerate(sentences):
            if i == 0:
                grounded.append(GroundedSentence(text=s + ".", grounded_to=evidence_ids or []))
            else:
                grounded.append(GroundedSentence(text=s + ".", grounded_to=evidence_ids))

        # grounding gate: ensure all tokens come from answer/evidence pool
        self._grounding_gate(grounded, answer, reasoning_paths)

        meta = {
            "template_id": template.id,
            "tone": self.config.tone,
            "moves": self.config.moves,
            "evidence_used": evidence_ids,
        }
        final_text = " ".join([g.text for g in grounded]).strip()
        return final_text, grounded, meta

    def _lead_phrase(self, tone: str) -> str:
        if tone == "friendly":
            return "Here’s what I found:"
        if tone == "formal":
            return "According to the knowledge base,"
        if tone == "regulatory":
            return "Per documented sources,"
        return "Summary:"

    def _has_evidence(self, paths: Optional[List[Dict[str, Any]]]) -> bool:
        return bool(paths)

    def _evidence_1(self, paths: Optional[List[Dict[str, Any]]]) -> Tuple[str, List[str]]:
        if not paths:
            return "", []
        # take first concept from first path as evidence snippet
        p0 = paths[0]
        # expected shape from ExplainableResult.reasoning_paths: dicts with 'concepts' and 'concept_ids'
        concepts = p0.get("concepts") or []
        cids = p0.get("concept_ids") or []
        if concepts:
            snippet = concepts[0]
            return f"{snippet}", cids[:1]
        return "", cids[:1]

    def _grounding_gate(self, grounded: List[GroundedSentence], answer: str, paths: Optional[List[Dict[str, Any]]]):
        allowed_pool = set((answer or "").lower().split())
        if paths:
            for p in paths:
                for text in (p.get("concepts") or []):
                    allowed_pool.update((text or "").lower().split())
        # very permissive initial check: ensure at least half tokens come from pool
        for g in grounded:
            tokens = [t.strip(",;:()[]") for t in g.text.lower().split()]
            if not tokens:
                raise GroundingError("Empty sentence after realization")
            overlap = sum(1 for t in tokens if t in allowed_pool)
            if overlap / max(1, len(tokens)) < 0.5:
                raise GroundingError("Grounding check failed: low lexical overlap")
    
    def _realize_hybrid(self,
                       query: str,
                       answer: str,
                       reasoning_paths: Optional[List[Dict[str, Any]]] = None) -> Tuple[str, List[GroundedSentence], Dict[str, Any]]:
        """
        Hybrid LLM-based realization with strict grounding validation.
        
        Falls back to template if:
        1. Service unavailable
        2. Generation fails
        3. Grounding validation fails
        """
        import requests
        
        # Build fact pool from reasoning paths
        fact_pool = self._extract_fact_pool(answer, reasoning_paths)
        
        if not fact_pool:
            logger.warning("No facts available for hybrid NLG, using template")
            return self._realize_template(query, answer, reasoning_paths)
        
        # Build constrained prompt
        prompt = self._build_constrained_prompt(query, fact_pool, self.config.tone)
        
        # Call NLG service
        try:
            response = requests.post(
                f"{self.config.service_url}/generate",
                json={
                    "prompt": prompt,
                    "max_tokens": 150,
                    "temperature": 0.3,
                    "stop_sequences": ["\n\nFACTS:", "QUESTION:", "ANSWER:"]
                },
                timeout=5.0
            )
            
            if response.status_code != 200:
                logger.error(f"NLG service returned {response.status_code}")
                raise Exception(f"Service error: {response.status_code}")
            
            result = response.json()
            generated_text = result["text"].strip()
            
            # CRITICAL: Validate grounding
            if not self._validate_hybrid_grounding(generated_text, fact_pool):
                logger.warning("Hybrid NLG failed grounding validation")
                raise GroundingError("Generated text not grounded in facts")
            
            # Extract evidence IDs
            evidence_ids = []
            if reasoning_paths:
                for path in reasoning_paths:
                    evidence_ids.extend(path.get("concept_ids", [])[:3])
            
            # Build grounded sentences
            grounded = [
                GroundedSentence(
                    text=generated_text,
                    grounded_to=evidence_ids
                )
            ]
            
            meta = {
                "mode": "hybrid",
                "model": result.get("model", "unknown"),
                "tone": self.config.tone,
                "processing_time_ms": result.get("processing_time_ms", 0),
                "tokens_generated": result.get("tokens_generated", 0),
                "evidence_used": evidence_ids,
                "grounding_validated": True
            }
            
            logger.info(f"Hybrid NLG succeeded ({result.get('tokens_generated', 0)} tokens, {result.get('processing_time_ms', 0):.1f}ms)")
            
            return generated_text, grounded, meta
            
        except requests.RequestException as e:
            logger.error(f"NLG service request failed: {e}")
            raise Exception(f"Service unavailable: {e}")
    
    def _extract_fact_pool(self, answer: str, reasoning_paths: Optional[List[Dict[str, Any]]]) -> List[str]:
        """Extract verified facts from answer and reasoning paths"""
        facts = []
        
        # Add answer as primary fact
        if answer and answer.strip():
            facts.append(answer.strip())
        
        # Add concepts from reasoning paths
        if reasoning_paths:
            for path in reasoning_paths:
                concepts = path.get("concepts", [])
                for concept in concepts[:5]:  # Limit to prevent prompt overflow
                    if concept and concept.strip() and concept not in facts:
                        facts.append(concept.strip())
        
        return facts
    
    def _build_constrained_prompt(self, query: str, fact_pool: List[str], tone: str) -> str:
        """
        Build prompt that CONSTRAINS the LLM to only use verified facts.
        
        Critical: This is the primary defense against hallucinations.
        """
        facts_text = "\n".join([f"- {fact}" for fact in fact_pool])
        
        tone_instruction = {
            "friendly": "Write a friendly, conversational answer.",
            "formal": "Write a formal, professional answer.",
            "concise": "Write a brief, direct answer.",
            "regulatory": "Write a precise, compliance-ready answer."
        }.get(tone, "Write a clear answer.")
        
        prompt = f"""You are a factual answer generator. You MUST ONLY use the following verified facts to answer the question.

IMPORTANT RULES:
1. Use ONLY information from the FACTS below
2. Do NOT add any information not in the facts
3. Do NOT speculate or infer beyond the facts
4. {tone_instruction}

VERIFIED FACTS:
{facts_text}

QUESTION: {query}

ANSWER (using ONLY the verified facts above):"""
        
        return prompt
    
    def _validate_hybrid_grounding(self, generated_text: str, fact_pool: List[str]) -> bool:
        """
        Validate that generated text is grounded in fact pool.
        
        Stricter than template validation - at least 70% overlap required.
        """
        if not generated_text or not fact_pool:
            return False
        
        # Build allowed token pool
        allowed_tokens = set()
        for fact in fact_pool:
            allowed_tokens.update(fact.lower().split())
        
        # Check overlap
        generated_tokens = [t.strip(",;:()[]!?.") for t in generated_text.lower().split()]
        generated_tokens = [t for t in generated_tokens if t]  # Remove empty
        
        if not generated_tokens:
            return False
        
        overlap_count = sum(1 for t in generated_tokens if t in allowed_tokens)
        overlap_ratio = overlap_count / len(generated_tokens)
        
        # Require 70% overlap (stricter than template's 50%)
        is_valid = overlap_ratio >= 0.70
        
        if not is_valid:
            logger.warning(
                f"Grounding validation failed: {overlap_ratio:.1%} overlap "
                f"({overlap_count}/{len(generated_tokens)} tokens)"
            )
        
        return is_valid
