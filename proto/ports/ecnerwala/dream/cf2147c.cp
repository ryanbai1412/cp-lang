// Codeforces 2147 C: Rabbits.
// Adjacent equal characters (`00` or `11`) decouple the string into independent
// maximal alternating runs (`chunk_by((a, b) => a != b)`):
// - If a run starts or ends with `'0'`, that endpoint `'0'` sits at a string
//   boundary or next to another `'0'`, so every `'0'` in the run can point
//   toward that free end without conflict.
// - Otherwise the run has the form `1010...1` of odd length `2k + 1` (`k`
//   isolated `'0'`s trapped between `'1'`s). Each rabbit must pair with a
//   neighbour across an interior `'1'`, which is possible iff `k` is even —
//   i.e., `r.len % 4 != 3`.
@!multitest
use std.seq.{chunk_by}
let _n: 1..=200_000 = read()
let s: Str = read()
let ok = all(s[r.start] != '1' or r.len % 4 != 3 for r in s.chunk_by((a, b) => a != b))
print(if ok: "YES" else: "NO")
