#![deny(missing_docs)]
#![deny(rust_2021_compatibility)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![no_std]

//! RTIC implementation for GD32VF103XX devices

pub(crate) use gd32vf103xx_hal as hal;
pub use gd32vf103_rtic_macros::app;
pub use rtic_core::{prelude as mutex_prelude, Exclusive, Mutex};
pub use rtic_monotonic::{self, Monotonic};

#[doc(hidden)]
pub mod export;
#[doc(hidden)]
mod tq;

use crate::hal::eclic::EclicExt;
use riscv::interrupt::Nr;

/// Sets the given `interrupt` as pending
///
/// This is a convenience function around
/// [`Exti::pend`](../gd32vf103xx_hal/peripheral/struct.Exti.html#method.pend)
///
/// GPIO pin must be in range 0-15
pub fn pend<I: Nr>(interrupt: I) {
    hal::pac::ECLIC::pend(interrupt);
}

use core::cell::UnsafeCell;

/// Internal replacement for `static mut T`
///
/// Used to represent RTIC Resources
///
/// Soundness:
/// 1) Unsafe API for internal use only
/// 2) get_mut(&self) -> *mut T
///    returns a raw mutable pointer to the inner T
///    casting to &mut T is under control of RTIC
///    RTIC ensures &mut T to be unique under Rust aliasing rules.
///
///    Implementation uses the underlying UnsafeCell<T>
///    self.0.get() -> *mut T
///
/// 3) get(&self) -> *const T
///    returns a raw immutable (const) pointer to the inner T
///    casting to &T is under control of RTIC
///    RTIC ensures &T to be shared under Rust aliasing rules.
///
///    Implementation uses the underlying UnsafeCell<T>
///    self.0.get() -> *mut T, demoted to *const T
///    
#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    /// Create a RacyCell
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get `*mut T`
    ///
    /// # Safety
    ///
    /// See documentation notes for [`RacyCell`]
    #[inline(always)]
    pub unsafe fn get_mut(&self) -> *mut T {
        self.0.get()
    }

    /// Get `*const T`
    ///
    /// # Safety
    ///
    /// See documentation notes for [`RacyCell`]
    #[inline(always)]
    pub unsafe fn get(&self) -> *const T {
        self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}
