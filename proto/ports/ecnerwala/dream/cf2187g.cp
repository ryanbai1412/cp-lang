// Codeforces 2187 G — Tree Walk Reconstruction
//
// `s[u][v] == '1'` marks reachability in the ordered binary tree on `0..n`
// (`ch1[i]` = rightmost child of `i` to its right). Starting at node `0`
// (value `1`), each step follows the unique valid successor:
// - Move up to the leftmost left-parent `p0 < cur` iff `cur` is `p0`'s
//   rightmost child (`ch1[p0] == cur`);
// - Otherwise step to the rightmost right-parent `p1 > cur`.
// Because every non-root node has a unique next edge in this traversal and only
// the final node (`v == n`) has neither, the walk visits each index of `0..n`
// exactly once and assigns a permutation of `1..=n`.

@!multitest
let n: 1..=3_000 = read()
let s: [0..n, 0..n]Char = read()
let ch1 = [0..n] i => first(j for j in (i + 1..n).rev if s[j, i] == '1')

var ans: [0..n]Int = 0
var cur = 0
for v in 1..=n:
    ans[cur] = v
    let pa0 = first(j for j in 0..cur if s[cur, j] == '1')
    let pa1 = first(j for j in (cur + 1..n).rev if s[cur, j] == '1')
    if let p0 = pa0 and ch1[p0] == cur: cur = p0
    elif let p1 = pa1: cur = p1
print(ans)
