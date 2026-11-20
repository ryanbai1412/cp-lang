// NOI '20 P4, Dish Making (multi-answer constructive, buildable today-form)
// original: corpus/submissions/3890337.cpp
// The m = n - 2 subset sum keeps one Bool per (prefix, offset) in a flat
// List<Bool>: 8x the memory of the bitset the full limits need, so this form
// is verified on small and medium cases only.
let T: 1..=10 = read()

var rem: List<Int> = []          // grams left of each ingredient of this case
var k = 0

// Uses up the ingredients in `group` with exactly `dishes` dishes
// (dishes >= group.len - 1 and their total is dishes * k).
fn greedy(group: List<Int>, dishes: Int):
    var live: List<Int> = []
    for i in group:
        if rem[i] > 0:
            live.push(i)
    for made in 0..dishes:
        var a = 0
        var b = 0
        for p in 0..live.len:
            if rem[live[p]] < rem[live[a]]:
                a = p
            if rem[live[p]] >= rem[live[b]]:
                b = p
        let i = live[a]
        let j = live[b]
        let small = rem[i]
        if dishes - made > live.len - 1:
            print(j + 1, k)
            rem[j] -= k
        else:
            print(i + 1, small, j + 1, k - small)
            rem[i] = 0
            rem[j] -= k - small
        var keep: List<Int> = []
        for q in live:
            if rem[q] > 0:
                keep.push(q)
        live = keep

for t in 0..T:
    let n: 1..=500 = read()
    let m: 1..=5000 = read()
    let kk: 1..=5000 = read()
    k = kk
    rem = []
    for i in 0..n:
        let d: Int = read()
        rem.push(d)
    var everyone: List<Int> = []
    for i in 0..n:
        everyone.push(i)
    if m >= n - 1:
        greedy(everyone, m)
    else:
        // offsets d - k summed over a subset; find a subset summing to -k
        var mn = 0
        var mx = 0
        for i in 0..n:
            mn = min(mn, mn + rem[i] - k)
            mx = max(mx, mx + rem[i] - k)
        let W = mx - mn + 1
        var reach: List<Bool> = []
        for o in 0..W:
            reach.push(o == -mn)
        for i in 0..n:
            let c = rem[i] - k
            for o in 0..W:
                var r = reach[i * W + o]
                if o - c >= 0 and o - c < W and reach[i * W + o - c]:
                    r = true
                reach.push(r)
        var off = -k - mn
        if off < 0 or not reach[n * W + off]:
            print(-1)
        else:
            var S: List<Int> = []
            var rest: List<Int> = []
            for back in 0..n:
                let i = n - 1 - back
                if reach[i * W + off]:
                    rest.push(i)
                else:
                    S.push(i)
                    off -= rem[i] - k
            greedy(S, S.len - 1)
            greedy(rest, rest.len - 1)
