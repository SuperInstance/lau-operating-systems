# lau-operating-systems

OS fundamentals in pure Rust — CPU scheduling, memory management, page replacement, virtual memory, disk scheduling, concurrency primitives, deadlock detection, and an agent task scheduler.

Built as educational-grade implementations of every major OS subsystem you'd find in an undergraduate operating systems course, with full test coverage and practical agent-scheduling extensions.

---

## What This Does

| Module | What you get |
|---|---|
| **CPU Scheduling** | FCFS, SJF, Round Robin, Priority, MLFQ |
| **Memory Management** | Page tables, TLB simulation with LRU eviction |
| **Page Replacement** | FIFO, LRU, Clock (Second Chance), Optimal |
| **Virtual Memory** | Demand paging simulator with fault rate tracking |
| **Disk Scheduling** | FCFS, SSTF, SCAN, C-SCAN |
| **Synchronization** | Mutex, counting semaphore, producer-consumer |
| **Deadlock** | Resource Allocation Graph (cycle detection), Banker's Algorithm |
| **Agent Scheduler** | Auto-selects best algorithm, dependency-aware scheduling (topological sort + SJF) |

58 tests cover correctness, edge cases, Belady's anomaly, and classic OS textbook examples.

---

## Key Idea

Every OS concept is implemented as a self-contained, serializable simulation. You don't need a kernel — you construct the state, run the algorithm, and inspect the results. The `PageReplacement` trait lets you swap FIFO/LRU/Clock/Optimal interchangeably via `simulate_replacement`. The `ScheduleResult` struct gives you turnaround, waiting, and response times plus throughput.

The agent scheduler is the practical layer: it runs all four scheduling algorithms on your task set and picks whichever gives the lowest average waiting time. The dependency-aware variant uses Kahn's algorithm (topological sort) to respect task ordering constraints.

---

## Install

```toml
[dependencies]
lau-operating-systems = "0.1.0"
```

Or as a git dependency:

```toml
[dependencies]
lau-operating-systems = { git = "https://github.com/SuperInstance/lau-operating-systems" }
```

Requires **Rust 2021 edition**.

### Dependencies

| Crate | Why |
|---|---|
| `serde` | Serialize/deserialize all state types |
| `nalgebra` | Matrix utilities for analysis |

---

## Quick Start

### CPU Scheduling

```rust
use lau_operating_systems::scheduling::{Process, fcfs, sjf, round_robin, priority_schedule};

let processes = vec![
    Process { id: 0, arrival_time: 0, burst_time: 6, priority: 3 },
    Process { id: 1, arrival_time: 1, burst_time: 2, priority: 1 },
    Process { id: 2, arrival_time: 2, burst_time: 8, priority: 4 },
];

let result = fcfs(&processes);
println!("order: {:?}", result.order);         // [0, 1, 2]
println!("avg waiting: {:.2}", result.avg_waiting);

let rr = round_robin(&processes, 2);
println!("throughput: {:.4}", rr.throughput);
```

### Page Replacement

```rust
use lau_operating_systems::page_replacement::{FifoReplacement, LruReplacement, simulate_replacement};

let refs = [1, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];

let mut fifo = FifoReplacement::new(3);
let fifo_faults = simulate_replacement(&mut fifo, &refs);

let mut lru = LruReplacement::new(3);
let lru_faults = simulate_replacement(&mut lru, &refs);
```

### Page Table + TLB

```rust
use lau_operating_systems::memory::{PageTable, Tlb};

let mut pt = PageTable::new(16, 8, 4096);
pt.map(0, 3);  // page 0 -> frame 3
let phys = pt.translate(100);  // Some(3 * 4096 + 100)

let mut tlb = Tlb::new(4);
tlb.insert(0, 3);
tlb.lookup(0);  // Some(3) — hit
```

### Banker's Algorithm

```rust
use lau_operating_systems::deadlock::BankersAlgorithm;

let bankers = BankersAlgorithm::new(
    vec![3, 3, 2],                           // available
    vec![vec![7,5,3], vec![3,2,2], vec![9,0,2], vec![2,2,2], vec![4,3,3]],  // max
    vec![vec![0,1,0], vec![2,0,0], vec![3,0,2], vec![2,1,1], vec![0,0,2]],  // allocation
);

if let Some(safe_seq) = bankers.is_safe() {
    println!("safe sequence: {:?}", safe_seq);
}

let result = bankers.request(1, vec![1, 0, 2]);  // P1 requests (1,0,2)
```

### Disk Scheduling

```rust
use lau_operating_systems::disk::{disk_fcfs, disk_sstf, disk_scan, disk_cscan};

let requests = [98, 183, 37, 122, 14, 124, 65, 67];
let fcfs = disk_fcfs(53, &requests);
let sstf = disk_sstf(53, &requests);
let scan = disk_scan(53, &requests, 199, true);
let cscan = disk_cscan(53, &requests, 199);
```

### Agent Task Scheduling

```rust
use lau_operating_systems::agent_scheduler::{AgentTask, schedule_agent_tasks, schedule_with_dependencies};

let tasks = vec![
    AgentTask { id: "build".into(), priority: 1, estimated_duration_ms: 100,
                dependencies: vec![], deadline: None },
    AgentTask { id: "test".into(), priority: 1, estimated_duration_ms: 200,
                dependencies: vec!["build".into()], deadline: None },
    AgentTask { id: "deploy".into(), priority: 2, estimated_duration_ms: 50,
                dependencies: vec!["test".into()], deadline: None },
];

let result = schedule_with_dependencies(&tasks);
println!("execution order: {:?}", result.task_order);  // ["build", "test", "deploy"]
```

---

## API Reference

### `scheduling` — CPU Scheduling

| Function | Algorithm | Preemptive? |
|---|---|---|
| `fcfs` | First-Come, First-Served | No |
| `sjf` | Shortest Job First | No |
| `round_robin` | Round Robin with quantum | Yes |
| `priority_schedule` | Priority (lower = higher) | No |
| `mlfq` | Multi-Level Feedback Queue | Yes |

**`Process`**: `id`, `arrival_time`, `burst_time`, `priority`

**`ScheduleResult`**: `order`, `turnaround_times`, `waiting_times`, `response_times`, `throughput`, `avg_turnaround`, `avg_waiting`, `avg_response`

### `memory` — Page Tables and TLB

| Type | Key Methods |
|---|---|
| `PageTable` | `new(pages, frames, page_size)`, `translate(addr)`, `map(page, frame)`, `unmap(page)` |
| `Tlb` | `new(capacity)`, `lookup(page)`, `insert(page, frame)`, `hit_rate()` |

### `page_replacement` — Page Replacement Algorithms

All implement the `PageReplacement` trait: `access(page) → bool (fault?)`, `frames()`, `faults()`, `name()`.

| Struct | Algorithm |
|---|---|
| `FifoReplacement` | First-In, First-Out |
| `LruReplacement` | Least Recently Used |
| `ClockReplacement` | Second Chance / Clock |
| `OptimalReplacement` | Clairvoyant (needs future reference string) |

`simulate_replacement(algo, refs)` runs the full reference string and returns total faults.

### `virtual_memory` — Demand Paging

| Type | Key Methods |
|---|---|
| `DemandPager` | `new(frames)`, `access(page)`, `process(pages)`, `page_fault_rate()`, `reset()` |

### `disk` — Disk Scheduling

| Function | Algorithm |
|---|---|
| `disk_fcfs` | First-Come, First-Served |
| `disk_sstf` | Shortest Seek Time First |
| `disk_scan` | Elevator (direction: true=right, false=left) |
| `disk_cscan` | Circular SCAN |

**`DiskScheduleResult`**: `order`, `total_seek_distance`, `seek_sequence`

### `synchronization` — Concurrency Primitives

| Type | Key Methods |
|---|---|
| `Mutex` | `lock(thread)`, `unlock(thread)`, `is_locked()`, `waiting_count()` |
| `Semaphore` | `wait(thread)`, `signal()`, `available()`, `waiting_count()` |
| `ProducerConsumer` | `new(capacity)`, `produce(thread, item)`, `consume(thread)`, `len()` |

### `deadlock` — Deadlock Detection and Avoidance

| Type | Key Methods |
|---|---|
| `ResourceAllocationGraph` | `request(process, resource)`, `assign(resource, process)`, `detect_deadlock()` |
| `BankersAlgorithm` | `new(available, max, allocation)`, `need()`, `is_safe()`, `request(process, req)` |

### `agent_scheduler` — Agent Task Scheduling

| Function | Description |
|---|---|
| `schedule_agent_tasks` | Runs FCFS/SJF/RR/Priority, picks lowest avg wait |
| `schedule_with_dependencies` | Topological sort + priority-aware SJF |

**`AgentTask`**: `id`, `priority`, `estimated_duration_ms`, `dependencies`, `deadline`

---

## How It Works

### CPU Scheduling (`scheduling.rs`)

- **FCFS**: Sort processes by arrival time. Run each to completion in order. Simple but suffers from the "convoy effect" — short jobs wait behind long ones.
- **SJF**: At each scheduling point, pick the process with the shortest burst time among those that have arrived. Provably minimizes average waiting time for non-preemptive scheduling.
- **Round Robin**: Each process gets a time slice (quantum). If it doesn't finish, it goes to the back of the ready queue. Fair but context-switch overhead grows with small quantum values.
- **Priority**: Lower priority number = higher priority. Non-preemptive — once a process starts, it runs to completion. Suffers from starvation (low-priority processes may never run).
- **MLFQ**: Multiple queues with decreasing priority levels. New processes enter Q0. If a process uses its full quantum, it's demoted to the next lower queue. Higher queues have shorter quantums (interactive tasks finish fast), lower queues have longer quantums (CPU-bound tasks get bigger slices when they finally run).

### Memory Management (`memory.rs`)

- **Page Table**: Array of `PageTableEntry` structs (frame number, valid/dirty/accessed bits). Translates virtual addresses by splitting into page number and offset: `phys_addr = frame * page_size + offset`.
- **TLB**: Small associative cache that speeds up translation. On lookup hit, return the frame directly. On miss, fall through to the page table. Uses LRU eviction when full — the least-recently-used entry is replaced.

### Page Replacement (`page_replacement.rs`)

When a page fault occurs and all frames are full, one page must be evicted:

- **FIFO**: Evict the page that has been in memory the longest. Simple but suffers from Belady's anomaly (more frames can cause more faults). The classic test: reference string [1,2,3,4,1,2,5,1,2,3,4,5] with 3 frames gives 9 faults, with 4 frames gives 10.
- **LRU**: Evict the page that hasn't been used for the longest time. No Belady's anomaly — guaranteed to be stack algorithm. Approximated in hardware with reference bits.
- **Clock**: Approximation of LRU using a circular buffer of reference bits. When a page is accessed, its reference bit is set. The clock hand scans: if bit is 0, evict; if bit is 1, clear it and advance. Second-chance behavior.
- **Optimal**: Evict the page that won't be used for the longest time in the future. Requires oracle knowledge of the reference string — used as a theoretical lower bound.

### Virtual Memory (`virtual_memory.rs`)

The demand pager simulates loading pages on fault. It tracks page faults, total accesses, and computes the fault rate. Uses FIFO eviction internally. The `reset()` method clears all state for running comparisons with different configurations.

### Disk Scheduling (`disk.rs`)

All algorithms minimize total head movement differently:

- **FCFS**: Service requests in arrival order. No optimization — total distance can be large on random workloads.
- **SSTF**: Always service the nearest request. Greedy — minimizes immediate seek but can cause starvation of distant requests.
- **SCAN**: Move in one direction servicing all requests, then reverse (like an elevator). Guarantees bounded wait time.
- **C-SCAN**: Move in one direction only. After reaching the end, jump to the beginning and scan again. More uniform wait times than SCAN.

### Synchronization (`synchronization.rs`)

- **Mutex**: Binary semaphore with ownership. Only the locking thread can unlock. Blocked threads queue in FIFO order.
- **Semaphore**: Generalized counter. `wait()` decrements; if negative, the thread blocks. `signal()` increments; if a thread is waiting, wake one. Used for resource pooling.
- **Producer-Consumer**: Classic bounded buffer using three semaphores: `mutex` (binary, protects buffer), `items` (counting, tracks produced items), `spaces` (counting, tracks free slots).

### Deadlock (`deadlock.rs`)

- **Resource Allocation Graph**: Bipartite graph (processes, resources) with request and assignment edges. Deadlock exists iff there's a cycle. Uses DFS-based cycle detection.
- **Banker's Algorithm**: Avoidance strategy. Maintains `available`, `max_demand`, and `allocation` matrices. The `need` matrix is `max - allocation`. A state is safe if there exists a sequence where every process can eventually get its maximum resources (run to completion, return resources, enable the next). Resource requests are granted only if the resulting state is safe.

### Agent Scheduler (`agent_scheduler.rs`)

- **`schedule_agent_tasks`**: Converts agent tasks to OS processes, runs all four scheduling algorithms, and picks the result with the lowest average waiting time. Reports which algorithm won.
- **`schedule_with_dependencies`**: Builds a dependency graph, runs Kahn's algorithm for topological sort, and within each "ready set" picks by (priority, duration) — giving priority-aware SJF behavior while respecting ordering constraints.

---

## The Math

### Scheduling Metrics

For process *i* with arrival time *aᵢ*, burst time *bᵢ*, completion time *cᵢ*, and first run time *fᵢ*:

- **Turnaround time**: Tᵢ = cᵢ - aᵢ (total time in the system)
- **Waiting time**: Wᵢ = Tᵢ - bᵢ (time waiting, not executing)
- **Response time**: Rᵢ = fᵢ - aᵢ (time until first CPU allocation)
- **Throughput**: n / T_total (processes completed per unit time)

SJF minimizes average waiting time in the non-preemptive case. Round Robin with quantum q gives response time ≤ q·n for n processes.

### Belady's Anomaly

FIFO is not a stack algorithm: the set of pages in memory with k frames is not necessarily a subset of the pages with k+1 frames. This means adding frames can increase faults. LRU and Optimal are stack algorithms and don't exhibit this anomaly.

### Banker's Safety

The safety algorithm checks: can every process eventually run? Starting with `work = available`, it repeatedly finds a process whose need ≤ work, simulates its completion (work += allocation), and marks it finished. If all finish, the state is safe. Time complexity: O(m·n²) for n processes and m resources.

### Disk Scheduling Bounds

For n requests uniformly distributed on [0, N-1]:
- FCFS: O(n·N) expected total seek
- SSTF: O(n·√N) expected (nearest-neighbor on random points)
- SCAN/C-SCAN: O(n·N) worst case, but O(n) per scan sweep

---

## License

MIT
