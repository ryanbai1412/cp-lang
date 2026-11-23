@!multitest
let n: 1..=2e5 = read()
let p: [0..n]1..=n = read()
let ps = p.prev_smaller()
let ns = p.next_smaller()
var total = 0
for i in 0..n:
    let left = i - (ps[i] ?? -1)
    let right = (ns[i] ?? n) - i
    total += if left == 1 or right == 1: left + right - 1 else: 2
print(total)

@alt(program)
fn slow():
    var cnt = HashMap<(Int, Int), Int>()
    for l in 0..n:
        var m = n + 1
        for r in l..n:
            m = min(m, p[r])
            cnt[(r - l, m)] = (cnt.get((r - l, m)) ?? 0) + 1
    print(cnt.values.count(c => c == 1))

@gen
fn make_case(size: Int):
    let m = rand(1, size)
    var q = [i for i in 1..=m]
    shuffle(q)
    print(1)
    print(m)
    print(q)
