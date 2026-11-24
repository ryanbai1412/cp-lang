// expect: ok, output "6\n2\n-1\n1\n3\n2"
// rule: 01.10.1
// A generator argument is the free form of the method fold (decision 53): `max(e for x in s)` is `s.map(x => e).max()`, a `?T`.
domain Row = 0..2
domain Col = 0..3
let xs = [3, 1, 2]
let g: [Row, Col]Char = [Row, Col] (r, c) => if (Int(r) + Int(c)) % 2 == 0: '#' else: '.'
print(max(x * 2 for x in xs)!)
print(min(x * 2 for x in xs)!)
print(min(x for x in xs[0..0]) ?? -1)
print(max(g[.., c].count(x => x == '#') for c in Col)!)
print(sum(g[r].count(c => c == '#') for r in Row))
print(count(x > 1 for x in xs))
