# TEST-052: Unit Tests for CSR Graph Construction

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Key Invariants Checklist

## Description
Test CSR graph construction from RoadNetwork: verify node count, edge count, edge symmetry (undirected), and consistency with source RoadNetwork.

## Acceptance Criteria
- [ ] Test CSR graph node count matches RoadNetwork unique nodes
- [ ] Test edge count is correct
- [ ] Test graph is symmetric (if edge A->B exists, B->A exists)
- [ ] Test CSR matches RoadNetwork adjacency
- [ ] Test empty network produces empty graph
