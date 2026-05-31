use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Mutex simulation (binary semaphore).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mutex {
    pub locked: bool,
    pub owner: Option<usize>, // process/thread id
    pub wait_queue: VecDeque<usize>,
}

impl Mutex {
    pub fn new() -> Self {
        Mutex {
            locked: false,
            owner: None,
            wait_queue: VecDeque::new(),
        }
    }

    /// Try to lock. Returns true if acquired immediately.
    pub fn lock(&mut self, thread_id: usize) -> bool {
        if self.locked {
            self.wait_queue.push_back(thread_id);
            false
        } else {
            self.locked = true;
            self.owner = Some(thread_id);
            true
        }
    }

    /// Unlock. Returns the next thread to wake (if any).
    pub fn unlock(&mut self, thread_id: usize) -> Option<usize> {
        if self.owner == Some(thread_id) {
            if let Some(next) = self.wait_queue.pop_front() {
                self.owner = Some(next);
                Some(next)
            } else {
                self.locked = false;
                self.owner = None;
                None
            }
        } else {
            None
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn waiting_count(&self) -> usize {
        self.wait_queue.len()
    }
}

/// Counting semaphore simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semaphore {
    pub value: isize,
    pub wait_queue: VecDeque<usize>,
}

impl Semaphore {
    pub fn new(initial: isize) -> Self {
        Semaphore {
            value: initial,
            wait_queue: VecDeque::new(),
        }
    }

    /// Wait (P operation). Returns true if no blocking needed.
    pub fn wait(&mut self, thread_id: usize) -> bool {
        self.value -= 1;
        if self.value < 0 {
            self.wait_queue.push_back(thread_id);
            false
        } else {
            true
        }
    }

    /// Signal (V operation). Returns the thread to wake (if any).
    pub fn signal(&mut self) -> Option<usize> {
        self.value += 1;
        if self.value <= 0 {
            self.wait_queue.pop_front()
        } else {
            None
        }
    }

    pub fn available(&self) -> isize {
        self.value.max(0)
    }

    pub fn waiting_count(&self) -> usize {
        self.wait_queue.len()
    }
}

/// Producer-Consumer simulation using semaphores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerConsumer {
    pub buffer: VecDeque<i64>,
    pub capacity: usize,
    pub mutex: Semaphore,
    pub items: Semaphore,
    pub spaces: Semaphore,
}

impl ProducerConsumer {
    pub fn new(capacity: usize) -> Self {
        ProducerConsumer {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            mutex: Semaphore::new(1),
            items: Semaphore::new(0),
            spaces: Semaphore::new(capacity as isize),
        }
    }

    /// Produce an item. Returns true if successful (non-blocking).
    pub fn produce(&mut self, thread_id: usize, item: i64) -> bool {
        if self.spaces.wait(thread_id) {
            if self.mutex.wait(thread_id) {
                self.buffer.push_back(item);
                self.mutex.signal();
                self.items.signal();
                return true;
            }
        }
        false
    }

    /// Consume an item. Returns Some(item) if successful.
    pub fn consume(&mut self, thread_id: usize) -> Option<i64> {
        if self.items.wait(thread_id) {
            if self.mutex.wait(thread_id) {
                let item = self.buffer.pop_front();
                self.mutex.signal();
                self.spaces.signal();
                return item;
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
