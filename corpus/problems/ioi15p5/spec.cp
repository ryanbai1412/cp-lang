// IOI 2015 "Sorting": each round Ermek swaps two positions, then we swap two;
// sort in as few rounds as possible. After r rounds Ermek's swaps alone leave
// an arrangement that n - cycles value swaps sort, so r rounds suffice iff that
// is at most r; one round moves each side by at most one, so binary search r.
// Our swaps exchange values: plan them on that final arrangement (put value i
// at position i, in turn) and replay each at the values' current positions.
//
// The task is a grader function; this reads the sample grader's input (N; S;
// M; M lines X Y) and prints R and our R pairs, `0 0` once already sorted.
use std.seq.{perm_cycles}

let n = read<1..=2e5>()
domain Pos = [n]
let start = read<[Pos]Pos>()
let m = read<1..=6e5>()
let ermek = read<[m](x: Pos, y: Pos)>()

fn after(r: Int) -> [Pos]Pos:
    var a = start.clone()
    for e in ermek.take(r):
        a.swap_at(e.x, e.y)
    a

// Swap positions x and y of arrangement `a`, keeping `at` its inverse.
fn swap(mut a: [Pos]Pos, mut at: [Pos]Pos, x: Pos, y: Pos):
    a.swap_at(x, y)
    (at[a[x]], at[a[y]]) = (x, y)

let rounds = (0..=m).first_true(r => n - perm_cycles(after(r)).len <= r)!
var end = after(rounds)
var end_at = end.inverse()
var plan: List<(Pos, Pos)> = []
for i in Pos:
    if end[i] != i:
        plan.push((i, end[i]))
        swap(mut end, mut end_at, i, end_at[i])

var now = start.clone()
var at = now.inverse()
print(rounds)
for (t, e) in ermek.take(rounds).items:
    swap(mut now, mut at, e.x, e.y)
    let (u, v) = plan.get(t) ?? (now[Pos.first!], now[Pos.first!])
    let (pu, pv) = (at[u], at[v])
    print(pu, pv)
    swap(mut now, mut at, pu, pv)
