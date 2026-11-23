// Codeforces 1981C Turtle and an Incomplete Sequence: fill the -1s so neighbours halve each other.
// In: t; per test n, then a'_1..a'_n (-1 or 1..1e8). Out: b_1..b_n on one line, or -1.
@!multitest
let n = read<2..=2e5>()
domain I = 0..n
let a = read<[I]-1..=1e8>()

// The tree path x -> lca -> y in the binary heap where v's parent is v / 2.
fn path(x: Int, y: Int) -> List<Int>:
    var (u, v) = (x, y)
    var up: List<Int> = []
    var down: List<Int> = []
    while u != v:
        if u > v:
            up.push(u)
            u /= 2
        else:
            down.push(v)
            v /= 2
    up.push(u)
    for w in down.rev: up.push(w)
    up

var b: [I]Int = 0
var ok = true
let known = I.filter(i => a[i] > 0).collect()
if known.is_empty:
    for i in I: b[i] = 1 + i.ord % 2
else:
    let first = known.first!
    let last = known.last!
    for i in first.before: b[i] = if (first.ord - i.ord) % 2 == 1: a[first] * 2 else: a[first]
    for i in last.after: b[i] = if (i.ord - last.ord) % 2 == 1: a[last] * 2 else: a[last]
    for (i, j) in known.adjacent:
        let p = path(a[i], a[j])
        let d = p.len - 1
        let len = j.ord - i.ord
        if d > len or (len - d) % 2 == 1:
            ok = false
            break
        // Walk the path, then bounce between a[j] * 2 and a[j] until j.
        for q in i..=j:
            let s = q.ord - i.ord
            b[q] = if s <= d: p[s] elif (s - d) % 2 == 1: a[j] * 2 else: a[j]
if ok: print(b) else: print(-1)
