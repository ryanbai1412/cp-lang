// n: 512
// fixed: A
use std.strings.{suffix_array, suffix_tree_edges}
let n: Int = read()
let ss = [Str([Char(Int('a') + (i * 7919 + i / 3) % 3) for i in 0..n]) for _ in 0..2]
var acc = 0
for r in 0..256:
    let s = ss[r % 2]
    acc += suffix_array(s).len // timed: strings.suffix_array
    acc += suffix_tree_edges(s).len // timed: strings.suffix_tree_edges
print(acc)
