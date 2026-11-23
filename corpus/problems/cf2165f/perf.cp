// Codeforces 2165 F: 21435, tuned: orig.cpp's two flat max segment trees over
// values, written as bottom-up loops on one list each, in place of
// `SegTree<V, ?I>(max)` (whose `set` pulls `op` through every parent and whose
// `fold` builds tuples). The "1"s stack loop runs inside the answer sweep, so
// no per-position list of opened "2"s is built.

// Both trees are written out on purpose: this is the tuned variant.
with lint(hand_rolled_range_tree: allow)
@!multitest
let n: 1..=1_000_000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()

// shapes[k]: (value of "3", position of "5") for each "435" whose "4" is at k.
var shapes: [I]List<(Int, Int)> = []
var seg = List.filled(2 * n, -1)
var stk: List<I> = []
for i in I:
    let x = a[i].ord
    while stk.len > 0 and a[stk.last!].ord < x:
        let t = stk.pop()!
        var (lo, l, r) = (-1, a[t].ord + 1 + n, x + n)
        while l < r:
            if l & 1 == 1:
                lo max= seg[l]
                l += 1
            if r & 1 == 1:
                r -= 1
                lo max= seg[r]
            l >>= 1
            r >>= 1
        if lo != -1: shapes[I(lo)].push((a[t].ord, i.ord))
    stk.push(i)
    var p = x + n
    while p > 0:
        seg[p] = i.ord
        p >>= 1

// best[r]: latest "2" left of a "1" < "3" for a 21435 ending by r. A "2" opens
// when the stack of "1" candidates pops it.
var two = List.filled(2 * n, -1)
var best = List.filled(n, -1)
stk.clear()
for i in I:
    for (v, r) in shapes[i]:
        var (l, h) = (n, v + n)
        while l < h:
            if l & 1 == 1:
                best[r] max= two[l]
                l += 1
            if h & 1 == 1:
                h -= 1
                best[r] max= two[h]
            l >>= 1
            h >>= 1
    while stk.len > 0 and a[i] < a[stk.last!]:
        let j = stk.pop()!
        var p = a[j].ord + n
        while p > 0:
            two[p] max= j.ord
            p >>= 1
    stk.push(i)

var (run, total) = (-1, 0)
for b in best:
    run max= b
    total += run + 1
print(total)
