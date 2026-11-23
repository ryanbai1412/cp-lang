// NOI '19 P3 (Sequence): choose k indices from a and k from b with >= l overlap to
// maximize total sum. Starts with the top k items in bucket 2 (both), then iteratively
// relaxes overlap down to l by releasing the worst bucket-2 item and running two
// successive augmenting-path steps over buckets {0: a-only, 1: b-only, 2: both}.
// v2.2 SortedBag replaces ~90 lines of custom lazy-deletion multi-PQ bookkeeping.
// original: corpus/submissions/5669058.cpp

use std.bag.{SortedBag}
use std.seq.{next_permutation}

@multitest
let (n, k, l): (Int, Int, Int) = read()
domain I = 0..n
let a: [I]Int = read()
let b: [I]Int = read()

fn val(s: Int, i: I) -> Int =
    match s:
        0 => a[i]
        1 => b[i]
        2 => a[i] + b[i]
        _ => 0

// States 0, 1, 2 = buckets (a-only, b-only, both); state 3 = unused.
var bags: [0..4, 0..3]SortedBag<(Int, I)> = SortedBag()
var own: SortedBag<(Int, I)> = SortedBag()
var sz: [0..4]Int = 0
var cap: [0..3]Int = [0, 0, k]
var flow = 0

fn insert_state(s: Int, i: I):
    sz[s] += 1
    if s == 2: own.add((val(2, i), i))
    for t in 0..3 if t != s: bags[s, t].add((val(t, i) - val(s, i), i))

fn erase_state(s: Int, i: I):
    sz[s] -= 1
    if s == 2: own.remove((val(2, i), i))
    for t in 0..3 if t != s: bags[s, t].remove((val(t, i) - val(s, i), i))

fn move_item(i: I, fr: Int, to: Int):
    if fr < 3: cap[fr] += 1
    if to < 3: cap[to] -= 1
    flow += val(to, i) - val(fr, i)
    erase_state(fr, i)
    insert_state(to, i)

for i in I: insert_state(3, i)
for i in sorted(I, by: i => -(a[i] + b[i]))[..k]: move_item(i, 3, 2)

fn augment():
    var best: ?Int = None
    var best_path: List<Int> = []
    var perm = [0, 1, 2]
    while true:
        var prv = 3
        var gain = 0
        var path: List<Int> = []
        var ok = true
        for nxt in perm:
            if let pair = bags[prv, nxt].last:
                gain += pair.0
                path.push(nxt)
                if cap[nxt] > 0: break
                if sz[nxt] == 0:
                    ok = false
                    break
                prv = nxt
            else:
                ok = false
                break
        if ok and cap[path.last!] > 0 and (best == None or gain > best!):
            best = gain
            best_path = path.copy()
        if not next_permutation(mut perm): break
    var prv = 3
    for nxt in best_path:
        move_item(bags[prv, nxt].last!.1, prv, nxt)
        prv = nxt

var ans = flow
while sz[2] > l and sz[3] > 0:
    move_item(own.first!.1, 2, 3)
    cap[2] -= 1
    cap[0] += 1
    cap[1] += 1
    augment()
    augment()
    ans max= flow

print(ans)
