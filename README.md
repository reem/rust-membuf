# membuf

> A safe-ish wrapper for allocating, reallocating and deallocating heap buffers.

## Overview


A safe wrapper around a heap allocated buffer of Ts, tracking capacity only.

MemBuf makes no promises about the actual contents of this memory, that's up
to the user of the structure and can be manipulated using the standard pointer
utilities, accessible through the impl of `Deref<Target=*mut T>` for `MemBuf<T>`.

You can think of `MemBuf<T>` as an approximation for `Box<[T]>` where the elements
are not guaranteed to be valid/initialized. It is meant to be used as a building
block for other collections, so they do not have to concern themselves with the
minutiae of allocating, reallocating, and deallocating memory.

However, note that `MemBuf<T>` does not have a destructor, and implements `Copy`,
as a result, it does not implement `Send` or `Sync`, and it is the responsibility
of the user to call `deallocate` if they wish to free memory.

There is also a `UniqueBuf<T>` which does not implement `Copy`, implements
`Send` and `Sync`, and has a destructor responsible for deallocation.

## Usage

Use the crates.io repository; add this to your `Cargo.toml` along
with the rest of your dependencies:

```toml
[dependencies]
membuf = "*"
```

## Author

[Jonathan Reem](https://medium.com/@jreem) is the primary author and maintainer
of membuf.

## License

MIT

