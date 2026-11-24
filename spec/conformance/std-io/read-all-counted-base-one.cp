// expect: ok, output "2 0 1 9 2"
// input: "1 2 3 4 5 6 7\n"
// rule: 08.1.2 10.2.2
// `read_all` under `io(base: 1)` reads counted positions 1-based and fixed arrays at their length, as the caller's own `read<T>()` would.
use std.io.{read_all}
with io(base: 1)
domain E = [3]
let xs = read_all<E>(until: x => x == E(2))
let rest = read_all<[2]Int>()
print(xs.len, Int(xs[0]), Int(xs[1]), rest[0].sum(), rest.len)
