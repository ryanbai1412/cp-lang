// Source: Codeforces 2268 D (AghaBalaSar and Hamed), https://codeforces.com/contest/2268/problem/D
// limits: 2000 ms, 256 MB
// Problem: a permutation p of length n; from i one step goes to any j < i or to the first
//   j > i with p_j > p_i. f(i, j) is the fewest steps from i to j (0 if unreachable); print
//   the sum of f over all pairs. Multitest, sum of n <= 1e6.
// Solution: pairs a > b cost 1 each. a < b is reachable iff both lie in one prefix-max
//   block [l..=r], the spine l -> up[l] -> ... -> r of the next-greater forest. Charge each
//   spine edge i -> u in closed form: pairs crossing it, pairs leaving (i, u) to the right
//   (1 step via u's other children, else 2), and pairs inside (i, u] at distance 1, 2 or 3,
//   counted from up[], first child and child counts. Every block is O(its length): O(n).
use std.seq.*

@!multitest
let n: 1..=1e6 = read()
domain I = 0..n
let p: [I](1..=n) = read()

let up = p.next_smaller(by: x => desc(x))
var first: [I]?I = None
var kids: [I]Int = 0
for i in I if let u = up[i]:
    if first[u] == None: first[u] = i
    kids[u] += 1

fn charge_edge(l: I, r: I, i: I, u: I) -> Int:
    let inside = (u - i) - 1
    let cross = (i - l + 1) * (r - i + inside)
    let exit_right = (r - u) * (2 * inside - (kids[u] - 1))
    let total_in = (u - i) * inside / 2
    var two = 0
    if let st = i.next:
        two = sum((up[a]! - a) - 1 for a in st..u)
        for b in st..=u if let fb = first[b]:
            two += (b - fb) - kids[b] + (if b < u: kids[fb] else: 0)
    let three = total_in - inside - two
    cross + exit_right + inside + 2 * two + 3 * three

var ans = n * (n - 1) / 2
var cur: ?I = I.first
while let l = cur:
    var r = l
    while let u = up[r]: r = u
    var i = l
    while let u = up[i]:
        ans += charge_edge(l, r, i, u)
        i = u
    cur = r.next
print(ans)
