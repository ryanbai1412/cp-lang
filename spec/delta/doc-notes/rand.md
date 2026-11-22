# std.rand doc notes

Choices made where the spec is silent, and contradictions found, while bringing
`spec/std/rand.cp` to the doc standard. Fixing the exact bit-level algorithms is
deliberate: the module promises the same numbers from `cpl run` and the built C++,
which only holds if every implementation draws identically.

- `Rng.new`: chose state `U64(seed)` (two's complement for negative seeds) with no pre-mixing, so `Rng.new(0).next()` is the published splitmix64 first output `0xE220A8397B1DCDAF`.
- `Rng.next`: chose the standard splitmix64 step (add `0x9E3779B97F4A7C15`, then the 30/27/31 mix with the two standard multipliers); the module header already names splitmix64.
- `Rng.int`: chose Lemire's multiply-high map `start + ((U128(next()) * n) >> 64)` with one draw and no rejection; deterministic draw count keeps streams aligned, and the bias (at most `n / 2^64`) is irrelevant for contest ranges.
- `Rng.int` on an empty range: traps (the old doc already said so); stated as `empty: traps`.
- `Rng.float`: chose `F64(next() >> 11) * 2^-53`, the usual 53-bit map, exactly representable and never 1.0.
- `Rng.chance`: chose `float() < p`, always one draw even when `p <= 0` or `p >= 1`, so the stream does not depend on `p`.
- `Rng.pick`: chose position `int(0..len)` (zero-based, so any key type works); an empty input returns `None` without drawing.
- `Rng.shuffle`: chose the downward Fisher-Yates (`i` from `len-1` to `1`, swap with `int(0..=i)`); empty and one-element inputs draw nothing.
- `Rng.perm`: chose "identity `[D]D` then `shuffle`", so a seed fixes the permutation.
- `Rng.sample`: chose partial upward Fisher-Yates over positions (`i` in `0..k`, swap with `int(i..n)`), result in draw order; distinct means distinct positions, so repeated values may repeat; traps when `k` is outside `0..=len`.
- Contradiction: spec/lib/random-stress.md says the generator is xoshiro256** and lists `Rng.time()` and `rng.f64()`; the stub (and its module header) says splitmix64 and has `float()` and no `time()`. Docs follow the stub; one of the two needs to change. (resolved: stub kept (splitmix64, `float()`, no `time()`, as its doc says); chapter fixed)
- Contradiction: decisions.md 21.4 spells `.int(lo, hi)` (two arguments); the stub and random-stress.md take a range `int(lo..=hi)`. Docs follow the stub. (resolved: decision 21.4 wins: `int(lo, hi)`, both inclusive; gold updated; decision 167)
