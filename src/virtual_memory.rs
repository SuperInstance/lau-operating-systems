use serde::{Deserialize, Serialize};

/// Demand paging simulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandPager {
    pub num_frames: usize,
    pub frames: Vec<Option<usize>>,
    pub page_faults: usize,
    pub total_accesses: usize,
    pub frame_order: Vec<usize>, // FIFO order for eviction
}

impl DemandPager {
    pub fn new(num_frames: usize) -> Self {
        DemandPager {
            num_frames,
            frames: vec![None; num_frames],
            page_faults: 0,
            total_accesses: 0,
            frame_order: Vec::new(),
        }
    }

    /// Access a page. Returns true if page fault.
    pub fn access(&mut self, page: usize) -> bool {
        self.total_accesses += 1;

        // Check if already in frame
        if self.frames.iter().any(|f| f == &Some(page)) {
            return false;
        }

        self.page_faults += 1;

        // Find a free frame or evict
        if let Some(idx) = self.frames.iter().position(|f| f.is_none()) {
            self.frames[idx] = Some(page);
            self.frame_order.push(idx);
        } else {
            // FIFO eviction
            if let Some(&evict_idx) = self.frame_order.first() {
                self.frames[evict_idx] = Some(page);
                self.frame_order.remove(0);
                self.frame_order.push(evict_idx);
            }
        }

        true
    }

    /// Process a sequence of page accesses.
    pub fn process(&mut self, pages: &[usize]) -> (usize, f64) {
        for &page in pages {
            self.access(page);
        }
        let rate = if self.total_accesses > 0 {
            self.page_faults as f64 / self.total_accesses as f64
        } else {
            0.0
        };
        (self.page_faults, rate)
    }

    pub fn page_fault_rate(&self) -> f64 {
        if self.total_accesses == 0 { 0.0 }
        else { self.page_faults as f64 / self.total_accesses as f64 }
    }

    pub fn reset(&mut self) {
        self.frames = vec![None; self.num_frames];
        self.page_faults = 0;
        self.total_accesses = 0;
        self.frame_order.clear();
    }
}
