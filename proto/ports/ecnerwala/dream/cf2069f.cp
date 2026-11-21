// cf2069f as it should read: answers over time, edges alive on ranges.
let (n, q): (2..=4e5, 1..=4e5) = read()
domain V = 1..=n
domain T = 0..q
let qs: [T](g: Char, e: Pair<V>) = read()

let alive = toggles(T, qs, key: q => q)          // Map<(Char, Pair<V>), List<Range<T>>>
var (a, ab) = (Dsu<V>(), Dsu<V>())
let ans = offline_over(T, alive, with: (a, ab)) ((g, e), _) =>
    if g == 'A': a.unite(e.a, e.b)
    ab.unite(e.a, e.b)
at t => a.components - ab.components
print(ans, sep: "\n")
