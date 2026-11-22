// cco21p2 Weird Numeral System: write x as sum d_i k^i with every digit d_i from a given set.
// The lowest digit d must have d = x (mod k), and then x becomes (x - d) / k. Breadth-first
// search over those values finds a way down to 0; its arcs, read backwards, are the digits.
let (k, q, d, m) = read<(2..=1_000_000, 1..=5, 1..=5_001, 0..=2_500)>()
var by_residue: [0..k]List<Int> = []
for a in read<[d]-m..=m>():
    by_residue[a % k].push(a)
fn peel(x: Int) = by_residue[x % k].map(a => (to: (x - a) / k, digit: a))
for x in read<[q]Int>():
    let p = bfs(peel, from: [x], until: 0, hashed: true)
    if let arcs = p.arcs(0):
        print(arcs.rev.map(a => a.digit))
    else:
        print("IMPOSSIBLE")
