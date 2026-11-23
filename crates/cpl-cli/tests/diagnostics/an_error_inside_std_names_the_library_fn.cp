use std.strings.{SuffixArray}

let n: 1..=10 = read()
domain Pos = 0..n
let s: [Pos]Char = read()
let sa = SuffixArray(s)
print(sa.len)
