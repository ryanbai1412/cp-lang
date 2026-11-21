// OLY 2023 Practice 5, buildable under proto/cpc today. See ../oly23practice5.cp.
// original: corpus/submissions/6415646.cpp
@multitest
fn solve():
    let n: 1..=300_000 = read()
    domain I = 0..n
    let a: [I]Int = read()

    var pen = Map<(Int, Int), Int>()
    for i in 0..(n - 1):
        if i > 0 and a[I.from_idx(i + 1)!] == a[I.from_idx(i - 1)!]: continue
        var j = i + 2
        while j < n and a[I.from_idx(j)!] == a[I.from_idx(i + (j - i) % 2)!]:
            j += 1
        var ca = a[I.from_idx(i)!]
        var cb = a[I.from_idx(i + 1)!]
        if i % 2 == 0:
            swap(mut ca, mut cb)
        let k = (ca, cb)
        let prev = pen.get(k) ?? 0
        pen[k] = prev + (j - i) / 2

    var even: [0..=n]Int = 0
    var odd: [0..=n]Int = 0
    for i in 0..n:
        let x = a[I.from_idx(i)!]
        if i % 2 == 0: even[x] += 1
        else: odd[x] += 1

    var ans = n
    for (k, extra) in pen:
        let x = k.0
        let y = k.1
        if x != y:
            ans min= n - even[x] - odd[y] + extra

    var by_odd: List<(Int, Int)> = [(-odd[y], y) for y in 0..=n]
    sort(mut by_odd)

    for x in 0..=n:
        for p in by_odd:
            let y = p.1
            if y != x and pen.get((x, y)) == none:
                ans min= n - even[x] - odd[y]
                break
    print(ans)
