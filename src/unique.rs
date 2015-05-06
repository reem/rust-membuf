use std::ops::Deref;
use MemBuf;

/// A safe wrapper around a heap allocated buffer of Ts, tracking capacity only.
///
/// MemBuf makes no promises about the actual contents of this memory, that's up
/// to the user of the structure and can be manipulated using the standard pointer
/// utilities, accessible through the impl of `Deref<Target=*mut T>` for `UniqueBuf<T>`.
///
/// As a result of this hands-off approach, `UniqueBuf`s destructor does not attempt
/// to drop any of the contained elements; the destructor simply frees the contained
/// memory.
///
/// You can think of `UniqueBuf<T>` as an approximation for `Box<[T]>` where the elements
/// are not guaranteed to be valid/initialized. It is meant to be used as a building
/// block for other collections, so they do not have to concern themselves with the
/// minutiae of allocating, reallocating, and deallocating memory.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct UniqueBuf<T> {
    inner: MemBuf<T>
}

unsafe impl<T: Send> Send for UniqueBuf<T> {}
unsafe impl<T: Sync> Sync for UniqueBuf<T> {}

impl<T> UniqueBuf<T> {
    /// Create a new, empty UniqueBuf.
    ///
    /// ```
    /// # use membuf::UniqueBuf;
    ///
    /// let buffer: UniqueBuf<usize> = UniqueBuf::new();
    /// assert_eq!(buffer.capacity(), 0);
    /// ```
    pub fn new() -> UniqueBuf<T> {
        UniqueBuf { inner: MemBuf::new() }
    }

    /// Create a new buffer with space for cap Ts.
    ///
    /// Unlike `std::rt::heap::allocate`, cap == 0 is allowed.
    ///
    /// ```
    /// # use membuf::UniqueBuf;
    ///
    /// let buffer: UniqueBuf<usize> = UniqueBuf::allocate(128);
    /// assert_eq!(buffer.capacity(), 128);
    /// ```
    pub fn allocate(cap: usize) -> UniqueBuf<T> {
        UniqueBuf { inner: MemBuf::allocate(cap) }
    }

    /// Reallocate this buffer to fit a new number of Ts.
    ///
    /// Unlike `std::rt::heap::reallocate`, cap == 0 is allowed.
    ///
    /// ```
    /// # use membuf::UniqueBuf;
    ///
    /// let mut buffer: UniqueBuf<usize> = UniqueBuf::allocate(128);
    /// assert_eq!(buffer.capacity(), 128);
    ///
    /// buffer.reallocate(1024);
    /// assert_eq!(buffer.capacity(), 1024);
    /// ```
    pub fn reallocate(&mut self, cap: usize) {
        unsafe { self.inner.reallocate(cap) }
    }

    /// Get the current capacity of the UniqueBuf.
    ///
    /// ```
    /// # use membuf::UniqueBuf;
    ///
    /// let buffer: UniqueBuf<usize> = UniqueBuf::allocate(128);
    /// assert_eq!(buffer.capacity(), 128);
    /// ```
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Create a UniqueBuf from an existing MemBuf.
    ///
    /// ```
    /// # use membuf::{MemBuf, UniqueBuf};
    ///
    /// let buffer = unsafe { UniqueBuf::from_raw(MemBuf::<u8>::allocate(256)) };
    /// assert_eq!(buffer.capacity(), 256);
    /// ```
    pub unsafe fn from_raw(buffer: MemBuf<T>) -> UniqueBuf<T> {
        UniqueBuf { inner: buffer }
    }
}

impl<T> Drop for UniqueBuf<T> {
    fn drop(&mut self) {
        unsafe { self.inner.deallocate() }
    }
}

impl<T> Deref for UniqueBuf<T> {
    type Target = *mut T;

    fn deref(&self) -> &*mut T { &*self.inner }
}

