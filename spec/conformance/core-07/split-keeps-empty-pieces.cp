// expect: ok, output "1 0 1\n1 0\n0\n2 4"
// rule: 07.7.10
use std.seq.{split}
print([p.len for p in [1, 0, 0, 2].split(x => x == 0)])
print([p.len for p in [1, 0].split(x => x == 0)])
let e: List<Int> = []
print([p.len for p in e.split(x => x == 0)])
for p in [7, 0, 3, 4][2..].split(x => x == 0): print(p.keys.first!, p.len + p[p.keys.first!] - 1)
