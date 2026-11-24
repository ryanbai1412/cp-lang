// Source: Codeforces 2152 E (Monotone Subsequence), https://codeforces.com/contest/2152/problem/E
// Problem: interactive, multitest (sum of n^2 + 1 <= 10001). A hidden permutation of length
//   n^2 + 1 (n <= 100); query "? k i_1 < .. < i_k" returns the indices of the left-to-right
//   maxima among them. In at most n queries, print "! s_1 .. s_{n+1}", the indices of an
//   increasing or decreasing subsequence of length n + 1.
// Solution: query all indices not yet returned; the maxima form an increasing run, so a reply
//   longer than n is printed at once. Otherwise label them layer i. After n rounds some index
//   is unlabelled, and each element left after round i has a larger layer-i element to its
//   left; walking back from it through layers n - 1..0 gives a decreasing sequence. O(n^3).
// Bounds: n: 1..=100

@!interactive
@!multitest
let n: 1..=100 = read()
domain Pos = 1..=n * n + 1
var layer: [Pos]?Int = None

for i in 0..n:
    let q = [v for v in Pos if layer[v] == None]
    print("?", q.len, q)
    let cnt: Int = read()
    let got: [0..cnt]Pos = read()
    for v in got: layer[v] = i
    if cnt > n:
        print("!", [v for v in Pos if layer[v] == i][..=n])
        return

var cur = Pos.find(u => layer[u] == None)!
var ans: List<Pos> = [cur]
for j in (0..n).rev:
    while layer[cur] != j: cur = cur.prev!
    ans.push(cur)
ans.reverse()
print("!", ans)
