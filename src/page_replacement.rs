use serde::{Deserialize, Serialize};

/// Page replacement algorithm trait.
pub trait PageReplacement: Send + Sync {
    /// Access a page. Returns true if page fault occurred.
    fn access(&mut self, page: usize) -> bool;
    /// Get current frames in memory.
    fn frames(&self) -> Vec<usize>;
    /// Get total faults so far.
    fn faults(&self) -> usize;
    /// Name of the algorithm.
    fn name(&self) -> &'static str;
}

/// FIFO page replacement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FifoReplacement {
    pub capacity: usize,
    pub frames: Vec<usize>,
    pub faults: usize,
    pub queue: Vec<usize>, // insertion order
}

impl FifoReplacement {
    pub fn new(capacity: usize) -> Self {
        FifoReplacement {
            capacity,
            frames: Vec::new(),
            faults: 0,
            queue: Vec::new(),
        }
    }
}

impl PageReplacement for FifoReplacement {
    fn access(&mut self, page: usize) -> bool {
        if self.frames.contains(&page) {
            return false;
        }
        self.faults += 1;
        if self.frames.len() < self.capacity {
            self.frames.push(page);
            self.queue.push(page);
        } else {
            // Remove the oldest
            let oldest = self.queue.remove(0);
            if let Some(pos) = self.frames.iter().position(|&x| x == oldest) {
                self.frames[pos] = page;
            }
            self.queue.push(page);
        }
        true
    }

    fn frames(&self) -> Vec<usize> {
        self.frames.clone()
    }

    fn faults(&self) -> usize {
        self.faults
    }

    fn name(&self) -> &'static str {
        "FIFO"
    }
}

/// LRU page replacement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LruReplacement {
    pub capacity: usize,
    pub frames: Vec<usize>,
    pub faults: usize,
    pub use_order: Vec<usize>, // most recently used at end
}

impl LruReplacement {
    pub fn new(capacity: usize) -> Self {
        LruReplacement {
            capacity,
            frames: Vec::new(),
            faults: 0,
            use_order: Vec::new(),
        }
    }
}

impl PageReplacement for LruReplacement {
    fn access(&mut self, page: usize) -> bool {
        let fault = if self.frames.contains(&page) {
            self.use_order.retain(|&x| x != page);
            self.use_order.push(page);
            false
        } else {
            self.faults += 1;
            if self.frames.len() < self.capacity {
                self.frames.push(page);
            } else {
                // Evict least recently used
                if let Some(&lru) = self.use_order.first() {
                    if let Some(pos) = self.frames.iter().position(|&x| x == lru) {
                        self.frames[pos] = page;
                    }
                    self.use_order.remove(0);
                }
            }
            self.use_order.push(page);
            true
        };
        fault
    }

    fn frames(&self) -> Vec<usize> {
        self.frames.clone()
    }

    fn faults(&self) -> usize {
        self.faults
    }

    fn name(&self) -> &'static str {
        "LRU"
    }
}

/// Clock (Second Chance) page replacement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockReplacement {
    pub capacity: usize,
    pub frames: Vec<Option<usize>>,
    pub reference_bits: Vec<bool>,
    pub faults: usize,
    pub hand: usize,
}

impl ClockReplacement {
    pub fn new(capacity: usize) -> Self {
        ClockReplacement {
            capacity,
            frames: vec![None; capacity],
            reference_bits: vec![false; capacity],
            faults: 0,
            hand: 0,
        }
    }

    fn find_page(&self, page: usize) -> Option<usize> {
        self.frames.iter().position(|f| f == &Some(page))
    }

    fn find_free(&self) -> Option<usize> {
        self.frames.iter().position(|f| f.is_none())
    }
}

impl PageReplacement for ClockReplacement {
    fn access(&mut self, page: usize) -> bool {
        if let Some(idx) = self.find_page(page) {
            self.reference_bits[idx] = true;
            return false;
        }

        self.faults += 1;

        if let Some(idx) = self.find_free() {
            self.frames[idx] = Some(page);
            self.reference_bits[idx] = true;
            return true;
        }

        // Clock eviction
        loop {
            if !self.reference_bits[self.hand] {
                self.frames[self.hand] = Some(page);
                self.reference_bits[self.hand] = true;
                self.hand = (self.hand + 1) % self.capacity;
                return true;
            }
            self.reference_bits[self.hand] = false;
            self.hand = (self.hand + 1) % self.capacity;
        }
    }

    fn frames(&self) -> Vec<usize> {
        self.frames.iter().filter_map(|f| *f).collect()
    }

    fn faults(&self) -> usize {
        self.faults
    }

    fn name(&self) -> &'static str {
        "Clock"
    }
}

/// Optimal page replacement (needs future knowledge).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalReplacement {
    pub capacity: usize,
    pub frames: Vec<usize>,
    pub faults: usize,
    pub future: Vec<usize>, // remaining reference string
    pub position: usize,
}

impl OptimalReplacement {
    pub fn new(capacity: usize, references: Vec<usize>) -> Self {
        OptimalReplacement {
            capacity,
            frames: Vec::new(),
            faults: 0,
            future: references,
            position: 0,
        }
    }

    fn next_use(&self, page: usize) -> Option<usize> {
        for (i, &p) in self.future[self.position + 1..].iter().enumerate() {
            if p == page {
                return Some(i + self.position + 1);
            }
        }
        None
    }
}

impl PageReplacement for OptimalReplacement {
    fn access(&mut self, page: usize) -> bool {
        if self.frames.contains(&page) {
            self.position += 1;
            return false;
        }

        self.faults += 1;

        if self.frames.len() < self.capacity {
            self.frames.push(page);
        } else {
            // Find page used farthest in future (or never)
            let mut evict_idx = 0;
            let mut farthest: Option<usize> = None;

            for (i, &f) in self.frames.iter().enumerate() {
                match (self.next_use(f), farthest) {
                    (None, _) => {
                        // Never used again — evict this
                        evict_idx = i;
                        farthest = None;
                        break;
                    }
                    (Some(next), Some(far)) if next > far => {
                        farthest = Some(next);
                        evict_idx = i;
                    }
                    (Some(next), None) => {
                        farthest = Some(next);
                        evict_idx = i;
                    }
                    _ => {}
                }
            }

            self.frames[evict_idx] = page;
        }

        self.position += 1;
        true
    }

    fn frames(&self) -> Vec<usize> {
        self.frames.clone()
    }

    fn faults(&self) -> usize {
        self.faults
    }

    fn name(&self) -> &'static str {
        "Optimal"
    }
}

/// Simulate a page replacement algorithm on a reference string.
pub fn simulate_replacement(algo: &mut dyn PageReplacement, references: &[usize]) -> usize {
    for &page in references {
        algo.access(page);
    }
    algo.faults()
}
