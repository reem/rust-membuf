#![feature(core, nonzero, alloc, oom, heap_api)]
#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
#![allow(raw_pointer_derive)]

//! # membuf
//!
//! A safe-ish wrapper for allocating and reallocating heap buffers.
//!

extern crate core;

pub use unique::UniqueBuf;

use core::nonzero::NonZero;
use std::ops::Deref;
use std::mem;

pub mod alloc;
mod unique;

/// A safe wrapper around a heap allocated buffer of Ts, tracking capacity only.
///
/// MemBuf makes no promises about the actual contents of this memory, that's up
/// to the user of the structure and can be manipulated using the standard pointer
/// utilities, accessible through the impl of `Deref<Target=*mut T>` for `MemBuf<T>`.
///
/// You can think of `MemBuf<T>` as an approximation for `Box<[T]>` where the elements
/// are not guaranteed to be valid/initialized. It is meant to be used as a building
/// block for other collections, so they do not have to concern themselves with the
/// minutiae of allocating, reallocating, and deallocating memory.
///
/// However, note that `MemBuf<T>` does not have a destructor, and implements `Copy`,
/// as a result, it does not implement `Send` or `Sync`, and it is the responsibility
/// of the user to call `deallocate` if they wish to free memory.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MemBuf<T> {
    buffer: NonZero<*mut T>,
    cap: usize
}

impl<T> Clone for MemBuf<T> { fn clone(&self) -> MemBuf<T> { *self } }
impl<T> Copy for MemBuf<T> {}

impl<T> MemBuf<T> {
    /// Create a new, empty MemBuf.
    ///
    /// ```
    /// # use membuf::MemBuf;
    ///
    /// let buffer: MemBuf<usize> = MemBuf::new();
    /// assert_eq!(buffer.capacity(), 0);
    /// ```
    pub fn new() -> MemBuf<T> {
        MemBuf {
            buffer: alloc::empty(),
            cap: 0
        }
    }

    /// Create a new buffer with space for cap Ts.
    ///
    /// Unlike `std::rt::heap::allocate`, cap == 0 is allowed.
    ///
    /// ```
    /// # use membuf::MemBuf;
    ///
    /// let buffer: MemBuf<usize> = MemBuf::allocate(128);
    /// assert_eq!(buffer.capacity(), 128);
    /// ```
    pub fn allocate(cap: usize) -> MemBuf<T> {
        if cap == 0 { return MemBuf::new() }

        MemBuf {
            buffer: unsafe { alloc::allocate(NonZero::new(cap)) },
            cap: cap
        }
    }

    /// Reallocate this buffer to fit a new number of Ts.
    ///
    /// Unlike `std::rt::heap::reallocate`, cap == 0 is allowed.
    ///
    /// ## Safety
    ///
    /// `reallocate` will invalidate the buffer in all other `MemBuf`s which
    /// share the same underlying buffer as this one. As a result, it is possible
    /// to cause a double-free by cloning or copying a `MemBuf` and calling
    /// `reallocate` from both handles.
    ///
    /// `UniqueBuf` has a safe `reallocate` implementation, since it cannot be
    /// copied or cloned into multiple handles.
    ///
    /// ```
    /// # use membuf::MemBuf;
    ///
    /// let mut buffer: MemBuf<usize> = MemBuf::allocate(128);
    /// assert_eq!(buffer.capacity(), 128);
    ///
    /// unsafe { buffer.reallocate(1024); }
    /// assert_eq!(buffer.capacity(), 1024);
    /// ```
    pub unsafe fn reallocate(&mut self, cap: usize) {
        if self.cap == 0 || cap == 0 {
            // Safe to drop the old buffer because either it never
            // allocated or we're getting rid of the allocation.
            *self = MemBuf::allocate(cap)
        } else {
            // We need to set the capacity to 0 because if the capacity
            // overflows unwinding is triggered, which if we don't
            // change the capacity would try to free empty().
            let old_cap = mem::replace(&mut self.cap, 0);
            let buffer = mem::replace(&mut self.buffer, alloc::empty());

            self.buffer = alloc::reallocate(buffer,
                                            NonZero::new(old_cap),
                                            NonZero::new(cap));
            self.cap = cap;
        }
    }

    /// Get the current capacity of the MemBuf.
    ///
    /// ```
    /// # use membuf::MemBuf;
    ///
    /// let buffer: MemBuf<usize> = MemBuf::allocate(128);
    /// assert_eq!(buffer.capacity(), 128);
    /// ```
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Deallocate the memory contained within the buffer.
    ///
    /// The MemBuf will *only* deallocate the contained memory. It will
    /// *not* run any destructors on data in that memory.
    ///
    /// ## Safety
    ///
    /// `deallocate` will invalidate the buffer in all other `MemBuf`s which
    /// share the same underlying buffer as this one. As a result, it is possible
    /// to cause a double-free by cloning or copying a `MemBuf` and calling
    /// `deallocate` from both handles.
    ///
    /// `UniqueBuf` has a safe `deallocate` implementation as part of its `Drop`
    /// implementation, but cannot be copied or cloned into multiple handles.
    ///
    pub unsafe fn deallocate(self) {
        if self.cap == 0 { return }
        alloc::deallocate(self.buffer, NonZero::new(self.cap));
    }

    /// Create a MemBuf from a previously allocated data pointer and a
    /// capacity.
    pub unsafe fn from_raw(data: NonZero<*mut T>, capacity: usize) -> MemBuf<T> {
        MemBuf {
            buffer: data,
            cap: capacity
        }
    }
}

impl<T> Deref for MemBuf<T> {
    type Target = *mut T;

    fn deref(&self) -> &*mut T {
        &*self.buffer
    }
}

#[cfg(test)]
mod test {
    use std::ptr;
    use alloc::empty;
    use MemBuf;

    #[test]
    fn test_empty() {
        let buffer: MemBuf<usize> = MemBuf::new();
        assert_eq!(buffer.cap, 0);
        assert_eq!(buffer.buffer, empty());
    }

    #[test]
    fn test_allocate() {
        let buffer: MemBuf<usize> = MemBuf::allocate(8);

        assert_eq!(buffer.cap, 8);

        unsafe {
            ptr::write(buffer.offset(0), 8);
            ptr::write(buffer.offset(1), 4);
            ptr::write(buffer.offset(3), 5);
            ptr::write(buffer.offset(5), 3);
            ptr::write(buffer.offset(7), 6);

            assert_eq!(ptr::read(buffer.offset(0)), 8);
            assert_eq!(ptr::read(buffer.offset(1)), 4);
            assert_eq!(ptr::read(buffer.offset(3)), 5);
            assert_eq!(ptr::read(buffer.offset(5)), 3);
            assert_eq!(ptr::read(buffer.offset(7)), 6);
        };

        // Try a large buffer
        let buffer: MemBuf<usize> = MemBuf::allocate(1024 * 1024);

        unsafe {
            ptr::write(buffer.offset(1024 * 1024 - 1), 12);
            assert_eq!(ptr::read(buffer.offset(1024 * 1024 - 1)), 12);
        };
    }

    #[test]
    fn test_reallocate() {
        let mut buffer: MemBuf<usize> = MemBuf::allocate(8);
        assert_eq!(buffer.cap, 8);

        unsafe { buffer.reallocate(16); }
        assert_eq!(buffer.cap, 16);

        unsafe {
            // Put some data in the buffer
            ptr::write(buffer.offset(0), 8);
            ptr::write(buffer.offset(1), 4);
            ptr::write(buffer.offset(5), 3);
            ptr::write(buffer.offset(7), 6);
        };

        // Allocate so in-place fails.
        let _: MemBuf<usize> = MemBuf::allocate(128);

        unsafe { buffer.reallocate(32); }

        unsafe {
            // Ensure the data is still there.
            assert_eq!(ptr::read(buffer.offset(0)), 8);
            assert_eq!(ptr::read(buffer.offset(1)), 4);
            assert_eq!(ptr::read(buffer.offset(5)), 3);
            assert_eq!(ptr::read(buffer.offset(7)), 6);
        };
    }

    #[test]
    #[should_panic = "Capacity overflow."]
    fn test_allocate_capacity_overflow() {
        let _: MemBuf<usize> = MemBuf::allocate(10_000_000_000_000_000_000);
    }

    #[test]
    #[should_panic = "Capacity overflow."]
    fn test_fresh_reallocate_capacity_overflow() {
        let mut buffer: MemBuf<usize> = MemBuf::new();
        unsafe { buffer.reallocate(10_000_000_000_000_000_000); }
    }

    #[test]
    #[should_panic = "Capacity overflow."]
    fn test_reallocate_capacity_overflow() {
        let mut buffer: MemBuf<usize> = MemBuf::allocate(128);
        unsafe { buffer.reallocate(10_000_000_000_000_000_000); }
    }
}

