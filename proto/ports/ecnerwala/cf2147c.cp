// Codeforces 2147 C: split into maximal alternating runs; any run starting
// with '1' of length 3 mod 4 makes the arrangement impossible.
@!multitest
use std.seq.{chunk_by}
let _n: Int = read()
let s: Str = read()
let ok = all(s[r.start] != '1' or r.len % 4 != 3 for r in s.chunk_by((a, b) => a != b))
print(if ok: "YES" else: "NO")
