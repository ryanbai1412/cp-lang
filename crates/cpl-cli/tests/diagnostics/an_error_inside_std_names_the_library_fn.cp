use std.prefix.{scan}

let n: 1..=10 = read()
domain Pos = 0..n
let s: [Pos]Str = read()
let best = s.scan(max)
print(best.all)
