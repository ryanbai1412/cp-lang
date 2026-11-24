# atcoder_dp_l

Ported from `corpus/submissions/4131776.cpp`. Dream: `corpus/problems/atcoder_dp_l/dream.cp`.

## A range table is 3.3x slower than the C++
- **Where:** `var lead: [I.ranges]Int` and the reads `lead[r.drop(1)]`, `lead[r.drop_last(1)]`.
- **What felt wrong:** median of 15 at `--max` (n = 3000): port 77.7 ms, original 23.7 ms. Two
  causes, both in the emitted C++.
  1. Layout. The table is indexed by `cpl::sym_idx(lo, hi - 1) = b*(b+1)/2 + a`
     (`crates/cpl-codegen/src/runtime/ranges.rs`), which is end-major. `for r in I.ranges` walks
     by length, and both reads are in the previous length, so every access strides across
     36 MB. With a length-major index (`l*(n+1) - l*(l+1)/2 + a`, l = b - a) patched into the
     emitted C++, the output is identical and the time is 47.0 ms.
  2. Index arithmetic. Each `lead[r.drop(1)]` is about 12 lines of C++: a `Range<int>` copy,
     an `an empty range has no cell` check, and `iadd`/`min` with an overflow guard for a
     `drop(1)` the loop already proves in range.
- **Goal it hurts:** idiomatic-code/R10; "the obvious way is the correct and fast way".
- **Ideal code:** unchanged. The port is the dream.
- **General fix:** store `[V.ranges]` tables length-major (the table knows `n`), and fold
  `drop`/`drop_last`/`splits` index arithmetic when the range's length is known to be >= 2.
- **Severity:** medium (3.3x; inside the time limit by 25x).
- **Owner:** Core (codegen); repro and numbers sent to Core.

## Measurements
- Tokens: port 118, original 321.
- `--max`: port 77.7 ms / 36.9 MB, original 23.7 ms / 47.6 MB. Same output.
- Emitted C++: 14,258 bytes.
