// MCCC 4 J4: players in groups play round robins (W = 3 points to the first, T = 1 each,
// L = 3 to the second). Rank each group by points, then by id, and print each group's k-th.
// copies: 1
let (n, m): (Int, Int) = read()
domain P = 1..=n
let groups = read<[n / m][m]P>()
var points: [P]Int = 0
for (u, v, res) in read<[(m - 1) * n / 2](P, P, Char)>():
    match res:
        'W' => points[u] += 3
        'T' =>
            points[u] += 1
            points[v] += 1
        _ => points[v] += 3
let k: Int = read()
print(groups.map(g => List(g).sorted(by: p => (desc(points[p]), p))[k - 1]))
