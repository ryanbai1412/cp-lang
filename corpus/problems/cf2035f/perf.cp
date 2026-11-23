// Codeforces 2035 F: Tree Operations, tuned.
// port.cp's check with `ops / n` and `ops % n` worked out once per check, as
// orig.cpp does, instead of inside `finish` for every vertex.

@!multitest
let n: 1..=2000 = read()
domain V = 1..=n
let root: V = read()
let a: [V]0..=1e9 = read()
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), root)

fn settle(need: Int) -> Int = if need < 0: -need % 2 else: need

fn works(ops: Int) -> Bool:
    let each = ops / n
    let extra = ops % n
    t.up(+, finish: (s, v) => settle(s + a[v] - each - Int(v.label <= extra)))[root] == 0

let cycle = 2 * n
let rounds = 0..=2 * (n + a.max()!)
print(
    [
        k * cycle + r for r in 0..cycle if let k = rounds.first_true(k => works(k * cycle + r))
    ].min() ?? -1
)
