// CCO '23 P2 — Real Mountains (DMOJ: cco23p2)
// original: corpus/submissions/5579928.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given mountain heights h[0..n], compute modulo 1_000_003 the minimum cost to
// make the sequence unimodal (no valley strictly below both its prefix and
// suffix maximum).
//
// Sweep heights from lowest to highest using an ordered `Map<Int, List<V>>`,
// maintaining the active interior indices in an ordered `Set<V>` (trimming
// endpoints that reach their prefix/suffix cap via `while let ... and`), then
// resolve offline prefix/suffix next-greater queries with `Set<Int>.first_gt`.
const MOD = 1_000_003
type Z = Mod<MOD>

let n: 1..=1e6 = read()
domain V = 0..n
let h: [V]Int = read()

let pref = scan(h, max)
var suff: [V]Int = 0
var run_max = 0
for i in V.rev:
    run_max max= h[i]
    suff[i] = run_max
let cap: [V]Int = [V] i => min(pref[i], suff[i])

var by_h = Map<Int, List<V>>()
for i in V: (by_h[h[i]] ??= []).push(i)

fn tri(x: Int) -> Z: Z(x * (x + 1) / 2)
fn sum_range(l: Int, r: Int) -> Z: tri(r) - tri(l - 1)

var pref_q: [V]List<(x: Int, mul: Int)> = [V] _ => []
var suff_q: [V]List<(x: Int, mul: Int)> = [V] _ => []
var active = Set<V>()
var ans: Z = 0
var prev_h: ?Int = none

for (cur_h, idxs) in by_h.items:
    if let lo_h = prev_h:
        if not active.is_empty:
            let first = active.first!
            let last = active.last!
            let span = cur_h - lo_h
            if active.len == 1:
                ans += sum_range(lo_h, cur_h - 1)
                pref_q[first].push((lo_h, span))
                suff_q[first].push((lo_h, span))
            else:
                ans += Z(active.len) * sum_range(lo_h, cur_h - 1)
                    + Z(active.len * 2 - 3) * sum_range(lo_h + 1, cur_h)
                pref_q[first].push((lo_h, span))
                suff_q[last].push((lo_h, span))
                pref_q[V.last].push((lo_h, span))
    for i in idxs: active.add(i)
    while let f = active.first and cur_h >= cap[f]: active.pop_first()
    while let l = active.last and cur_h >= cap[l]: active.pop_last()
    prev_h = cur_h

var seen_l = Set<Int>()
for i in V:
    seen_l.add(h[i])
    for q in pref_q[i]: ans += Z(q.mul) * seen_l.first_gt(q.x)!

var seen_r = Set<Int>()
for i in V.rev:
    seen_r.add(h[i])
    for q in suff_q[i]: ans += Z(q.mul) * seen_r.first_gt(q.x)!

print(ans)
