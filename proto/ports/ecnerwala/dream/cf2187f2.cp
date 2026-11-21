// Codeforces 2187 F2: Al Fine (counting version).
// Build the permutation tree `PermTree.of(b.rev.map(a.inverse()))`:
//   - An increasing node multiplies its two children's counts.
//   - A maximal decreasing chain multiplies the counts of its non-leaf right
//     children and bottom-most child by `leaf_ways(rights)`, the number of ways
//     to interleave its leaf right-children (a bottom-up prefix-sum DP where
//     each leaf right-child shifts the running prefix sums by one slot).

type Z = Mod<998_244_353>

@!multitest
let n: 2..=5000 = read()
domain V = 1..=n
let (a, b): (Perm<V>, Perm<V>) = read()
let t = PermTree.of(b.rev.map(a.inverse()))

fn leaf_ways(rights: List<t.Node>) -> Z:
    var ends: List<Z> = [1]
    for r in rights.rev:
        ends.push(0)
        if t[r] is leaf: ends = ends.scan(+)
    ends.sum()

fn count(v: t.Node) -> Z = match t[v]:
    leaf => 1
    incr(l, r) => count(l) * count(r)
    decr => chain(v)

fn chain(top: t.Node) -> Z:
    let spine = successors(top, v => if t[v] is decr: t.left(v) else: none)
    let rights = spine.drop_last().map(t.right)
    let non_leaf_prod = rights.filter(r => t[r] is not leaf).map(count).product()
    non_leaf_prod * count(spine.last!) * leaf_ways(rights)

print(count(t.root))
