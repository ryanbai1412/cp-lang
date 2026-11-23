// cf1923d Slimes: slime i is eaten from the left in 1 second if a[i-1] > a[i];
// otherwise the eater is a merged block j..=i-1 that must hold two different
// sizes (it starts before the run containing i-1) and outweigh a[i]. The
// largest such j comes from a binary search on prefix sums. The right side is
// the same on the mirrored array.
@!multitest
let n: 1..=3e5 = read()
domain I = 1..=n
let a = read<[I]1..=1e9>()

fn from_left(a: [I]Int) -> [I]?Int:
    let pre = a.scan(+)
    var start: [I]I = I.first!
    for r in a.runs():
        for i in r.keys: start[i] = r.keys.first!
    var out: [I]?Int = None
    for i in I:
        let j = i.prev else: continue
        if a[j] > a[i]:
            out[i] = 1
            continue
        let l = start[j].prev else: continue
        let x = (I.first!..=l).last_true(x => pre[j] - pre[x.l] > a[i]) else: continue
        out[i] = Int(i) - Int(x)
    out

fn mirror(i: I) -> I = I(n + 1 - Int(i))
let left = from_left(a)
let right = from_left([I] i => a[mirror(i)])
print([I] i => min(left[i], right[mirror(i)]), none: -1)
