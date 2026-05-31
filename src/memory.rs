use serde::{Deserialize, Serialize};

/// A page table entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTableEntry {
    pub frame_number: usize,
    pub valid: bool,
    pub dirty: bool,
    pub accessed: bool,
}

/// A simple page table simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTable {
    pub entries: Vec<PageTableEntry>,
    pub num_pages: usize,
    pub num_frames: usize,
    pub page_size: usize,
}

impl PageTable {
    pub fn new(num_pages: usize, num_frames: usize, page_size: usize) -> Self {
        PageTable {
            entries: (0..num_pages)
                .map(|_| PageTableEntry {
                    frame_number: 0,
                    valid: false,
                    dirty: false,
                    accessed: false,
                })
                .collect(),
            num_pages,
            num_frames,
            page_size,
        }
    }

    /// Translate a virtual address to a physical address. Returns None on fault.
    pub fn translate(&self, virtual_addr: usize) -> Option<usize> {
        let page = virtual_addr / self.page_size;
        let offset = virtual_addr % self.page_size;
        if page >= self.num_pages {
            return None;
        }
        let entry = &self.entries[page];
        if entry.valid {
            Some(entry.frame_number * self.page_size + offset)
        } else {
            None
        }
    }

    /// Map a page to a frame.
    pub fn map(&mut self, page: usize, frame: usize) {
        if page < self.num_pages {
            self.entries[page] = PageTableEntry {
                frame_number: frame,
                valid: true,
                dirty: false,
                accessed: false,
            };
        }
    }

    /// Unmap a page.
    pub fn unmap(&mut self, page: usize) {
        if page < self.num_pages {
            self.entries[page].valid = false;
        }
    }
}

/// TLB entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlbEntry {
    pub page: usize,
    pub frame: usize,
    pub valid: bool,
}

/// Simple TLB simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tlb {
    pub entries: Vec<TlbEntry>,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
    pub lru_order: Vec<usize>, // indices into entries, most recent last
}

impl Tlb {
    pub fn new(capacity: usize) -> Self {
        Tlb {
            entries: Vec::with_capacity(capacity),
            capacity,
            hits: 0,
            misses: 0,
            lru_order: Vec::new(),
        }
    }

    /// Look up a page in the TLB. Returns frame if hit.
    pub fn lookup(&mut self, page: usize) -> Option<usize> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.valid && entry.page == page {
                self.hits += 1;
                // Move to MRU
                self.lru_order.retain(|&x| x != i);
                self.lru_order.push(i);
                return Some(entry.frame);
            }
        }
        self.misses += 1;
        None
    }

    /// Insert a page→frame mapping into the TLB.
    pub fn insert(&mut self, page: usize, frame: usize) {
        // Check if already present
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if entry.page == page {
                entry.frame = frame;
                entry.valid = true;
                self.lru_order.retain(|&x| x != i);
                self.lru_order.push(i);
                return;
            }
        }

        if self.entries.len() < self.capacity {
            let idx = self.entries.len();
            self.entries.push(TlbEntry { page, frame, valid: true });
            self.lru_order.push(idx);
        } else {
            // Evict LRU
            if let Some(&lru_idx) = self.lru_order.first() {
                self.entries[lru_idx] = TlbEntry { page, frame, valid: true };
                self.lru_order.remove(0);
                self.lru_order.push(lru_idx);
            }
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
}
