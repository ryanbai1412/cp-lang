// Source: Codeforces 2129C1 (Interactive RBS, Easy Version),
//   https://codeforces.com/contest/2129/problem/C1
// Problem: interactive, multitest (t <= 20). A hidden bracket string s of length n (2 <= n <=
//   1000) has at least one '(' and one ')'. A query lists k <= 1000 indices (repeats allowed)
//   and returns the number of balanced substrings of the string they spell. Find s in at
//   most 550 queries.
// Solution: binary search the shortest prefix with an answer > 0: it ends in "()", giving a
//   known '(' and ')'; if the whole string answers 0 it is ")...(", so s[n] s[1] is "()".
//   Then each pair (p1, p2) costs one query, [p1, ')', '(', p2, '(', p2], whose answer
//   0, 1, 3 or 6 tells both brackets (odd n repeats the last index). log n + n/2 queries.

@!interactive

@!multitest
let n: 2..=1_000 = read()
domain Pos = 1..=n
var ans: [Pos]Char = '?'

fn ask(inds: List<Pos>) -> Int:
    print("?", inds.len, inds)
    read()

var (mi, ma) = (0, n)
while ma - mi > 1:
    let md = (mi + ma) / 2
    if ask([Pos(i) for i in mi + 1..=md + 1]) > 0: ma = md else: mi = md
let open_i = Pos(mi + 1)
let close_i = Pos((mi + 1) % n + 1)

fn check2(p1: Pos, p2: Pos):
    let r = ask([p1, close_i, open_i, p2, open_i, p2])
    ans[p1] = if r == 1 or r == 6: '(' else: ')'
    ans[p2] = if r == 0 or r == 1: '(' else: ')'

for i in 1..=n by 2: check2(Pos(i), Pos(min(i + 1, n)))
print("!", Str(ans))
