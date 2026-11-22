// expect: ok, output "1 2 1\n40 none\n2 1\nac b"
use std.seq.{dedup, first_some, tally, group}
var a = [1, 1, 2, 1, 1]
a.dedup()
print(a)
print([1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None), [1, 2].first_some(x => if x > 3: Some(x) else: None))
let t = "abca".tally()
print(t['a'], t['b'])
let g = [(1, 'a'), (2, 'b'), (1, 'c')].group(by: p => p.0, of: p => p.1)
print(g[1], g[2])
