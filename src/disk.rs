use serde::{Deserialize, Serialize};

/// Disk scheduling result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskScheduleResult {
    pub order: Vec<usize>,
    pub total_seek_distance: usize,
    pub seek_sequence: Vec<usize>,
}

/// FCFS disk scheduling.
pub fn disk_fcfs(start: usize, requests: &[usize]) -> DiskScheduleResult {
    let mut total = 0usize;
    let mut current = start;
    let mut seeks = Vec::new();

    for &r in requests {
        let dist = (current as isize - r as isize).unsigned_abs();
        total += dist;
        seeks.push(dist);
        current = r;
    }

    DiskScheduleResult {
        order: requests.to_vec(),
        total_seek_distance: total,
        seek_sequence: seeks,
    }
}

/// SSTF (Shortest Seek Time First) disk scheduling.
pub fn disk_sstf(start: usize, requests: &[usize]) -> DiskScheduleResult {
    let mut remaining: Vec<usize> = requests.to_vec();
    let mut order = Vec::new();
    let mut seeks = Vec::new();
    let mut current = start;
    let mut total = 0usize;

    while !remaining.is_empty() {
        let (idx, _) = remaining.iter().enumerate()
            .min_by_key(|(_, &r)| (current as isize - r as isize).unsigned_abs())
            .unwrap();
        let next = remaining.remove(idx);
        let dist = (current as isize - next as isize).unsigned_abs();
        seeks.push(dist);
        total += dist;
        order.push(next);
        current = next;
    }

    DiskScheduleResult {
        order,
        total_seek_distance: total,
        seek_sequence: seeks,
    }
}

/// SCAN (elevator) disk scheduling.
pub fn disk_scan(start: usize, requests: &[usize], max_cylinder: usize, direction: bool) -> DiskScheduleResult {
    let mut left: Vec<usize> = requests.iter().filter(|&&r| r < start).copied().collect();
    let mut right: Vec<usize> = requests.iter().filter(|&&r| r >= start).copied().collect();
    left.sort();
    right.sort();

    let mut order = Vec::new();
    let mut seeks = Vec::new();
    let mut current = start;
    let mut total = 0usize;

    if direction {
        // Moving right (towards higher cylinders)
        for &r in &right {
            let dist = (current as isize - r as isize).unsigned_abs();
            total += dist;
            seeks.push(dist);
            order.push(r);
            current = r;
        }
        // Go to end if there are left requests
        if !left.is_empty() {
            let end = max_cylinder;
            let dist = (current as isize - end as isize).unsigned_abs();
            total += dist;
            seeks.push(dist);
            current = end;
        }
        for &r in left.iter().rev() {
            let dist = (current as isize - r as isize).unsigned_abs();
            total += dist;
            seeks.push(dist);
            order.push(r);
            current = r;
        }
    } else {
        // Moving left
        for &r in left.iter().rev() {
            let dist = (current as isize - r as isize).unsigned_abs();
            total += dist;
            seeks.push(dist);
            order.push(r);
            current = r;
        }
        if !right.is_empty() {
            let dist = current; // go to 0
            total += dist;
            seeks.push(dist);
            current = 0;
        }
        for &r in &right {
            let dist = (current as isize - r as isize).unsigned_abs();
            total += dist;
            seeks.push(dist);
            order.push(r);
            current = r;
        }
    }

    DiskScheduleResult {
        order,
        total_seek_distance: total,
        seek_sequence: seeks,
    }
}

/// C-SCAN disk scheduling.
pub fn disk_cscan(start: usize, requests: &[usize], max_cylinder: usize) -> DiskScheduleResult {
    let mut right: Vec<usize> = requests.iter().filter(|&&r| r >= start).copied().collect();
    let mut left: Vec<usize> = requests.iter().filter(|&&r| r < start).copied().collect();
    right.sort();
    left.sort();

    let mut order = Vec::new();
    let mut seeks = Vec::new();
    let mut current = start;
    let mut total = 0usize;

    // Go right to end
    for &r in &right {
        let dist = (current as isize - r as isize).unsigned_abs();
        total += dist;
        seeks.push(dist);
        order.push(r);
        current = r;
    }

    // Jump to 0 and service left
    if !left.is_empty() {
        let dist = (current as isize - max_cylinder as isize).unsigned_abs() + max_cylinder;
        total += dist;
        seeks.push(dist);
        current = 0;

        for &r in &left {
            let dist = (current as isize - r as isize).unsigned_abs();
            total += dist;
            seeks.push(dist);
            order.push(r);
            current = r;
        }
    }

    DiskScheduleResult {
        order,
        total_seek_distance: total,
        seek_sequence: seeks,
    }
}
