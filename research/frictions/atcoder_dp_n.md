# atcoder_dp_n

Ported from `corpus/submissions/4131779.cpp`. Dream: `corpus/problems/atcoder_dp_n/dream.cp`.

## A range table is 1.8x slower than the C++
- **Where:** `var cost: [I.ranges]Int` and `min(cost[x] + cost[y] for (x, y) in r.splits)!`.
- **What felt wrong:** median of 15 at `--max` (n = 400): port 35.4 ms, original 20.1 ms. This
  has the same causes as atcoder_dp_l: the end-major `sym_idx` layout and the per-read
  index lambda. Each split's two reads emit about 20 lines of C++ through a
  `cpl_each_..._splits` callback. With length-major indexing patched in, it runs in 25.7 ms with
  the same output.
- **Goal it hurts:** idiomatic-code/R10.
- **Ideal code:** unchanged.
- **General fix:** the one in atcoder_dp_l: length-major `[V.ranges]` storage and folded split
  indices.
- **Severity:** medium.
- **Owner:** Core (codegen); sent together with atcoder_dp_l.

## The dream spells the reducer differently
`dream.cp` writes `r.splits.map((x, y) => cost[x] + cost[y]).min()!`, and the port writes
`min(cost[x] + cost[y] for (x, y) in r.splits)!`. The corpus uses the comprehension reducer for
"the least of an expression over a sequence" (idiomatic-code/R8), so the dream should take the
port's line. That is a `dream.cp` edit (owner: idioms, this package), landing with the next
`.cp`-edit commit.

## Measurements
- Tokens: port 111, original 368.
- `--max`: port 35.4 ms / 3.1 MB, original 20.1 ms / 3.9 MB. Same output.
- Emitted C++: 16,076 bytes.
