// Codeforces 2159 A — Interactive Duplicate-Finding Pair Recovery
//
// Each value in `1..=n` appears twice across positions `P = 1..=2n`, and
// `ask(s)` returns the maximum value appearing twice in `s` (or 0 if all
// elements in `s` are distinct).
//
// Pass 1: grow `cur` from `1..=2n`. Whenever `ask(cur) = res != 0`, the new
// position `p` is the second occurrence of `res`, so record `ans[p] = res` and
// move `p` to `oth`. After pass 1, `oth` contains one copy of every value
// `1..=n` and `cur` holds the `n` remaining first occurrences.
// Pass 2: for each `p in cur`, querying `oth + [p]` uniquely returns `ans[p]`.

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
