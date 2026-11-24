// Source: Codeforces 2159A (MAD Interactive Problem),
//   https://codeforces.com/contest/2159/problem/A
// Problem: interactive, multitest. A hidden sequence of length 2n holds each of 1..n twice
//   (2 <= n <= 300, sum of n^2 <= 1e5). A query of distinct indices returns the largest
//   value appearing twice among them, or 0. Find the sequence in at most 3n queries.
// Solution: grow a set cur of positions one at a time; a nonzero answer res means the new
//   position p is the second copy of res: record it and move p to oth. Then cur and oth each
//   hold one copy of every value, so asking oth plus one p of cur names p's value. 2n + n
//   queries of O(n) indices each.
// Bounds: n: 1..=300

@!interactive

@!multitest
let n: 1..=300 = read()
domain P = 1..=2 * n

fn ask(s: List<P>) -> Int:
    print("?", s.len, s)
    read()

var ans: [P]Int = 0
var cur: List<P> = []
var oth: List<P> = []

for p in P:
    cur.push(p)
    let res = ask(cur)
    if res != 0:
        ans[p] = res
        cur.pop()
        oth.push(p)

for p in cur:
    oth.push(p)
    ans[p] = ask(oth)
    oth.pop()

print("!", ans)
