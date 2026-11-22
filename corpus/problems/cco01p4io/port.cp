// CCO '01 P4 — Two-Egg Drop (Interactive) (today form for proto/cpc)
// original: corpus/submissions/5034487.cpp
let n: 1..=100 = ask("GetN")
let d: 1..=100 = ask("GetD")

domain M = 0..105
var memo: [M]Int = -1
var opt: [M]Int = 0

fn dp(len: Int) -> Int:
    if len <= 1: return 0
    if memo[M(len)] != -1: return memo[M(len)]
    var best = 1_000_000_000
    for i in 0..len:
        let rem = dp(len - i - 1)
        let cx = (if i > rem: i else: rem) + 1
        if cx <= best:
            best = cx
            opt[M(len)] = i
    memo[M(len)] = best
    best

let worst = dp(n)
assert worst <= d

fn search(l: Int, r: Int):
    if l == r:
        print("Decide", l)
        return
    let _ = dp(r - l)
    let i = opt[M(r - l)]
    let res: Int = ask("Drop", l + i)
    if res != 0:
        for j in l..l + i:
            let b: Int = ask("Drop", j)
            if b != 0:
                print("Decide", j)
                return
        print("Decide", l + i)
    else:
        search(l + i + 1, r)

search(1, n)
