use serde::{Deserialize, Serialize};

/// A process for scheduling purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: usize,
    pub arrival_time: u64,
    pub burst_time: u64,
    pub priority: u32,
}

/// Result of scheduling a set of processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResult {
    pub order: Vec<usize>,
    pub turnaround_times: Vec<(usize, u64)>,
    pub waiting_times: Vec<(usize, u64)>,
    pub response_times: Vec<(usize, u64)>,
    pub throughput: f64,
    pub avg_turnaround: f64,
    pub avg_waiting: f64,
    pub avg_response: f64,
}

/// FCFS (First-Come, First-Served) scheduler.
pub fn fcfs(processes: &[Process]) -> ScheduleResult {
    let mut procs: Vec<_> = processes.iter().enumerate().collect::<Vec<_>>();
    procs.sort_by_key(|(_, p)| (p.arrival_time, p.id));

    let mut order = Vec::new();
    let mut turnaround_times = Vec::new();
    let mut waiting_times = Vec::new();
    let mut response_times = Vec::new();

    let mut current_time: u64 = 0;
    for (_, p) in &procs {
        let start = current_time.max(p.arrival_time);
        let end = start + p.burst_time;
        let turnaround = end - p.arrival_time;
        let waiting = start - p.arrival_time;
        let response = start - p.arrival_time;

        order.push(p.id);
        turnaround_times.push((p.id, turnaround));
        waiting_times.push((p.id, waiting));
        response_times.push((p.id, response));

        current_time = end;
    }

    build_result(processes, order, turnaround_times, waiting_times, response_times, current_time)
}

/// SJF (Shortest Job First) non-preemptive scheduler.
pub fn sjf(processes: &[Process]) -> ScheduleResult {
    let n = processes.len();
    let mut remaining: Vec<(usize, &Process, bool)> = processes.iter().enumerate()
        .map(|(i, p)| (i, p, false))
        .collect();

    let mut order = Vec::new();
    let mut turnaround_times = Vec::new();
    let mut waiting_times = Vec::new();
    let mut response_times = Vec::new();
    let mut current_time: u64 = 0;
    let mut completed = 0;

    while completed < n {
        let mut best: Option<usize> = None;
        let mut best_burst = u64::MAX;

        for (idx, (_, p, done)) in remaining.iter().enumerate() {
            if !done && p.arrival_time <= current_time && p.burst_time < best_burst {
                best_burst = p.burst_time;
                best = Some(idx);
            }
        }

        let idx = match best {
            Some(idx) => idx,
            None => {
                // Advance time to next arrival
                let earliest = remaining.iter()
                    .filter(|(_, _, done)| !done)
                    .map(|(_, p, _)| p.arrival_time)
                    .min()
                    .unwrap();
                current_time = earliest;
                continue;
            }
        };

        let (_, p, _) = &remaining[idx];
        let pid = p.id;
        let start = current_time.max(p.arrival_time);
        let end = start + p.burst_time;

        order.push(pid);
        turnaround_times.push((pid, end - p.arrival_time));
        waiting_times.push((pid, start - p.arrival_time));
        response_times.push((pid, start - p.arrival_time));

        remaining[idx].2 = true;
        current_time = end;
        completed += 1;
    }

    build_result(processes, order, turnaround_times, waiting_times, response_times, current_time)
}

/// Round Robin scheduler.
pub fn round_robin(processes: &[Process], quantum: u64) -> ScheduleResult {
    let n = processes.len();
    let mut remaining_burst: Vec<u64> = processes.iter().map(|p| p.burst_time).collect();
    let mut first_run: Vec<Option<u64>> = vec![None; n];
    let mut completion_time: Vec<u64> = vec![0; n];

    let mut order = Vec::new();
    let mut current_time: u64 = 0;

    // Sort by arrival time
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by_key(|&i| (processes[i].arrival_time, processes[i].id));

    let mut queue: Vec<usize> = Vec::new();
    let mut next_to_enqueue = 0;
    let mut visited = vec![false; n];

    // Initial enqueue of processes arriving at time 0
    for &i in &indices {
        if processes[i].arrival_time <= current_time {
            queue.push(i);
            visited[i] = true;
            next_to_enqueue += 1;
        } else {
            break;
        }
    }

    while !queue.is_empty() || next_to_enqueue < n {
        if queue.is_empty() {
            // Jump to next arrival
            let next_arr = indices[next_to_enqueue];
            current_time = current_time.max(processes[next_arr].arrival_time);
            queue.push(next_arr);
            visited[next_arr] = true;
            next_to_enqueue += 1;
        }

        let idx = queue.remove(0);
        let p = &processes[idx];

        if first_run[idx].is_none() {
            first_run[idx] = Some(current_time);
        }

        let exec_time = remaining_burst[idx].min(quantum);
        remaining_burst[idx] -= exec_time;
        current_time += exec_time;
        order.push(p.id);

        // Enqueue newly arrived processes
        while next_to_enqueue < n {
            let ni = indices[next_to_enqueue];
            if processes[ni].arrival_time <= current_time {
                if !visited[ni] {
                    queue.push(ni);
                    visited[ni] = true;
                }
                next_to_enqueue += 1;
            } else {
                break;
            }
        }

        if remaining_burst[idx] > 0 {
            queue.push(idx);
        } else {
            completion_time[idx] = current_time;
        }
    }

    let mut turnaround_times = Vec::new();
    let mut waiting_times = Vec::new();
    let mut response_times = Vec::new();

    for i in 0..n {
        let tt = completion_time[i] - processes[i].arrival_time;
        let rt = first_run[i].unwrap() - processes[i].arrival_time;
        let wt = tt - processes[i].burst_time;
        turnaround_times.push((processes[i].id, tt));
        waiting_times.push((processes[i].id, wt));
        response_times.push((processes[i].id, rt));
    }

    build_result(processes, order, turnaround_times, waiting_times, response_times, current_time)
}

/// Priority scheduler (non-preemptive). Lower number = higher priority.
pub fn priority_schedule(processes: &[Process]) -> ScheduleResult {
    let n = processes.len();
    let mut remaining: Vec<(usize, &Process, bool)> = processes.iter().enumerate()
        .map(|(i, p)| (i, p, false))
        .collect();

    let mut order = Vec::new();
    let mut turnaround_times = Vec::new();
    let mut waiting_times = Vec::new();
    let mut response_times = Vec::new();
    let mut current_time: u64 = 0;
    let mut completed = 0;

    while completed < n {
        let mut best: Option<usize> = None;
        let mut best_priority = u32::MAX;

        for (idx, (_, p, done)) in remaining.iter().enumerate() {
            if !done && p.arrival_time <= current_time && p.priority < best_priority {
                best_priority = p.priority;
                best = Some(idx);
            }
        }

        let idx = match best {
            Some(idx) => idx,
            None => {
                let earliest = remaining.iter()
                    .filter(|(_, _, done)| !done)
                    .map(|(_, p, _)| p.arrival_time)
                    .min()
                    .unwrap();
                current_time = earliest;
                continue;
            }
        };

        let (_, p, _) = &remaining[idx];
        let pid = p.id;
        let start = current_time.max(p.arrival_time);
        let end = start + p.burst_time;

        order.push(pid);
        turnaround_times.push((pid, end - p.arrival_time));
        waiting_times.push((pid, start - p.arrival_time));
        response_times.push((pid, start - p.arrival_time));

        remaining[idx].2 = true;
        current_time = end;
        completed += 1;
    }

    build_result(processes, order, turnaround_times, waiting_times, response_times, current_time)
}

/// Multi-Level Feedback Queue (MLFQ) scheduler.
pub fn mlfq(processes: &[Process], num_queues: usize, quantums: &[u64]) -> ScheduleResult {
    let n = processes.len();
    let mut remaining_burst: Vec<u64> = processes.iter().map(|p| p.burst_time).collect();
    let mut first_run: Vec<Option<u64>> = vec![None; n];
    let mut completion_time: Vec<u64> = vec![0; n];
    let mut current_level: Vec<usize> = vec![0; n]; // each process starts at queue 0

    let mut queues: Vec<Vec<usize>> = (0..num_queues).map(|_| Vec::new()).collect();
    let mut order = Vec::new();
    let mut current_time: u64 = 0;
    let mut completed = 0;
    let mut visited = vec![false; n];

    // Sort by arrival time
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by_key(|&i| (processes[i].arrival_time, processes[i].id));

    let mut next_to_enqueue = 0;

    // Initial enqueue at time 0
    for &i in &indices {
        if processes[i].arrival_time <= current_time {
            queues[0].push(i);
            visited[i] = true;
            next_to_enqueue += 1;
        } else {
            break;
        }
    }

    while completed < n {
        // Find highest priority non-empty queue
        let mut active_queue = None;
        for q in 0..num_queues {
            if !queues[q].is_empty() {
                active_queue = Some(q);
                break;
            }
        }

        if active_queue.is_none() {
            if next_to_enqueue < n {
                let next_arr = indices[next_to_enqueue];
                current_time = current_time.max(processes[next_arr].arrival_time);
                queues[0].push(next_arr);
                visited[next_arr] = true;
                next_to_enqueue += 1;
                continue;
            }
            break;
        }

        let q = active_queue.unwrap();
        let idx = queues[q].remove(0);
        let p = &processes[idx];

        if first_run[idx].is_none() {
            first_run[idx] = Some(current_time);
        }

        let quantum = quantums[q.min(quantums.len() - 1)];
        let exec_time = remaining_burst[idx].min(quantum);
        remaining_burst[idx] -= exec_time;
        current_time += exec_time;
        order.push(p.id);

        // Enqueue newly arrived processes at queue 0
        while next_to_enqueue < n {
            let ni = indices[next_to_enqueue];
            if processes[ni].arrival_time <= current_time {
                if !visited[ni] {
                    queues[0].push(ni);
                    visited[ni] = true;
                }
                next_to_enqueue += 1;
            } else {
                break;
            }
        }

        if remaining_burst[idx] > 0 {
            // Demote to lower queue
            let new_level = (current_level[idx] + 1).min(num_queues - 1);
            current_level[idx] = new_level;
            queues[new_level].push(idx);
        } else {
            completion_time[idx] = current_time;
            completed += 1;
        }
    }

    let mut turnaround_times = Vec::new();
    let mut waiting_times = Vec::new();
    let mut response_times = Vec::new();

    for i in 0..n {
        let tt = completion_time[i] - processes[i].arrival_time;
        let rt = first_run[i].unwrap() - processes[i].arrival_time;
        let wt = tt - processes[i].burst_time;
        turnaround_times.push((processes[i].id, tt));
        waiting_times.push((processes[i].id, wt));
        response_times.push((processes[i].id, rt));
    }

    build_result(processes, order, turnaround_times, waiting_times, response_times, current_time)
}

fn build_result(
    processes: &[Process],
    order: Vec<usize>,
    turnaround_times: Vec<(usize, u64)>,
    waiting_times: Vec<(usize, u64)>,
    response_times: Vec<(usize, u64)>,
    total_time: u64,
) -> ScheduleResult {
    let n = processes.len() as f64;
    let throughput = if total_time > 0 { n / total_time as f64 } else { 0.0 };
    let avg_turnaround = turnaround_times.iter().map(|&(_, t)| t as f64).sum::<f64>() / n;
    let avg_waiting = waiting_times.iter().map(|&(_, t)| t as f64).sum::<f64>() / n;
    let avg_response = response_times.iter().map(|&(_, t)| t as f64).sum::<f64>() / n;

    ScheduleResult {
        order,
        turnaround_times,
        waiting_times,
        response_times,
        throughput,
        avg_turnaround,
        avg_waiting,
        avg_response,
    }
}
