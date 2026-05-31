use serde::{Deserialize, Serialize};
use crate::scheduling::{Process, ScheduleResult, fcfs, sjf, round_robin, priority_schedule};

/// An agent task for scheduling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub priority: u32,
    pub estimated_duration_ms: u64,
    pub dependencies: Vec<String>,
    pub deadline: Option<u64>, // ms from epoch
}

/// Result of agent scheduling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentScheduleResult {
    pub task_order: Vec<String>,
    pub total_time_ms: u64,
    pub estimated_completion: Vec<(String, u64)>,
    pub algorithm_used: String,
}

/// Schedule agent tasks using the best algorithm.
pub fn schedule_agent_tasks(tasks: &[AgentTask]) -> AgentScheduleResult {
    let processes: Vec<Process> = tasks.iter().enumerate().map(|(i, t)| Process {
        id: i,
        arrival_time: 0,
        burst_time: t.estimated_duration_ms,
        priority: t.priority,
    }).collect();

    // Try multiple algorithms and pick the one with lowest avg waiting time
    let results: Vec<(String, ScheduleResult)> = vec![
        ("FCFS".to_string(), fcfs(&processes)),
        ("SJF".to_string(), sjf(&processes)),
        ("RoundRobin".to_string(), round_robin(&processes, 100)),
        ("Priority".to_string(), priority_schedule(&processes)),
    ];

    let (best_name, best) = results.into_iter()
        .min_by(|(_, a), (_, b)| a.avg_waiting.partial_cmp(&b.avg_waiting).unwrap())
        .unwrap();

    let task_order: Vec<String> = best.order.iter()
        .map(|&id| tasks[id].id.clone())
        .collect();

    let mut completion = Vec::new();
    let mut time = 0u64;
    for pid in &best.order {
        let task = &tasks[*pid];
        time += task.estimated_duration_ms;
        completion.push((task.id.clone(), time));
    }

    AgentScheduleResult {
        task_order,
        total_time_ms: time,
        estimated_completion: completion,
        algorithm_used: best_name,
    }
}

/// Schedule tasks respecting dependencies (topological sort + SJF).
pub fn schedule_with_dependencies(tasks: &[AgentTask]) -> AgentScheduleResult {
    let task_ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    let n = tasks.len();

    // Topological sort (Kahn's algorithm)
    let mut in_degree = vec![0usize; n];
    let mut adj = vec![vec![]; n];

    for (i, task) in tasks.iter().enumerate() {
        for dep in &task.dependencies {
            if let Some(j) = task_ids.iter().position(|id| id == dep) {
                adj[j].push(i);
                in_degree[i] += 1;
            }
        }
    }

    // Use a priority queue (min-heap by priority, then burst time)
    let mut ready: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    ready.sort_by_key(|&i| (tasks[i].priority, tasks[i].estimated_duration_ms));

    let mut order = Vec::new();
    while !ready.is_empty() {
        ready.sort_by_key(|&i| (tasks[i].priority, tasks[i].estimated_duration_ms));
        let idx = ready.remove(0);
        order.push(idx);

        for &next in &adj[idx] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                ready.push(next);
            }
        }
    }

    let mut time = 0u64;
    let mut completion = Vec::new();
    let task_order: Vec<String> = order.iter().map(|&i| {
        time += tasks[i].estimated_duration_ms;
        completion.push((tasks[i].id.clone(), time));
        tasks[i].id.clone()
    }).collect();

    AgentScheduleResult {
        task_order,
        total_time_ms: time,
        estimated_completion: completion,
        algorithm_used: "DependencyAware-SJF".to_string(),
    }
}
