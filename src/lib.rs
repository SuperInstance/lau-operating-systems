pub mod scheduling;
pub mod memory;
pub mod page_replacement;
pub mod virtual_memory;
pub mod disk;
pub mod synchronization;
pub mod deadlock;
pub mod agent_scheduler;

pub mod prelude {
    pub use crate::scheduling::*;
    pub use crate::memory::*;
    pub use crate::page_replacement::*;
    pub use crate::virtual_memory::*;
    pub use crate::disk::*;
    pub use crate::synchronization::*;
    pub use crate::deadlock::*;
    pub use crate::agent_scheduler::*;
}
