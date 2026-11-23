// expect: ok, output "3\n3"
// input: "2 1 2\n1 3\n"
// rule: 10.2.3
// `[m]List<T>` reads m count-prefixed rows for a range element type too.
let rows = read<[2]List<1..=9>>()
for r in rows: print(r.sum())
