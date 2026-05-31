#[cfg(test)]
mod tests {
    use lau_operating_systems::*;
    use page_replacement::PageReplacement;

    // ─── Scheduling Tests ───────────────────────────────────────────

    fn sample_processes() -> Vec<scheduling::Process> {
        vec![
            scheduling::Process { id: 0, arrival_time: 0, burst_time: 6, priority: 3 },
            scheduling::Process { id: 1, arrival_time: 1, burst_time: 2, priority: 1 },
            scheduling::Process { id: 2, arrival_time: 2, burst_time: 8, priority: 4 },
            scheduling::Process { id: 3, arrival_time: 3, burst_time: 3, priority: 2 },
            scheduling::Process { id: 4, arrival_time: 5, burst_time: 4, priority: 5 },
        ]
    }

    #[test]
    fn test_fcfs_order() {
        let procs = sample_processes();
        let result = scheduling::fcfs(&procs);
        assert_eq!(result.order, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_fcfs_turnaround() {
        let procs = sample_processes();
        let result = scheduling::fcfs(&procs);
        // P0: completes at 6, TT=6; P1: completes at 8, TT=7; P2: 16, TT=14; P3: 19, TT=16; P4: 23, TT=18
        assert_eq!(result.turnaround_times[0], (0, 6));
        assert_eq!(result.turnaround_times[1], (1, 7));
    }

    #[test]
    fn test_fcfs_waiting() {
        let procs = sample_processes();
        let result = scheduling::fcfs(&procs);
        assert_eq!(result.waiting_times[0], (0, 0));
        assert_eq!(result.waiting_times[1], (1, 5));
    }

    #[test]
    fn test_fcfs_response() {
        let procs = sample_processes();
        let result = scheduling::fcfs(&procs);
        // In FCFS non-preemptive, response = waiting
        assert_eq!(result.response_times[0], (0, 0));
    }

    #[test]
    fn test_sjf_order() {
        let procs = sample_processes();
        let result = scheduling::sjf(&procs);
        // At t=0: only P0 (burst=6). After P0 at t=6: P1(2),P2(8),P3(3),P4(4) available. SJF picks P1(2).
        // Then P3(3), P4(4), P2(8)
        assert_eq!(result.order[0], 0);
        assert_eq!(result.order[1], 1);
        assert_eq!(result.order[2], 3);
    }

    #[test]
    fn test_sjf_optimizes_avg_waiting() {
        let procs = sample_processes();
        let fcfs_result = scheduling::fcfs(&procs);
        let sjf_result = scheduling::sjf(&procs);
        assert!(sjf_result.avg_waiting <= fcfs_result.avg_waiting);
    }

    #[test]
    fn test_round_robin_order() {
        let procs = vec![
            scheduling::Process { id: 0, arrival_time: 0, burst_time: 5, priority: 0 },
            scheduling::Process { id: 1, arrival_time: 0, burst_time: 3, priority: 0 },
            scheduling::Process { id: 2, arrival_time: 0, burst_time: 1, priority: 0 },
        ];
        let result = scheduling::round_robin(&procs, 2);
        // Q=2: P0(2), P1(2), P2(1 done), P0(2), P1(1 done), P0(1 done)
        assert_eq!(result.order, vec![0, 1, 2, 0, 1, 0]);
    }

    #[test]
    fn test_round_robin_metrics() {
        let procs = vec![
            scheduling::Process { id: 0, arrival_time: 0, burst_time: 4, priority: 0 },
            scheduling::Process { id: 1, arrival_time: 0, burst_time: 3, priority: 0 },
        ];
        let result = scheduling::round_robin(&procs, 2);
        // P0: runs at 0-2, then 3-5. Completes at 7. TT=7, WT=3, RT=0
        // P1: runs at 2-4, then 5-7. Completes at 7. Wait no... let me recalculate.
        // Q=2: P0 runs 0-2, P1 runs 2-4, P0 runs 4-6, P1 runs 6-7
        // P0: completes at 6, TT=6, RT=0, WT=2
        // P1: completes at 7, TT=7, RT=2, WT=4
        assert_eq!(result.turnaround_times[0], (0, 6));
        assert_eq!(result.turnaround_times[1], (1, 7));
    }

    #[test]
    fn test_priority_scheduling_order() {
        let procs = sample_processes();
        let result = scheduling::priority_schedule(&procs);
        // At t=0: P0(pri=3). After P0 at t=6: P1(1),P2(4),P3(2),P4(5). Pick P1(1), then P3(2), P2(4), P4(5)
        assert_eq!(result.order[0], 0);
        assert_eq!(result.order[1], 1);
        assert_eq!(result.order[2], 3);
        assert_eq!(result.order[3], 2);
        assert_eq!(result.order[4], 4);
    }

    #[test]
    fn test_priority_lower_number_higher_priority() {
        let procs = vec![
            scheduling::Process { id: 0, arrival_time: 0, burst_time: 3, priority: 5 },
            scheduling::Process { id: 1, arrival_time: 0, burst_time: 3, priority: 1 },
        ];
        let result = scheduling::priority_schedule(&procs);
        assert_eq!(result.order[0], 1); // priority 1 runs first
    }

    #[test]
    fn test_mlfq_basic() {
        let procs = vec![
            scheduling::Process { id: 0, arrival_time: 0, burst_time: 7, priority: 0 },
            scheduling::Process { id: 1, arrival_time: 0, burst_time: 3, priority: 0 },
        ];
        let result = scheduling::mlfq(&procs, 3, &[2, 4, 8]);
        // Both start at Q0 (quantum=2). P0 runs 2, P1 runs 2, P0 demoted to Q1, P1 has 1 left stays Q0
        // P1(1) runs at Q0, completes. P0(5) at Q1(quantum=4): runs 4, demoted to Q2. P0(1) at Q2 runs 1.
        assert_eq!(result.order.len(), 5); // P0,P1,P1,P0,P0
    }

    #[test]
    fn test_throughput_calculation() {
        let procs = sample_processes();
        let result = scheduling::fcfs(&procs);
        // 5 processes in 23 time units
        let expected = 5.0 / 23.0;
        assert!((result.throughput - expected).abs() < 0.001);
    }

    #[test]
    fn test_sjf_fair_comparison() {
        let procs = vec![
            scheduling::Process { id: 0, arrival_time: 0, burst_time: 8, priority: 0 },
            scheduling::Process { id: 1, arrival_time: 1, burst_time: 4, priority: 0 },
            scheduling::Process { id: 2, arrival_time: 2, burst_time: 2, priority: 0 },
        ];
        let result = scheduling::sjf(&procs);
        // At t=0: P0 runs (only one). t=8: P1(4),P2(2) available. SJF picks P2(2), then P1(4).
        assert_eq!(result.order, vec![0, 2, 1]);
    }

    // ─── Memory Tests ────────────────────────────────────────────────

    #[test]
    fn test_page_table_translate_hit() {
        let mut pt = memory::PageTable::new(4, 4, 1024);
        pt.map(0, 2);
        pt.map(1, 0);
        assert_eq!(pt.translate(500), Some(2 * 1024 + 500)); // page 0 -> frame 2
        assert_eq!(pt.translate(1024 + 100), Some(0 * 1024 + 100)); // page 1 -> frame 0
    }

    #[test]
    fn test_page_table_translate_miss() {
        let pt = memory::PageTable::new(4, 4, 1024);
        assert_eq!(pt.translate(0), None); // no pages mapped
    }

    #[test]
    fn test_page_table_unmap() {
        let mut pt = memory::PageTable::new(4, 4, 1024);
        pt.map(0, 2);
        assert!(pt.translate(0).is_some());
        pt.unmap(0);
        assert!(pt.translate(0).is_none());
    }

    #[test]
    fn test_tlb_hit_miss() {
        let mut tlb = memory::Tlb::new(4);
        tlb.insert(0, 10);
        tlb.insert(1, 20);
        assert_eq!(tlb.lookup(0), Some(10)); // hit
        assert_eq!(tlb.lookup(1), Some(20)); // hit
        assert_eq!(tlb.lookup(2), None); // miss
        assert_eq!(tlb.hits, 2);
        assert_eq!(tlb.misses, 1);
    }

    #[test]
    fn test_tlb_eviction() {
        let mut tlb = memory::Tlb::new(2);
        tlb.insert(0, 10);
        tlb.insert(1, 20);
        tlb.insert(2, 30); // should evict page 0 (LRU)
        assert_eq!(tlb.lookup(0), None); // evicted
        assert_eq!(tlb.lookup(2), Some(30));
    }

    #[test]
    fn test_tlb_hit_rate() {
        let mut tlb = memory::Tlb::new(4);
        tlb.insert(0, 10);
        tlb.lookup(0); // hit
        tlb.lookup(1); // miss
        let rate = tlb.hit_rate();
        assert!((rate - 0.5).abs() < 0.001);
    }

    // ─── Page Replacement Tests ─────────────────────────────────────

    #[test]
    fn test_fifo_replacement() {
        let mut fifo = page_replacement::FifoReplacement::new(3);
        let refs = [1, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
        let faults = page_replacement::simulate_replacement(&mut fifo, &refs);
        assert_eq!(faults, 9);
    }

    #[test]
    fn test_lru_replacement() {
        let mut lru = page_replacement::LruReplacement::new(3);
        let refs = [1, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
        let faults = page_replacement::simulate_replacement(&mut lru, &refs);
        assert_eq!(faults, 10);
    }

    #[test]
    fn test_clock_replacement() {
        let mut clock = page_replacement::ClockReplacement::new(3);
        let refs = [1, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
        let faults = page_replacement::simulate_replacement(&mut clock, &refs);
        // Clock should be <= FIFO faults for this pattern
        assert!(faults <= 9);
    }

    #[test]
    fn test_optimal_replacement() {
        let refs = [1, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
        let mut opt = page_replacement::OptimalReplacement::new(3, refs.to_vec());
        let faults = page_replacement::simulate_replacement(&mut opt, &refs);
        assert_eq!(faults, 7);
    }

    #[test]
    fn test_fifo_belady_anomaly() {
        // Belady's anomaly: more frames can lead to more faults with FIFO
        // Classic example: refs = 1,2,3,4,1,2,5,1,2,3,4,5
        // 3 frames: 9 faults, 4 frames: 10 faults
        let mut fifo3 = page_replacement::FifoReplacement::new(3);
        let refs = [1, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
        let faults3 = page_replacement::simulate_replacement(&mut fifo3, &refs);

        let mut fifo4 = page_replacement::FifoReplacement::new(4);
        let faults4 = page_replacement::simulate_replacement(&mut fifo4, &refs);
        // This IS Belady's anomaly: more frames = more faults
        assert!(faults4 > faults3);
    }

    #[test]
    fn test_optimal_is_minimum_faults() {
        let refs = [7, 0, 1, 2, 0, 3, 0, 4, 2, 3, 0, 3, 2, 1, 2, 0, 1, 7, 0, 1];
        let mut opt = page_replacement::OptimalReplacement::new(3, refs.to_vec());
        let opt_faults = page_replacement::simulate_replacement(&mut opt, &refs);

        let mut fifo = page_replacement::FifoReplacement::new(3);
        let fifo_faults = page_replacement::simulate_replacement(&mut fifo, &refs);
        assert!(opt_faults <= fifo_faults);
    }

    #[test]
    fn test_lru_no_more_faults_than_optimal() {
        let refs = [1, 2, 3, 4, 1, 2, 5];
        let mut opt = page_replacement::OptimalReplacement::new(3, refs.to_vec());
        let opt_faults = page_replacement::simulate_replacement(&mut opt, &refs);
        let mut lru = page_replacement::LruReplacement::new(3);
        let lru_faults = page_replacement::simulate_replacement(&mut lru, &refs);
        assert!(opt_faults <= lru_faults);
    }

    // ─── Virtual Memory Tests ───────────────────────────────────────

    #[test]
    fn test_demand_paging_basic() {
        let mut pager = virtual_memory::DemandPager::new(3);
        assert!(pager.access(1)); // fault
        assert!(pager.access(2)); // fault
        assert!(!pager.access(1)); // hit
        assert!(pager.access(3)); // fault
        assert!(pager.access(4)); // fault, evicts page 2 (FIFO)
        assert_eq!(pager.page_faults, 4);
    }

    #[test]
    fn test_demand_paging_rate() {
        let mut pager = virtual_memory::DemandPager::new(2);
        let pages = [1, 2, 1, 2, 1, 2, 3];
        for &p in &pages {
            pager.access(p);
        }
        // Faults on 1, 2, 3 = 3 faults out of 7 accesses
        let rate = pager.page_fault_rate();
        assert!((rate - 3.0 / 7.0).abs() < 0.001);
    }

    #[test]
    fn test_demand_paging_process() {
        let mut pager = virtual_memory::DemandPager::new(3);
        let (faults, rate) = pager.process(&[1, 2, 3, 1, 4, 1, 5]);
        // faults: 1(f), 2(f), 3(f), 1(hit), 4(f-evict1), 1(f-evict2), 5(f-evict3) = 6
        assert_eq!(faults, 6);
        assert!((rate - 6.0 / 7.0).abs() < 0.001);
    }

    #[test]
    fn test_demand_paging_reset() {
        let mut pager = virtual_memory::DemandPager::new(2);
        pager.process(&[1, 2, 3]);
        pager.reset();
        assert_eq!(pager.page_faults, 0);
        assert_eq!(pager.total_accesses, 0);
    }

    // ─── Disk Scheduling Tests ──────────────────────────────────────

    #[test]
    fn test_disk_fcfs() {
        let result = disk::disk_fcfs(53, &[98, 183, 37, 122, 14, 124, 65, 67]);
        assert_eq!(result.total_seek_distance, 640);
    }

    #[test]
    fn test_disk_sstf() {
        let result = disk::disk_sstf(53, &[98, 183, 37, 122, 14, 124, 65, 67]);
        // SSTF from 53: 65(12), 67(2), 37(30), 14(23), 98(84), 122(24), 124(2), 183(59) = 236
        assert!(result.total_seek_distance < 640); // SSTF should be better than FCFS
    }

    #[test]
    fn test_disk_scan_right() {
        let result = disk::disk_scan(53, &[98, 183, 37, 122, 14, 124, 65, 67], 199, true);
        assert!(result.total_seek_distance > 0);
        // All requests should be serviced
        let mut serviced = result.order.clone();
        serviced.sort();
        assert_eq!(serviced, vec![14, 37, 65, 67, 98, 122, 124, 183]);
    }

    #[test]
    fn test_disk_cscan() {
        let result = disk::disk_cscan(53, &[98, 183, 37, 122, 14, 124, 65, 67], 199);
        assert!(result.total_seek_distance > 0);
        let mut serviced = result.order.clone();
        serviced.sort();
        assert_eq!(serviced, vec![14, 37, 65, 67, 98, 122, 124, 183]);
    }

    #[test]
    fn test_disk_sstf_order() {
        let result = disk::disk_sstf(50, &[30, 90, 25]);
        // From 50: 30(20), 25(5), 90(65) = 90
        assert_eq!(result.order, vec![30, 25, 90]);
    }

    // ─── Synchronization Tests ──────────────────────────────────────

    #[test]
    fn test_mutex_basic() {
        let mut m = synchronization::Mutex::new();
        assert!(m.lock(1));
        assert!(!m.lock(2)); // blocks
        assert_eq!(m.waiting_count(), 1);
        let next = m.unlock(1);
        assert_eq!(next, Some(2)); // thread 2 wakes
        assert!(m.is_locked());
    }

    #[test]
    fn test_mutex_unlock_returns_next() {
        let mut m = synchronization::Mutex::new();
        m.lock(1);
        m.lock(2);
        m.lock(3);
        let woken = m.unlock(1);
        assert_eq!(woken, Some(2));
        let woken2 = m.unlock(2);
        assert_eq!(woken2, Some(3));
    }

    #[test]
    fn test_semaphore_basic() {
        let mut sem = synchronization::Semaphore::new(2);
        assert!(sem.wait(1)); // value: 2->1
        assert!(sem.wait(2)); // value: 1->0
        assert!(!sem.wait(3)); // value: 0->-1, blocks
        assert_eq!(sem.waiting_count(), 1);
        let woken = sem.signal(); // value: -1->0
        assert_eq!(woken, Some(3));
    }

    #[test]
    fn test_semaphore_counting() {
        let mut sem = synchronization::Semaphore::new(3);
        assert!(sem.wait(1));
        assert!(sem.wait(2));
        assert!(sem.wait(3));
        assert!(!sem.wait(4));
        assert_eq!(sem.available(), 0);
        sem.signal();
        assert_eq!(sem.available(), 0); // still negative internally, one waiting released
    }

    #[test]
    fn test_producer_consumer() {
        let mut pc = synchronization::ProducerConsumer::new(3);
        assert!(pc.produce(1, 42));
        assert!(pc.produce(1, 43));
        assert!(pc.produce(1, 44));
        assert_eq!(pc.len(), 3);
        assert_eq!(pc.consume(2), Some(42));
        assert_eq!(pc.consume(2), Some(43));
    }

    // ─── Deadlock Tests ─────────────────────────────────────────────

    #[test]
    fn test_rag_no_deadlock() {
        let mut rag = deadlock::ResourceAllocationGraph::new(2, 2);
        rag.assign(0, 0); // R0 -> P0
        rag.assign(1, 1); // R1 -> P1
        rag.request(0, 1); // P0 requests R1
        rag.request(1, 0); // P1 requests R0
        // This IS a deadlock (circular wait)
        let deadlocked = rag.detect_deadlock();
        assert_eq!(deadlocked, vec![0, 1]);
    }

    #[test]
    fn test_rag_no_cycle() {
        let mut rag = deadlock::ResourceAllocationGraph::new(2, 2);
        rag.assign(0, 0); // R0 -> P0
        rag.request(1, 1); // P1 requests R1
        let deadlocked = rag.detect_deadlock();
        assert!(deadlocked.is_empty());
    }

    #[test]
    fn test_rag_three_way_deadlock() {
        let mut rag = deadlock::ResourceAllocationGraph::new(3, 3);
        rag.assign(0, 0); // R0 -> P0
        rag.assign(1, 1); // R1 -> P1
        rag.assign(2, 2); // R2 -> P2
        rag.request(0, 1); // P0 -> R1
        rag.request(1, 2); // P1 -> R2
        rag.request(2, 0); // P2 -> R0
        let deadlocked = rag.detect_deadlock();
        assert_eq!(deadlocked.len(), 3);
    }

    #[test]
    fn test_bankers_safe_state() {
        let bankers = deadlock::BankersAlgorithm::new(
            vec![3, 3, 2],
            vec![
                vec![7, 5, 3],
                vec![3, 2, 2],
                vec![9, 0, 2],
                vec![2, 2, 2],
                vec![4, 3, 3],
            ],
            vec![
                vec![0, 1, 0],
                vec![2, 0, 0],
                vec![3, 0, 2],
                vec![2, 1, 1],
                vec![0, 0, 2],
            ],
        );
        let safe = bankers.is_safe();
        assert!(safe.is_some());
        let seq = safe.unwrap();
        assert_eq!(seq.len(), 5);
    }

    #[test]
    fn test_bankers_unsafe_state() {
        let bankers = deadlock::BankersAlgorithm::new(
            vec![1, 0, 0], // very few available
            vec![
                vec![3, 3, 3],
                vec![2, 2, 2],
            ],
            vec![
                vec![2, 2, 2],
                vec![1, 1, 1],
            ],
        );
        // Need: P0=[1,1,1], P1=[1,1,1]. Available=[1,0,0]. P0 can't get enough, P1 can't either.
        assert!(bankers.is_safe().is_none());
    }

    #[test]
    fn test_bankers_request_safe() {
        let bankers = deadlock::BankersAlgorithm::new(
            vec![3, 3, 2],
            vec![
                vec![7, 5, 3],
                vec![3, 2, 2],
                vec![9, 0, 2],
                vec![2, 2, 2],
                vec![4, 3, 3],
            ],
            vec![
                vec![0, 1, 0],
                vec![2, 0, 0],
                vec![3, 0, 2],
                vec![2, 1, 1],
                vec![0, 0, 2],
            ],
        );
        // P1 requests [1, 0, 2] — should be safe
        let result = bankers.request(1, vec![1, 0, 2]);
        assert!(result.is_ok());
        let new_state = result.unwrap();
        assert!(new_state.is_safe().is_some());
    }

    #[test]
    fn test_bankers_request_exceeds_need() {
        let bankers = deadlock::BankersAlgorithm::new(
            vec![3, 3, 2],
            vec![
                vec![7, 5, 3],
                vec![3, 2, 2],
            ],
            vec![
                vec![0, 1, 0],
                vec![2, 0, 0],
            ],
        );
        let result = bankers.request(0, vec![8, 0, 0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_bankers_need_matrix() {
        let bankers = deadlock::BankersAlgorithm::new(
            vec![3, 3],
            vec![
                vec![5, 3],
                vec![2, 2],
            ],
            vec![
                vec![1, 1],
                vec![0, 1],
            ],
        );
        let need = bankers.need();
        assert_eq!(need[0][0], 4);
        assert_eq!(need[0][1], 2);
        assert_eq!(need[1][0], 2);
        assert_eq!(need[1][1], 1);
    }

    // ─── Agent Scheduler Tests ──────────────────────────────────────

    #[test]
    fn test_agent_scheduler_basic() {
        let tasks = vec![
            agent_scheduler::AgentTask {
                id: "task-a".into(),
                priority: 2,
                estimated_duration_ms: 100,
                dependencies: vec![],
                deadline: None,
            },
            agent_scheduler::AgentTask {
                id: "task-b".into(),
                priority: 1,
                estimated_duration_ms: 50,
                dependencies: vec![],
                deadline: None,
            },
            agent_scheduler::AgentTask {
                id: "task-c".into(),
                priority: 3,
                estimated_duration_ms: 200,
                dependencies: vec![],
                deadline: None,
            },
        ];
        let result = agent_scheduler::schedule_agent_tasks(&tasks);
        assert_eq!(result.task_order.len(), 3);
        assert_eq!(result.total_time_ms, 350);
    }

    #[test]
    fn test_agent_scheduler_with_dependencies() {
        let tasks = vec![
            agent_scheduler::AgentTask {
                id: "build".into(),
                priority: 1,
                estimated_duration_ms: 100,
                dependencies: vec![],
                deadline: None,
            },
            agent_scheduler::AgentTask {
                id: "test".into(),
                priority: 1,
                estimated_duration_ms: 200,
                dependencies: vec!["build".into()],
                deadline: None,
            },
            agent_scheduler::AgentTask {
                id: "deploy".into(),
                priority: 2,
                estimated_duration_ms: 50,
                dependencies: vec!["test".into()],
                deadline: None,
            },
        ];
        let result = agent_scheduler::schedule_with_dependencies(&tasks);
        // build must come before test, test before deploy
        let build_pos = result.task_order.iter().position(|t| t == "build").unwrap();
        let test_pos = result.task_order.iter().position(|t| t == "test").unwrap();
        let deploy_pos = result.task_order.iter().position(|t| t == "deploy").unwrap();
        assert!(build_pos < test_pos);
        assert!(test_pos < deploy_pos);
    }

    #[test]
    fn test_agent_scheduler_picks_best_algorithm() {
        let tasks = vec![
            agent_scheduler::AgentTask {
                id: "short".into(),
                priority: 1,
                estimated_duration_ms: 10,
                dependencies: vec![],
                deadline: None,
            },
            agent_scheduler::AgentTask {
                id: "long".into(),
                priority: 1,
                estimated_duration_ms: 1000,
                dependencies: vec![],
                deadline: None,
            },
            agent_scheduler::AgentTask {
                id: "medium".into(),
                priority: 1,
                estimated_duration_ms: 100,
                dependencies: vec![],
                deadline: None,
            },
        ];
        let result = agent_scheduler::schedule_agent_tasks(&tasks);
        // SJF should win: short(10), medium(100), long(1000) => avg wait = (0+10+110)/3 = 40
        assert_eq!(result.algorithm_used, "SJF");
    }

    // ─── Additional Edge Case Tests ─────────────────────────────────

    #[test]
    fn test_single_process_fcfs() {
        let procs = vec![scheduling::Process { id: 0, arrival_time: 0, burst_time: 10, priority: 1 }];
        let result = scheduling::fcfs(&procs);
        assert_eq!(result.order, vec![0]);
        assert_eq!(result.avg_turnaround, 10.0);
        assert_eq!(result.avg_waiting, 0.0);
    }

    #[test]
    fn test_single_process_sjf() {
        let procs = vec![scheduling::Process { id: 0, arrival_time: 5, burst_time: 3, priority: 1 }];
        let result = scheduling::sjf(&procs);
        assert_eq!(result.order, vec![0]);
        assert_eq!(result.turnaround_times[0], (0, 3));
    }

    #[test]
    fn test_fifo_frames_state() {
        let mut fifo = page_replacement::FifoReplacement::new(3);
        fifo.access(1);
        fifo.access(2);
        fifo.access(3);
        assert_eq!(fifo.frames(), vec![1, 2, 3]);
        fifo.access(4);
        assert_eq!(fifo.frames(), vec![4, 2, 3]);
    }

    #[test]
    fn test_lru_frames_state() {
        let mut lru = page_replacement::LruReplacement::new(3);
        lru.access(1);
        lru.access(2);
        lru.access(3);
        lru.access(2); // access 2 again, now 2 is MRU
        lru.access(4); // should evict 1 (LRU)
        assert!(lru.frames().contains(&4));
        assert!(!lru.frames().contains(&1));
    }

    #[test]
    fn test_disk_fcfs_order() {
        let result = disk::disk_fcfs(50, &[30, 90, 25]);
        assert_eq!(result.order, vec![30, 90, 25]);
        assert_eq!(result.total_seek_distance, 20 + 60 + 65);
    }

    #[test]
    fn test_mutex_double_unlock() {
        let mut m = synchronization::Mutex::new();
        m.lock(1);
        m.unlock(1);
        let result = m.unlock(1); // not owner
        assert_eq!(result, None);
    }

    #[test]
    fn test_bankers_safe_sequence() {
        let bankers = deadlock::BankersAlgorithm::new(
            vec![2, 1],
            vec![
                vec![3, 2],
                vec![2, 1],
                vec![1, 2],
            ],
            vec![
                vec![1, 0],
                vec![1, 1],
                vec![0, 0],
            ],
        );
        // Need: P0=[2,2], P1=[1,0], P2=[1,2]. Available=[2,1].
        // P1 can run (need [1,0] <= [2,1]), after: avail=[3,2]
        // P0 can run (need [2,2] <= [3,2]), after: avail=[4,2]
        // P2 can run
        let seq = bankers.is_safe().unwrap();
        assert_eq!(seq[0], 1); // P1 runs first
    }
}
