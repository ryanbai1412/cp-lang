// DMOPC '21 Contest 8 P3 — Constructive Permutation via Order-Statistic Set (CP Lang v2 spec)
// original: corpus/submissions/4537755.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Construct a permutation of `1..=n` with exact score `k` (or `-1` if `k`
// exceeds the maximum). Replaces `__gnu_pbds::tree` (`find_by_order`) in the
// C++ original with `Set<Pos>.nth(offset)` from the v2 collection prelude, and
// replaces O(n)-deep recursion with a flat loop.

let (n, k): (1..=1e6, 0..=1e18) = read()
domain Pos = 0..n
domain Val = 1..=n

let cap = k + 1
var ways: [0..=n]Int = 0
for i in 1..=n:
    let l = i / 2
    let r = i - l - 1
    // Cap at `k + 1` so `ways[i]` never overflows `Int` even for huge `n`.
    let add = min(cap - ways[i - 1], l * r)
    ways[i] = ways[i - 1] + add

if k > ways[n]:
    print(-1)
    return

var offsets: [Val]Int = 0
var rem = k
for v in Val:
    let tot = n - v.to_idx
    if rem <= ways[tot - 1]:
        offsets[v] = 0
    elif tot == 5 and rem == 4:
        offsets[v] = 1
        rem -= 3
    else:
        let l = (tot - 1) / 2
        offsets[v] = l
        rem -= l * (tot - l - 1)

// FRICTION UD13: Initializing an ordered `Set<Pos>` with all elements of a
// domain requires a loop (`for p in Pos: avail.add(p)`); there is no
// `Set.from(Pos)` constructor, and `pop_nth(i)` would avoid the `nth` + `remove`
// double tree traversal (`O(log n)` twice per element).
var avail = Set<Pos>()
for p in Pos: avail.add(p)

var perm: [Pos]Val = Val.first
for v in Val:
    let p = avail.nth(offsets[v])!
    perm[p] = v
    avail.remove(p)

print(" ".join("{perm[p]}" for p in Pos))
