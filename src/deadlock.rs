use serde::{Deserialize, Serialize};

/// Resource Allocation Graph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RagEdge {
    Request { process: usize, resource: usize },
    Assignment { resource: usize, process: usize },
}

/// Resource Allocation Graph for deadlock detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocationGraph {
    pub num_processes: usize,
    pub num_resources: usize,
    pub edges: Vec<RagEdge>,
}

impl ResourceAllocationGraph {
    pub fn new(num_processes: usize, num_resources: usize) -> Self {
        ResourceAllocationGraph {
            num_processes,
            num_resources,
            edges: Vec::new(),
        }
    }

    pub fn request(&mut self, process: usize, resource: usize) {
        self.edges.push(RagEdge::Request { process, resource });
    }

    pub fn assign(&mut self, resource: usize, process: usize) {
        self.edges.push(RagEdge::Assignment { resource, process });
    }

    /// Detect deadlock using cycle detection in the RAG.
    pub fn detect_deadlock(&self) -> Vec<usize> {
        let total = self.num_processes + self.num_resources;
        let mut adj = vec![vec![]; total];
        for edge in &self.edges {
            match edge {
                RagEdge::Request { process, resource } => {
                    adj[*process].push(self.num_processes + resource);
                }
                RagEdge::Assignment { resource, process } => {
                    adj[self.num_processes + resource].push(*process);
                }
            }
        }

        let mut visited = vec![0u8; total];
        let mut in_cycle = vec![false; total];

        fn dfs(
            node: usize,
            adj: &[Vec<usize>],
            visited: &mut [u8],
            in_cycle: &mut [bool],
            path: &mut Vec<usize>,
        ) -> bool {
            visited[node] = 1;
            path.push(node);
            for &next in &adj[node] {
                if visited[next] == 1 {
                    let cycle_start = path.iter().position(|&n| n == next).unwrap();
                    for &n in &path[cycle_start..] {
                        in_cycle[n] = true;
                    }
                    return true;
                }
                if visited[next] == 0 {
                    if dfs(next, adj, visited, in_cycle, path) {
                        return true;
                    }
                }
            }
            visited[node] = 2;
            path.pop();
            false
        }

        for i in 0..self.num_processes {
            if visited[i] == 0 {
                let mut path = Vec::new();
                dfs(i, &adj, &mut visited, &mut in_cycle, &mut path);
            }
        }

        (0..self.num_processes).filter(|&i| in_cycle[i]).collect()
    }
}

/// Banker's Algorithm state using simple 2D vectors (nalgebra used internally for analysis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankersAlgorithm {
    pub num_processes: usize,
    pub num_resources: usize,
    pub available: Vec<usize>,
    pub max_demand: Vec<Vec<usize>>,
    pub allocation: Vec<Vec<usize>>,
}

impl BankersAlgorithm {
    pub fn new(
        available: Vec<usize>,
        max_demand: Vec<Vec<usize>>,
        allocation: Vec<Vec<usize>>,
    ) -> Self {
        let num_processes = max_demand.len();
        let num_resources = available.len();
        BankersAlgorithm {
            num_processes,
            num_resources,
            available,
            max_demand,
            allocation,
        }
    }

    /// Compute the need matrix (max - allocation).
    pub fn need(&self) -> Vec<Vec<usize>> {
        let mut need = Vec::with_capacity(self.num_processes);
        for i in 0..self.num_processes {
            let mut row = Vec::with_capacity(self.num_resources);
            for j in 0..self.num_resources {
                row.push(self.max_demand[i][j] - self.allocation[i][j]);
            }
            need.push(row);
        }
        need
    }

    /// Check if the system is in a safe state. Returns the safe sequence if safe.
    pub fn is_safe(&self) -> Option<Vec<usize>> {
        let need = self.need();
        let mut work = self.available.clone();
        let mut finish = vec![false; self.num_processes];
        let mut safe_seq = Vec::new();

        let mut found = true;
        while found {
            found = false;
            for i in 0..self.num_processes {
                if finish[i] {
                    continue;
                }
                let can_allocate = (0..self.num_resources)
                    .all(|j| need[i][j] <= work[j]);

                if can_allocate {
                    for j in 0..self.num_resources {
                        work[j] += self.allocation[i][j];
                    }
                    finish[i] = true;
                    safe_seq.push(i);
                    found = true;
                }
            }
        }

        if finish.iter().all(|&f| f) {
            Some(safe_seq)
        } else {
            None
        }
    }

    /// Request resources for a process.
    pub fn request(&self, process: usize, request: Vec<usize>) -> Result<BankersAlgorithm, String> {
        let need = self.need();

        for j in 0..self.num_resources {
            if request[j] > need[process][j] {
                return Err("Request exceeds maximum need".to_string());
            }
            if request[j] > self.available[j] {
                return Err("Resources not available".to_string());
            }
        }

        let mut new_available = self.available.clone();
        let mut new_allocation = self.allocation.clone();

        for j in 0..self.num_resources {
            new_available[j] -= request[j];
            new_allocation[process][j] += request[j];
        }

        let new_state = BankersAlgorithm {
            num_processes: self.num_processes,
            num_resources: self.num_resources,
            available: new_available,
            max_demand: self.max_demand.clone(),
            allocation: new_allocation,
        };

        if new_state.is_safe().is_some() {
            Ok(new_state)
        } else {
            Err("Request would lead to unsafe state".to_string())
        }
    }
}
