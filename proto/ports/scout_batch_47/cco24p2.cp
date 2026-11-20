// CCO '24 P2 — Pizza Party (DMOJ: cco24p2)
// original: corpus/submissions/6435481.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given sequences a and b of length n (values in 1..=n), assign each element of
// a and b to one of k stacks so that pushing a in order and popping b in order
// succeeds with minimum k (or print -1 if a and b have different multisets).
// Ports the exact verified subtask logic (impossible, binary values, distinct
// permutations via reverse patience sorting / Dilworth LIS).
let n: 1..=1e6 = read()
domain I = 0..n
domain Val = 1..=n
let a: [I]Val = read()
let b: [I]Val = read()

fn tally(xs: [I]Val) -> [Val]Int:
    var cnt: [Val]Int = 0
    for i in I: cnt[xs[i]] += 1
    cnt

if tally(a) != tally(b):
    print(-1)
    return

var ra = a.copy()
ra.reverse()

// Case 1: binary values (all values in 1..=2).
if max(a[i].to_idx for i in I)! <= 1 and max(b[i].to_idx for i in I)! <= 1:
    if ra == b:
        print(1)
        print(" ".join("1" for _ in I))
        print(" ".join("1" for _ in I))
    else:
        print(2)
        print(" ".join("{a[i]}" for i in I))
        print(" ".join("{b[i]}" for i in I))
    return

// Case 2: distinct permutations — map b into positions of rev(a), and partition
// into decreasing subsequences (minimum stack count via patience sort).
fn is_perm(xs: [I]Val) -> Bool:
    var seen: [Val]Bool = false
    for i in I:
        if seen[xs[i]]: return false
        seen[xs[i]] = true
    true

if not is_perm(a) or not is_perm(b): return

var pos_rev: [Val]Int = 0
for i in I: pos_rev[ra[i]] = i.to_idx

// Maintain stack tops in strictly decreasing order; each element joins the
// rightmost stack whose top is < cx (equivalently, first_ge on reversed tops).
var tops: List<Int> = []
var stack_id: [Val]Int = 0
for i in I:
    let x = b[i]
    let cx = pos_rev[x]
    // Find largest k in 0..tops.len with tops[k] < cx; tops is increasing left-to-right
    // if we store them from highest stack id down, or decreasing if stored 1..k.
    // Stored as decreasing tops[0] > tops[1] > ..., first index with tops[j] < cx
    // (or append a new stack if none).
    let slot = bsearch_first(0..tops.len, j => tops[j] < cx)
    if let j = slot:
        tops[j] = cx
        stack_id[x] = j + 1
    else:
        tops.push(cx)
        stack_id[x] = tops.len

print(tops.len)
print(" ".join("{stack_id[a[i]]}" for i in I))
print(" ".join("{stack_id[b[i]]}" for i in I))
