// IOI 2015 "Sorting": Ermek swaps two positions each round, then we swap two;
// sort the array within Ermek's rounds.
//
// Our swaps, picked as pairs of values, commute with his position swaps: R is the
// least r whose row after Ermek's first r swaps needs at most r swaps. Each cycle
// c0 -> c1 -> ... of that row is sorted by swapping the values (c0, c1), (c1, c2), ...;
// in round t we swap wherever the t-th pair now sits.
//
// stdin as the IOI sample grader: n, the permutation, m, then Ermek's m swaps.
let n: 1..=200_000 = read()
domain Pos = 0..n
let start: [Pos]Pos = read()
let m: 1..=600_000 = read()
let ermek: [m](x: Pos, y: Pos) = read()

fn after(r: Int) -> [Pos]Pos:
    var a = start.clone()
    for e in ermek.take(r): a.swap_at(e.x, e.y)
    a

let rounds = (0..=m).first_true(r => n - after(r).perm_cycles().len <= r)!
let plan = after(rounds).perm_cycles().flat_map(c => c.adjacent()).collect()

var now = start.clone()
var at = now.inverse()                              // where each value sits
fn swap(x: Pos, y: Pos):
    now.swap_at(x, y)
    at[now[x]] = x
    at[now[y]] = y

print(rounds)
for (t, e) in ermek.take(rounds).items:
    swap(e.x, e.y)
    let (u, v) = plan.get(t) ?? (now[Pos(0)], now[Pos(0)])
    let (pu, pv) = (at[u], at[v])
    print(pu, pv)
    swap(pu, pv)
