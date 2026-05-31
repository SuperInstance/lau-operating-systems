# lau-operating-systems

OS fundamentals implemented in Rust — scheduling, memory management, and concurrency primitives.

## Features

- **CPU Scheduling**: FCFS, SJF, Round Robin, Priority, MLFQ
- **Scheduling Analysis**: Turnaround time, waiting time, response time, throughput
- **Memory Management**: Page table simulation, TLB basics
- **Page Replacement**: FIFO, LRU, Clock, Optimal
- **Virtual Memory**: Demand paging simulation, page fault rates
- **Disk Scheduling**: FCFS, SSTF, SCAN, C-SCAN
- **Process Synchronization**: Mutex simulation, semaphore simulation, producer-consumer
- **Deadlock Detection**: Resource allocation graph, Banker's algorithm
- **Agent Scheduler**: Optimal scheduling for agent task queues with dependency support

## Usage

```rust
use lau_operating_systems::prelude::*;

// CPU scheduling
let processes = vec![
    Process { id: 0, arrival_time: 0, burst_time: 6, priority: 3 },
    Process { id: 1, arrival_time: 1, burst_time: 2, priority: 1 },
];
let result = fcfs(&processes);
println!("Avg waiting time: {}", result.avg_waiting);
```

## License

MIT
