"""
Advanced Query Planning for Complex Reasoning.

Decomposes complex queries into multi-stage reasoning plans:
- Query decomposition into sub-questions
- Dependency analysis between steps
- Temporal and logical operators
- Dynamic plan optimization
"""

import logging
import re
from dataclasses import dataclass
from enum import Enum
from typing import Dict, List, Optional, Set

logger = logging.getLogger(__name__)


class QueryType(Enum):
    """Types of query operations."""
    
    FACTUAL = "factual"           # Direct fact lookup
    RELATIONAL = "relational"     # Relationships between concepts
    CAUSAL = "causal"             # Cause-effect chains
    TEMPORAL = "temporal"         # Time-based reasoning
    COMPARATIVE = "comparative"   # Comparisons
    HYPOTHETICAL = "hypothetical" # What-if scenarios
    AGGREGATE = "aggregate"       # Combining multiple facts


class Quantifier(Enum):
    """Logical quantifiers."""
    
    ALL = "all"
    SOME = "some"
    NONE = "none"
    MOST = "most"


@dataclass
class QueryStep:
    """A single step in a query plan."""
    
    step_id: int
    query: str
    query_type: QueryType
    dependencies: List[int]  # IDs of steps that must complete first
    quantifier: Optional[Quantifier] = None
    temporal_constraint: Optional[str] = None
    confidence_threshold: float = 0.5
    
    def __repr__(self) -> str:
        deps = f"depends on {self.dependencies}" if self.dependencies else "independent"
        return f"Step {self.step_id}: {self.query} ({deps})"


@dataclass
class QueryPlan:
    """Complete execution plan for a complex query."""
    
    original_query: str
    steps: List[QueryStep]
    execution_order: List[int]  # Topologically sorted step IDs
    estimated_complexity: float
    
    def __repr__(self) -> str:
        return (
            f"QueryPlan({len(self.steps)} steps, "
            f"complexity={self.estimated_complexity:.2f})"
        )


class QueryPlanner:
    """
    Intelligent query planner that decomposes complex queries.
    
    Handles:
    - Multi-part questions
    - Temporal reasoning
    - Logical operators (and, or, not)
    - Quantifiers (all, some, none)
    - Causal chains
    - Hypotheticals
    """
    
    def __init__(self):
        """Initialize query planner."""
        # Patterns for query decomposition
        self.decomposition_patterns = self._init_patterns()
        
        # Temporal keywords
        self.temporal_keywords = {
            "before", "after", "during", "while", "when",
            "first", "then", "next", "finally", "previously"
        }
        
        # Causal keywords
        self.causal_keywords = {
            "because", "therefore", "thus", "hence", "leads to",
            "causes", "results in", "due to", "why"
        }
        
        # Comparative keywords
        self.comparative_keywords = {
            "better", "worse", "more", "less", "compare",
            "versus", "vs", "difference", "similar", "different"
        }
        
    def _init_patterns(self) -> List[tuple]:
        """Initialize decomposition patterns."""
        return [
            # Multi-part conjunctions
            (r"(.+?)\s+and\s+(.+)", "conjunction"),
            (r"(.+?)\s+or\s+(.+)", "disjunction"),
            
            # Causal chains
            (r"why\s+(.+)", "causal"),
            (r"what\s+causes\s+(.+)", "causal"),
            (r"(.+?)\s+because\s+(.+)", "causal"),
            
            # Temporal sequences
            (r"what\s+happens\s+after\s+(.+)", "temporal"),
            (r"what\s+happens\s+before\s+(.+)", "temporal"),
            (r"(.+?)\s+then\s+(.+)", "temporal"),
            
            # Comparisons
            (r"what\s+is\s+the\s+difference\s+between\s+(.+?)\s+and\s+(.+)", "comparative"),
            (r"compare\s+(.+?)\s+(?:and|to|with)\s+(.+)", "comparative"),
            
            # Hypotheticals
            (r"what\s+if\s+(.+)", "hypothetical"),
            (r"what\s+would\s+happen\s+if\s+(.+)", "hypothetical"),
            
            # Quantifiers
            (r"do\s+all\s+(.+)", "quantified"),
            (r"are\s+all\s+(.+)", "quantified"),
            (r"do\s+some\s+(.+)", "quantified"),
        ]
    
    def plan_query(
        self,
        query: str,
        max_steps: int = 10,
        optimize: bool = True,
    ) -> QueryPlan:
        """
        Create an execution plan for a complex query.
        
        Args:
            query: Natural language query
            max_steps: Maximum number of decomposition steps
            optimize: Whether to optimize the plan
            
        Returns:
            Query execution plan
        """
        logger.debug(f"Planning query: {query[:50]}...")
        
        # Classify query type
        query_type = self._classify_query(query)
        
        # Decompose into steps
        steps = self._decompose_query(query, query_type, max_steps)
        
        # Analyze dependencies
        steps = self._analyze_dependencies(steps)
        
        # Create execution order
        execution_order = self._topological_sort(steps)
        
        # Estimate complexity
        complexity = self._estimate_complexity(steps)
        
        # Optimize if requested
        if optimize:
            steps, execution_order = self._optimize_plan(steps, execution_order)
        
        plan = QueryPlan(
            original_query=query,
            steps=steps,
            execution_order=execution_order,
            estimated_complexity=complexity,
        )
        
        logger.debug(f"Generated plan: {plan}")
        return plan
    
    def _classify_query(self, query: str) -> QueryType:
        """Classify the type of query."""
        query_lower = query.lower()
        
        # Check for temporal keywords
        if any(kw in query_lower for kw in self.temporal_keywords):
            return QueryType.TEMPORAL
        
        # Check for causal keywords
        if any(kw in query_lower for kw in self.causal_keywords):
            return QueryType.CAUSAL
        
        # Check for comparative keywords
        if any(kw in query_lower for kw in self.comparative_keywords):
            return QueryType.COMPARATIVE
        
        # Check for hypotheticals
        if query_lower.startswith("what if") or "would happen" in query_lower:
            return QueryType.HYPOTHETICAL
        
        # Check for aggregations
        if any(word in query_lower for word in ["all", "every", "most"]):
            return QueryType.AGGREGATE
        
        # Check for relationships
        if any(word in query_lower for word in ["relate", "connect", "link", "between"]):
            return QueryType.RELATIONAL
        
        # Default to factual
        return QueryType.FACTUAL
    
    def _decompose_query(
        self,
        query: str,
        query_type: QueryType,
        max_steps: int,
    ) -> List[QueryStep]:
        """Decompose query into steps."""
        steps = []
        step_id = 0
        
        # Try pattern matching for decomposition
        decomposed = self._pattern_decompose(query)
        
        if decomposed:
            # Successfully decomposed
            for sub_query, sub_type in decomposed:
                if step_id >= max_steps:
                    break
                
                step = QueryStep(
                    step_id=step_id,
                    query=sub_query,
                    query_type=self._classify_query(sub_query),
                    dependencies=[],
                )
                steps.append(step)
                step_id += 1
        else:
            # Single-step query
            step = QueryStep(
                step_id=0,
                query=query,
                query_type=query_type,
                dependencies=[],
            )
            steps.append(step)
        
        return steps
    
    def _pattern_decompose(self, query: str) -> Optional[List[tuple]]:
        """Try to decompose query using patterns."""
        for pattern, decomp_type in self.decomposition_patterns:
            match = re.search(pattern, query, re.IGNORECASE)
            if match:
                if decomp_type == "conjunction":
                    # "A and B" -> [A, B]
                    return [(match.group(1).strip(), "factual"),
                            (match.group(2).strip(), "factual")]
                
                elif decomp_type == "causal":
                    # "Why X?" -> ["What is X?", "What causes X?"]
                    if "why" in query.lower():
                        subject = match.group(1).strip()
                        return [
                            (f"What is {subject}?", "factual"),
                            (f"What causes {subject}?", "causal"),
                        ]
                
                elif decomp_type == "temporal":
                    # "What happens after X?" -> ["What is X?", "What follows X?"]
                    subject = match.group(1).strip()
                    return [
                        (f"What is {subject}?", "factual"),
                        (f"What happens next?", "temporal"),
                    ]
                
                elif decomp_type == "comparative":
                    # "Compare A and B" -> ["What is A?", "What is B?", "How do they differ?"]
                    a = match.group(1).strip()
                    b = match.group(2).strip()
                    return [
                        (f"What is {a}?", "factual"),
                        (f"What is {b}?", "factual"),
                        (f"What is the difference?", "comparative"),
                    ]
                
                elif decomp_type == "hypothetical":
                    # "What if X?" -> ["What is X?", "What are the effects?"]
                    condition = match.group(1).strip()
                    return [
                        (f"What is {condition}?", "factual"),
                        (f"What would be the effects?", "hypothetical"),
                    ]
        
        return None
    
    def _analyze_dependencies(self, steps: List[QueryStep]) -> List[QueryStep]:
        """Analyze and set dependencies between steps."""
        # For now, simple sequential dependencies
        # TODO: More sophisticated dependency analysis
        for i, step in enumerate(steps):
            if i > 0:
                # Each step depends on previous step
                step.dependencies = [i - 1]
        
        return steps
    
    def _topological_sort(self, steps: List[QueryStep]) -> List[int]:
        """
        Create execution order using topological sort.
        
        Returns:
            List of step IDs in execution order
        """
        # Build adjacency list
        graph = {step.step_id: step.dependencies for step in steps}
        
        # Count in-degrees
        in_degree = {step.step_id: len(step.dependencies) for step in steps}
        
        # Queue of steps with no dependencies
        queue = [sid for sid, degree in in_degree.items() if degree == 0]
        order = []
        
        while queue:
            # Process step with no dependencies
            current = queue.pop(0)
            order.append(current)
            
            # Reduce in-degree of dependents
            for step in steps:
                if current in step.dependencies:
                    in_degree[step.step_id] -= 1
                    if in_degree[step.step_id] == 0:
                        queue.append(step.step_id)
        
        # Check for cycles
        if len(order) != len(steps):
            logger.warning("Circular dependencies detected, using sequential order")
            return [s.step_id for s in steps]
        
        return order
    
    def _estimate_complexity(self, steps: List[QueryStep]) -> float:
        """
        Estimate query complexity.
        
        Returns:
            Complexity score (higher = more complex)
        """
        base_complexity = len(steps)
        
        # Add complexity for different query types
        type_weights = {
            QueryType.FACTUAL: 1.0,
            QueryType.RELATIONAL: 1.5,
            QueryType.CAUSAL: 2.0,
            QueryType.TEMPORAL: 2.0,
            QueryType.COMPARATIVE: 2.5,
            QueryType.HYPOTHETICAL: 3.0,
            QueryType.AGGREGATE: 2.5,
        }
        
        type_complexity = sum(type_weights.get(s.query_type, 1.0) for s in steps)
        
        # Add complexity for dependencies
        dependency_complexity = sum(len(s.dependencies) for s in steps) * 0.5
        
        return base_complexity + type_complexity + dependency_complexity
    
    def _optimize_plan(
        self,
        steps: List[QueryStep],
        execution_order: List[int],
    ) -> tuple:
        """
        Optimize the query plan.
        
        Returns:
            Tuple of (optimized_steps, optimized_order)
        """
        # TODO: Implement optimizations:
        # - Merge similar steps
        # - Parallelize independent steps
        # - Reorder for cache efficiency
        # - Prune unnecessary steps
        
        # For now, just return as-is
        return steps, execution_order
    
    def explain_plan(self, plan: QueryPlan) -> str:
        """
        Generate human-readable explanation of the plan.
        
        Args:
            plan: Query plan to explain
            
        Returns:
            Explanation string
        """
        lines = [
            f"Query Plan for: '{plan.original_query}'",
            f"Complexity: {plan.estimated_complexity:.2f}",
            f"Steps: {len(plan.steps)}",
            "",
            "Execution Order:",
        ]
        
        for i, step_id in enumerate(plan.execution_order, 1):
            step = next(s for s in plan.steps if s.step_id == step_id)
            deps = f" (after steps {step.dependencies})" if step.dependencies else ""
            lines.append(f"  {i}. [{step.query_type.value}] {step.query}{deps}")
        
        return "\n".join(lines)
