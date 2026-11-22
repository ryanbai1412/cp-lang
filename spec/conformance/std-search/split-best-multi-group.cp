// expect: ok, output "9 8 7 6"
// rule: 07.7
use std.search.{split_best}
let rows = [[2, 9, 4], [8, 6, 7]]
let res = split_best([0, 1], range: g => 0..rows[g].len, best: (g, r) => r.map(i => (rows[g][i], i)).max()!).collect()
print(res[0], res[1], res[2], res[3])
