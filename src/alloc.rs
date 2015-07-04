//! Typed Allocation Utilities
//!
//! Unlike std::rt::heap these check for zero-sized types, capacity overflow,
//! oom etc. and calculate the appropriate size and alignment themselves.

extern crate alloc;

use core::nonzero::NonZero;
use std::rt::heap;
use std::mem;

/// Allocate a new pointer to the heap with space for `cap` `T`s.
pub unsafe fn allocate<T>(cap: NonZero<usize>) -> NonZero<*mut T> {
    if mem::size_of::<T>() == 0 { return empty() }

    // Allocate
    let ptr = heap::allocate(allocation_size::<T>(cap), mem::align_of::<T>());

    // Check for allocation failure
    if ptr.is_null() { alloc::oom() }

    NonZero::new(ptr as *mut T)
}

/// Reallocate an allocation allocated with `allocate` or a previous call to
/// `reallocate` to be a larger or smaller size.
pub unsafe fn reallocate<T>(ptr: NonZero<*mut T>,
                            old_cap: NonZero<usize>,
                            new_cap: NonZero<usize>) -> NonZero<*mut T> {
    if mem::size_of::<T>() == 0 { return empty() }

    let old_size = unchecked_allocation_size::<T>(old_cap);
    let new_size = allocation_size::<T>(new_cap);

    // Reallocate
    let new = heap::reallocate(*ptr as *mut u8, old_size, new_size, mem::align_of::<T>());

    // Check for allocation failure
    if new.is_null() {
        alloc::oom()
    }

    NonZero::new(new as *mut T)
}

/// A zero-sized allocation, appropriate for use with zero sized types.
pub fn empty<T>() -> NonZero<*mut T> {
    unsafe { NonZero::new(heap::EMPTY as *mut T) }
}

/// Deallocate an allocation allocated with `allocate` or `reallocate`.
pub unsafe fn deallocate<T>(ptr: NonZero<*mut T>, cap: NonZero<usize>) {
    if mem::size_of::<T>() == 0 { return }

    let old_size = unchecked_allocation_size::<T>(cap);

    heap::deallocate(*ptr as *mut u8, old_size, mem::align_of::<T>())
}

fn allocation_size<T>(cap: NonZero<usize>) -> usize {
    mem::size_of::<T>().checked_mul(*cap).expect("Capacity overflow")
}

fn unchecked_allocation_size<T>(cap: NonZero<usize>) -> usize {
    mem::size_of::<T>() * (*cap)
}

