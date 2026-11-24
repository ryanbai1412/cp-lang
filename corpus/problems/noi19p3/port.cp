// Source: NOI 2019 P3 (Sequence), https://dmoj.ca/problem/noi19p3
// limits: 1000 ms, 511 MB
// Problem: multitest (T <= 10). Given a, b of length n <= 2e5 (sum n <= 1e6, values <= 1e9) and
//   1 <= L <= K <= n, pick K indices c and K indices d sharing at least L indices; print the max
//   of the a-sum over c plus the b-sum over d.
// Solution: a simulated cost flow, one augmenting path per chosen pair. Each step takes the best
//   of: an unused index into both sides; an index already in d into c, with the best b outside
//   d (and its mirror); or, while fewer than K - L unpaired picks are used, the best a outside c
//   with the best b outside d. A pick that completes a pair hands its unpaired slot back. Five
//   lazy `Heap`s keep each candidate set; membership only grows, so stale tops are popped once.
//   O(n log n) per test.
// original: corpus/submissions/5669058.cpp
// Bounds: n: 1..=2e5; k: 1..=n; l: 1..=k; a, b: 1..=1e9

@multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let l: 1..=k = read()
domain I = 0..n
let a: [I]1..=1e9 = read()
let b: [I]1..=1e9 = read()

var in_c: [I]Bool = false
var in_d: [I]Bool = false
// Max-heaps as (-value, index): outside c by a, outside d by b, outside both by a + b,
// in d only by a, in c only by b.
var a_out = Heap.of([(-a[i], i) for i in I])
var b_out = Heap.of([(-b[i], i) for i in I])
var both_out = Heap.of([(-a[i] - b[i], i) for i in I])
var a_of_d = Heap<(Int, I)>.new()
var b_of_c = Heap<(Int, I)>.new()

fn best(mut h: Heap<(Int, I)>, live: I -> Bool) -> ?(gain: Int, i: I):
    while let Some((v, i)) = h.peek:
        if live(i): return Some((gain: -v, i:))
        h.pop()
    None

fn into_c(i: I):
    in_c[i] = true
    if not in_d[i]: b_of_c.push((-b[i], i))

fn into_d(i: I):
    in_d[i] = true
    if not in_c[i]: a_of_d.push((-a[i], i))

var spare = k - l
var total = 0
repeat k:
    let fa = best(mut a_out, i => not in_c[i])!
    let fb = best(mut b_out, i => not in_d[i])!
    let pair = best(mut both_out, i => not in_c[i] and not in_d[i])
    let ad = best(mut a_of_d, i => not in_c[i])
    let bc = best(mut b_of_c, i => not in_d[i])
    let gains = [pair.map(p => p.gain), ad.map(x => x.gain + fb.gain), bc.map(x => x.gain + fa.gain)]
    let top = gains.map(g => g ?? -1).max()!
    if spare > 0 and fa.i != fb.i and fa.gain + fb.gain > top:
        total += fa.gain + fb.gain
        spare -= 1
        if in_d[fa.i]: spare += 1
        if in_c[fb.i]: spare += 1
        into_c(fa.i)
        into_d(fb.i)
    elif gains[0] == Some(top):
        total += top
        into_c(pair!.i)
        into_d(pair!.i)
    elif gains[1] == Some(top):
        total += top
        if in_c[fb.i]: spare += 1
        into_c(ad!.i)
        into_d(fb.i)
    else:
        total += top
        if in_d[fa.i]: spare += 1
        into_d(bc!.i)
        into_c(fa.i)
print(total)
