pub use crate::tq::{NotReady, TimerQueue};
pub use heapless::sorted_linked_list::SortedLinkedList;
pub use heapless::spsc::Queue;
pub use heapless::BinaryHeap;
pub use rtic_monotonic as monotonic;

pub use core::sync::atomic::{AtomicBool, Ordering};
pub use crate::hal::eclic::{EclicExt, Level, Priority};
pub use crate::hal::pac::ECLIC;
pub use riscv::interrupt::{self, CriticalSection, Nr};

pub use crate::hal::pac::Peripherals;

#[cfg(riscv)]
pub fn nop() { riscv::asm::nop() }
#[cfg(not(riscv))]
pub fn nop() { }

#[cfg(riscv)]
pub fn wfi() { riscv::asm::wfi() }
#[cfg(not(riscv))]
pub fn wfi() { }


pub type SCFQ<const N: usize> = Queue<u8, N>;
pub type SCRQ<T, const N: usize> = Queue<(T, u8), N>;

#[inline(always)]
pub fn run<F, R>(f: F) -> R
where
    F: FnOnce(&'_ CriticalSection) -> R,
{
    f(unsafe { &CriticalSection::new() })
}

pub struct Barrier {
    inner: AtomicBool,
}

impl Barrier {
    pub const fn new() -> Self {
        Barrier {
            inner: AtomicBool::new(false),
        }
    }

    pub fn release(&self) {
        self.inner.store(true, Ordering::Release)
    }

    pub fn wait(&self) {
        while !self.inner.load(Ordering::Acquire) {}
    }
}

#[inline(always)]
pub fn assert_send<T>()
where
    T: Send,
{
}

#[inline(always)]
pub fn assert_sync<T>()
where
    T: Sync,
{
}

#[inline(always)]
pub fn assert_monotonic<T>()
where
    T: monotonic::Monotonic,
{
}

/// Lock the resource proxy by running the closure with interrupt::free
#[inline(always)]
pub unsafe fn lock<I: Nr + Copy, T, R>(
    int: I,
    priority: Priority,
    level: Level,
    f: impl FnOnce(&'_ CriticalSection) -> R
) -> R {
    let (cur_lvl, cur_pio) = (ECLIC::get_level(int), ECLIC::get_priority(int));
    ECLIC::set_level(int, level);
    ECLIC::set_priority(int, priority);
    let r = interrupt::free(|c| f(c));
    ECLIC::set_level(int, cur_lvl);
    ECLIC::set_priority(int, cur_pio);
    r
}
