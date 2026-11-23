// expect: ok, output "0 1"
let a: List<Int> = []
print([r.len for r in a.runs(link: (x, y) => y == x + 1)].len, [r.len for r in [7].runs(link: (x, y) => y == x + 1)][0])
