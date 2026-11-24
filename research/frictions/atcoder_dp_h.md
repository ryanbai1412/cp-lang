# atcoder_dp_h

Ported from `corpus/problems/atcoder_dp_h/orig.cpp`. Dream: `corpus/problems/atcoder_dp_h/dream.cp`.

## No code gaps
The port and the dream are the same program. `for c in Col if grid[r, c] == '.'` skips walls,
and `if let up = r.prev` / `if let left = c.prev` take the edges with no padding row or column.
`ways: [Row, Col]Z` keeps the modulus in the type.

## 1.2x on a 9 ms run
- **Where:** `--max` (1000 x 1000), median of 15: port 9.4 ms, `orig.cpp` 7.8 ms.
- **What felt wrong:** this is over the 1.1x bar, by 1.6 ms. `perf record` shares the time
  between `prog::solve` 33%, `cpl::read_chars` 15% (the grid read), kernel start-up and page
  faults 22%, and `vector<Mod>::_M_fill_assign` 7%. The last is the `= 0` fill of `ways`, which
  `orig.cpp` gets free from a zeroed global array.
- **Goal it hurts:** idiomatic-code/R10.
- **Ideal code:** unchanged.
- **General fix:** integer tables filled with 0 are emitted as `cpl::zero_grid`. This
  `Mod<P>` table is emitted as `cpl::grid<Mod_1000000007_17, 2>({Row_len, Col_len},
  Mod_1000000007_17_new(0))`, a fill with a constructed value. `Mod<P>` wraps one integer,
  and its zero is all zero bits, so it can take the `zero_grid` path too.
- **Severity:** low.
- **Owner:** Core (codegen): repro `cpl build corpus/problems/atcoder_dp_h/port.cp -o a.cpp`,
  `grep 'cpl::grid<Mod' a.cpp`, then `perf record` at `--max`.

## Measurements
- Tokens: port 164, original 426.
- `--max`: port 9.4 ms / 7.5 MB, `orig.cpp` 7.8 ms / 7.4 MB (1.21x). Same output.
- Emitted C++: 15,027 bytes.
