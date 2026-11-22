// NCCC 10 S2 (proto/ports/nccc10s2.cp), with a planted wrong DP order:
// the chains are built in label order, not in the order of the first list.
// The letters are numbered 1..=k here: `cpl run` reads a letter domain as
// numbers today.
use std.seq.{Perm}

let n: 1..=1000 = read()
let k: 1..=26 = read()
domain L = 1..=k
let orders: [n]Perm<L> = read()

fn ahead_in() -> [L, L]Bool:
    var ahead: [L, L]Bool = true
    for i in 0..n:
        for p in L:
            for q in L:
                if p < q: ahead[orders[i][q], orders[i][p]] = false
    ahead

fn longest() -> Int:
    let ahead = ahead_in()
    var best: [L]Int = 0
    for u in L:
        best[u] = (max(best[v] for v in L if v != u and ahead[v, u]) ?? 0) + 1
    max(best[u] for u in L)!

@alt(longest)
fn longest_slow() -> Int:
    let ahead = ahead_in()
    var best: [L]Int = 0
    for p in L:
        let u = orders[0][p]
        best[u] = (max(best[v] for v in L if v != u and ahead[v, u]) ?? 0) + 1
    max(best[u] for u in L)!

print(longest())
