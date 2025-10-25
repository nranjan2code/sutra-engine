/// Production-Grade Semantic Pathfinding
/// 
/// Graph traversal with semantic awareness - filters applied during traversal
/// for zero memory overhead. Supports temporal constraints, causal chains,
/// and domain-scoped search.

use super::query::{SemanticFilter, TemporalConstraint, CausalFilter};
use super::types::*;
use crate::read_view::{GraphSnapshot, ConceptNode};
use crate::types::ConceptId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// Result of semantic pathfinding
#[derive(Debug, Clone)]
pub struct SemanticPath {
    /// Concepts in the path
    pub concepts: Vec<ConceptId>,
    
    /// Overall confidence score
    pub confidence: f32,
    
    /// Semantic type distribution along path
    pub type_distribution: HashMap<SemanticType, usize>,
    
    /// Domain contexts encountered
    pub domains: HashSet<DomainContext>,
    
    /// Has temporal ordering
    pub is_temporally_ordered: bool,
}

impl SemanticPath {
    pub fn new(concepts: Vec<ConceptId>) -> Self {
        Self {
            concepts,
            confidence: 1.0,
            type_distribution: HashMap::new(),
            domains: HashSet::new(),
            is_temporally_ordered: false,
        }
    }
    
    pub fn len(&self) -> usize {
        self.concepts.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }
}

/// Semantic-aware pathfinding engine
pub struct SemanticPathFinder {
    /// Maximum path depth
    max_depth: usize,
    
    /// Maximum paths to find
    max_paths: usize,
}

impl SemanticPathFinder {
    pub fn new(max_depth: usize, max_paths: usize) -> Self {
        Self {
            max_depth,
            max_paths,
        }
    }
    
    /// Find paths with semantic filtering
    pub fn find_paths_filtered(
        &self,
        snapshot: Arc<GraphSnapshot>,
        start: ConceptId,
        end: ConceptId,
        filter: &SemanticFilter,
    ) -> Vec<SemanticPath> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Initialize with start node
        queue.push_back((start, vec![start], 0));
        visited.insert(start);
        
        while let Some((current, path, depth)) = queue.pop_front() {
            // Check if we reached the destination
            if current == end {
                let semantic_path = self.analyze_path(&snapshot, &path);
                paths.push(semantic_path);
                
                if paths.len() >= self.max_paths {
                    break;
                }
                continue;
            }
            
            // Check depth limit
            if depth >= self.max_depth {
                continue;
            }
            
            // Get neighbors and filter by semantic constraints
            if let Some(node) = snapshot.get_concept(&current) {
                for &neighbor_id in &node.neighbors {
                    if visited.contains(&neighbor_id) {
                        continue;
                    }
                    
                    // Apply semantic filter
                    if let Some(neighbor_node) = snapshot.get_concept(&neighbor_id) {
                        if let Some(ref semantic) = neighbor_node.semantic {
                            let content = String::from_utf8_lossy(&neighbor_node.content);
                            if !filter.matches(semantic, &content, &neighbor_id) {
                                continue;
                            }
                        } else if filter.semantic_type.is_some() || filter.domain_context.is_some() {
                            // Filter requires semantic metadata but concept doesn't have it
                            continue;
                        }
                        
                        let mut new_path = path.clone();
                        new_path.push(neighbor_id);
                        queue.push_back((neighbor_id, new_path, depth + 1));
                        visited.insert(neighbor_id);
                    }
                }
            }
        }
        
        paths
    }
    
    /// Find temporal chain (concepts ordered by time)
    pub fn find_temporal_chain(
        &self,
        snapshot: Arc<GraphSnapshot>,
        domain: Option<DomainContext>,
        start_time: i64,
        end_time: i64,
    ) -> Vec<SemanticPath> {
        let mut temporal_concepts = Vec::new();
        
        // Collect all concepts with temporal bounds in the time range
        for concept in snapshot.all_concepts() {
            if let Some(ref semantic) = concept.semantic {
                // Domain filter
                if let Some(required_domain) = domain {
                    if semantic.domain_context != required_domain {
                        continue;
                    }
                }
                
                // Temporal filter
                if let Some(ref bounds) = semantic.temporal_bounds {
                    if let Some(start) = bounds.start {
                        if start >= start_time && start <= end_time {
                            temporal_concepts.push((concept.id, start, semantic.clone()));
                        }
                    }
                }
            }
        }
        
        // Sort by temporal order
        temporal_concepts.sort_by_key(|(_, timestamp, _)| *timestamp);
        
        // Build path
        if temporal_concepts.is_empty() {
            return Vec::new();
        }
        
        let concepts: Vec<ConceptId> = temporal_concepts.iter().map(|(id, _, _)| *id).collect();
        let mut path = SemanticPath::new(concepts);
        path.is_temporally_ordered = true;
        
        // Analyze semantic distribution
        for (_, _, semantic) in &temporal_concepts {
            *path.type_distribution.entry(semantic.semantic_type).or_insert(0) += 1;
            path.domains.insert(semantic.domain_context);
        }
        
        vec![path]
    }
    
    /// Find causal chain (cause -> effect relationships)
    pub fn find_causal_chain(
        &self,
        snapshot: Arc<GraphSnapshot>,
        start: ConceptId,
        causal_type: CausalType,
    ) -> Vec<SemanticPath> {
        let mut chain = vec![start];
        let mut current = start;
        let mut visited = HashSet::new();
        visited.insert(start);
        
        // Follow causal relationships
        for _ in 0..self.max_depth {
            let mut found_next = false;
            
            if let Some(node) = snapshot.get_concept(&current) {
                if let Some(ref semantic) = node.semantic {
                    // Check if this concept has causal relations of the specified type
                    let has_causal = semantic.causal_relations.iter()
                        .any(|r| r.relation_type == causal_type);
                    
                    if has_causal {
                        // Find the next concept in the causal chain
                        for &neighbor_id in &node.neighbors {
                            if visited.contains(&neighbor_id) {
                                continue;
                            }
                            
                            if let Some(neighbor_node) = snapshot.get_concept(&neighbor_id) {
                                if let Some(ref neighbor_semantic) = neighbor_node.semantic {
                                    if neighbor_semantic.semantic_type == SemanticType::Causal {
                                        chain.push(neighbor_id);
                                        visited.insert(neighbor_id);
                                        current = neighbor_id;
                                        found_next = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if !found_next {
                break;
            }
        }
        
        if chain.len() > 1 {
            let path = self.analyze_path(&snapshot, &chain);
            vec![path]
        } else {
            Vec::new()
        }
    }
    
    /// Find contradictions (conflicting rules in same domain)
    pub fn find_contradictions(
        &self,
        snapshot: Arc<GraphSnapshot>,
        domain: DomainContext,
    ) -> Vec<(ConceptId, ConceptId, String)> {
        let mut contradictions = Vec::new();
        let mut rules = Vec::new();
        
        // Collect all rules in the domain
        for concept in snapshot.all_concepts() {
            if let Some(ref semantic) = concept.semantic {
                if semantic.semantic_type == SemanticType::Rule 
                    && semantic.domain_context == domain {
                    rules.push((concept.id, semantic.clone()));
                }
            }
        }
        
        // Check for temporal overlap and conflicts
        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                let (id1, semantic1) = &rules[i];
                let (id2, semantic2) = &rules[j];
                
                if semantic1.conflicts_with(semantic2) {
                    contradictions.push((
                        *id1,
                        *id2,
                        format!(
                            "Rules conflict: both apply in {} domain with overlapping temporal bounds",
                            domain.as_str()
                        )
                    ));
                }
            }
        }
        
        contradictions
    }
    
    /// Analyze a path and compute semantic metrics
    fn analyze_path(&self, snapshot: &GraphSnapshot, path: &[ConceptId]) -> SemanticPath {
        let mut semantic_path = SemanticPath::new(path.to_vec());
        let mut confidence_sum = 0.0;
        let mut confidence_count = 0;
        
        for &concept_id in path {
            if let Some(node) = snapshot.get_concept(&concept_id) {
                if let Some(ref semantic) = node.semantic {
                    // Update confidence
                    confidence_sum += semantic.classification_confidence;
                    confidence_count += 1;
                    
                    // Update type distribution
                    *semantic_path.type_distribution
                        .entry(semantic.semantic_type)
                        .or_insert(0) += 1;
                    
                    // Update domains
                    semantic_path.domains.insert(semantic.domain_context);
                }
            }
        }
        
        // Calculate average confidence
        if confidence_count > 0 {
            semantic_path.confidence = confidence_sum / confidence_count as f32;
        }
        
        semantic_path
    }
}

impl Default for SemanticPathFinder {
    fn default() -> Self {
        Self::new(10, 100) // Default: depth 10, max 100 paths
    }
}

impl DomainContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Medical => "medical",
            Self::Legal => "legal",
            Self::Financial => "financial",
            Self::Technical => "technical",
            Self::Scientific => "scientific",
            Self::Business => "business",
            Self::General => "general",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_view::ConceptNode;
    
    #[test]
    fn test_semantic_pathfinding_with_filter() {
        // Create a simple graph
        let mut snapshot = GraphSnapshot::new(0);
        
        let id1 = ConceptId::from_bytes([1u8; 16]);
        let id2 = ConceptId::from_bytes([2u8; 16]);
        let id3 = ConceptId::from_bytes([3u8; 16]);
        
        let mut node1 = ConceptNode::with_semantic(
            id1,
            b"Medical rule 1".to_vec(),
            None,
            1.0,
            1.0,
            1000,
            SemanticMetadata {
                semantic_type: SemanticType::Rule,
                domain_context: DomainContext::Medical,
                temporal_bounds: None,
                causal_relations: Vec::new(),
                negation_scope: None,
                classification_confidence: 0.9,
            }
        );
        node1.neighbors.push(id2);
        
        let mut node2 = ConceptNode::with_semantic(
            id2,
            b"Medical entity".to_vec(),
            None,
            1.0,
            1.0,
            1000,
            SemanticMetadata {
                semantic_type: SemanticType::Entity,
                domain_context: DomainContext::Medical,
                temporal_bounds: None,
                causal_relations: Vec::new(),
                negation_scope: None,
                classification_confidence: 0.85,
            }
        );
        node2.neighbors.push(id3);
        
        let node3 = ConceptNode::with_semantic(
            id3,
            b"Medical rule 2".to_vec(),
            None,
            1.0,
            1.0,
            1000,
            SemanticMetadata {
                semantic_type: SemanticType::Rule,
                domain_context: DomainContext::Medical,
                temporal_bounds: None,
                causal_relations: Vec::new(),
                negation_scope: None,
                classification_confidence: 0.92,
            }
        );
        
        snapshot.concepts.insert(id1, node1);
        snapshot.concepts.insert(id2, node2);
        snapshot.concepts.insert(id3, node3);
        snapshot.update_stats();
        
        let pathfinder = SemanticPathFinder::new(10, 10);
        let filter = SemanticFilter::new()
            .with_domain(DomainContext::Medical);
        
        let paths = pathfinder.find_paths_filtered(
            Arc::new(snapshot),
            id1,
            id3,
            &filter
        );
        
        assert!(!paths.is_empty());
        assert_eq!(paths[0].len(), 3);
        assert!(paths[0].domains.contains(&DomainContext::Medical));
    }
}
