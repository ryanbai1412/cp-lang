# 20. Intrinsics

**Layer:** core → library boundary. The complete list of functions the library may call that aren't written in the language: see `00-layers.md`, section 1.

Status: [proposed].

| intrinsic | C++ |
|---|---|
| `bytes_in.next() -> ?Byte`, `bytes_out.write(Byte)`, `flush()` | buffered `fread`/`fwrite` |
| `alloc<T>(n)`, `free` | `new T[n]` |
| `checked_add`/`sub`/`mul`, `wrapping_*`, `overflowing_*` | `__builtin_*_overflow`, unsigned casts |
| `mulhi(u64, u64)`, `I128` arithmetic | `__int128` |
| `popcount`, `ctz`, `clz` | `__builtin_*` |
| `trap(msg)` | message to stderr, `exit(1)` (debug: stack trace) |
| `stderr.write`, `clock()` | `fputs`, `chrono` |
| `rng(seed)` | splitmix64; recorded by the shrinker |
| `@cpp(...)` | the escape hatch (chapter 32) |
