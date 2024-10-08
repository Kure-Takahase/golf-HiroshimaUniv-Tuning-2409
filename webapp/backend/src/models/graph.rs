use sqlx::FromRow;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(FromRow, Clone, Debug)]
pub struct Node {
    pub id: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(FromRow, Clone, Debug)]
pub struct Edge {
    pub node_a_id: i32,
    pub node_b_id: i32,
    pub weight: i32,
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: HashMap<i32, Node>,
    pub edges: HashMap<i32, Vec<Edge>>,
    pub distances_cache: HashMap<i32, HashMap<i32, i32>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            distances_cache: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges
            .entry(edge.node_a_id)
            .or_default()
            .push(edge.clone());

        let reverse_edge = Edge {
            node_a_id: edge.node_b_id,
            node_b_id: edge.node_a_id,
            weight: edge.weight,
        };
        self.edges
            .entry(reverse_edge.node_a_id)
            .or_default()
            .push(reverse_edge);
    }

    pub fn shortest_path(&mut self, from_node_id: i32, to_node_id: i32) -> i32 {

        // Check if we already have the shortest path from the cache
        if let Some(distances) = self.distances_cache.get(&from_node_id) {
            if let Some(&distance) = distances.get(&to_node_id) {
                return distance;
            }
        }

        // Initialize distances and in_queue
        let mut distances = HashMap::new();
        let mut in_queue = HashMap::new();
        let mut queue = VecDeque::new();

        distances.insert(from_node_id, 0);
        queue.push_back(from_node_id);
        in_queue.insert(from_node_id, true);

        while let Some(current_node_id) = queue.pop_front() {
            in_queue.insert(current_node_id, false);

            if let Some(edges) = self.edges.get(&current_node_id) {
                for edge in edges {
                    let new_distance = distances
                        .get(&current_node_id)
                        .and_then(|d: &i32| d.checked_add(edge.weight))
                        .unwrap_or(i32::MAX);
                    let current_distance = distances.get(&edge.node_b_id).unwrap_or(&i32::MAX);

                    if new_distance < *current_distance {
                        distances.insert(edge.node_b_id, new_distance);

                        if !*in_queue.get(&edge.node_b_id).unwrap_or(&false) {
                            queue.push_back(edge.node_b_id);
                            in_queue.insert(edge.node_b_id, true);
                        }
                    }
                }
            }
        }

        // Cache the computed distances
        self.distances_cache.insert(from_node_id, distances.clone());

        // Return the distance to the target node
        distances.get(&to_node_id).cloned().unwrap_or(i32::MAX)

        /*

        let mut distances = HashMap::new();
        let mut in_queue = HashMap::new();
        let mut queue = VecDeque::new();

        distances.insert(from_node_id, 0);
        queue.push_back(from_node_id);
        in_queue.insert(from_node_id, true);

        while let Some(current_node_id) = queue.pop_front() {
            in_queue.insert(current_node_id, false);

            if let Some(edges) = self.edges.get(&current_node_id) {
                for edge in edges {
                    let new_distance = distances
                        .get(&current_node_id)
                        .and_then(|d: &i32| d.checked_add(edge.weight))
                        .unwrap_or(i32::MAX);
                    let current_distance = distances.get(&edge.node_b_id).unwrap_or(&i32::MAX);

                    if new_distance < *current_distance {
                        distances.insert(edge.node_b_id, new_distance);

                        if !*in_queue.get(&edge.node_b_id).unwrap_or(&false) {
                            queue.push_back(edge.node_b_id);
                            in_queue.insert(edge.node_b_id, true);
                        }
                    }
                }
            }
        }

        distances.get(&to_node_id).cloned().unwrap_or(i32::MAX)
        */
    
    }

    pub fn find_nearest_point(&self, from_node_id: i32, target_nodes: &[i32]) -> Vec<(i32, i32)> {
        let mut distances = HashMap::new();
        let mut in_queue = HashMap::new();
        let mut queue = VecDeque::new();

        distances.insert(from_node_id, 0);
        queue.push_back(from_node_id);
        in_queue.insert(from_node_id, true);

        while let Some(current_node_id) = queue.pop_front() {
            in_queue.insert(current_node_id, false);

            if let Some(edges) = self.edges.get(&current_node_id) {
                for edge in edges {
                    let new_distance = distances
                        .get(&current_node_id)
                        .and_then(|d: &i32| d.checked_add(edge.weight))
                        .unwrap_or(i32::MAX);
                    let current_distance = distances.get(&edge.node_b_id).unwrap_or(&i32::MAX);

                    if new_distance < *current_distance {
                        distances.insert(edge.node_b_id, new_distance);

                        if !*in_queue.get(&edge.node_b_id).unwrap_or(&false) {
                            queue.push_back(edge.node_b_id);
                            in_queue.insert(edge.node_b_id, true);
                        }
                    }
                }
            }
        }

        let mut result = Vec::new();

        for &target_node in target_nodes {
            if let Some(&distance) = distances.get(&target_node) {
                result.push((distance, target_node));
            }
        }

        result
    }

}
