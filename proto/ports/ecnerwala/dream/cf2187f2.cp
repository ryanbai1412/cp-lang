// Codeforces 2187 F2: Al Fine, counting version (dream). Same tree as F1:
// an increasing node multiplies its children's counts; a maximal chain of
// decreasing nodes multiplies the counts of its non-leaf right children by the
// number of ways to place its leaf pattern, a prefix-sum DP read bottom-up.
type Z = Mod<998_244_353>
@!multitest
let n: 2..=5000 = read()
domain V = 1..=n
let (a, b): (Perm<V>, Perm<V>) = read()
let t = PermTree.of(b.rev.map(a.inverse()))    // shared with F1, one line

fn count(v: t.Node) -> Z = match t[v]:
    leaf => 1
    incr(l, r) => count(l) * count(r)
    decr => chain(v)

fn chain(top: t.Node) -> Z:
    let spine = successors(top, v => t.left(v) if t[v] is decr)   // top-down, ends below the chain
    let rights = spine.drop_last().map(t.right)
    var ends: List<Z> = [1]
    for r in rights.rev:
        ends.push(0)
        if t[r] is leaf: ends.prefix_sums(in_place: true)
    product(count(r) for r in rights if t[r] is not leaf) * count(spine.last!) * ends.sum()

print(count(t.root))
