@!multitest
let n: 1..=2e5 = read()
let p: [0..n]1..=n = read()
let ps = p.prev_smaller()
let ns = p.next_smaller()
var ans = 0
for i in 0..n:
    let l = i - (ps[i] ?? -1) - 1
    let r = (ns[i] ?? n) - i - 1
    ans += if min(l, r) == 0: l + r + 1 else: 2
print(ans)

@alt(program)
fn slow():
    var cnt: Map<(Int, Int), Int> = Map()
    for l in 0..n:
        var m = n + 1
        for r in l..n:
            m = min(m, p[r])
            cnt[(r - l, m)] = (cnt.get((r - l, m)) ?? 0) + 1
    print(cnt.values.count(c => c == 1))

@gen
fn make_case(size: Int):
    print(1)
    let n = rand(1, size)
    print(n)
    var ys = [i for i in 1..=n]
    shuffle(mut ys)
    print(ys)
