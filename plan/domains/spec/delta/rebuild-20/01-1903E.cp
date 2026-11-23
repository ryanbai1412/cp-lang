// CF 1903E Geo Game (interactive). The squared distance between p and q is
// congruent to (p.x + p.y) + (q.x + q.y) mod 2, so the sum telescopes to
// par(start) + par(last point): the first player wins iff the last point has
// the start's parity. The player with the majority parity removes the other
// parity first and is left choosing (or forcing) the last point.
let wire = Wire()

@!multitest
let n: 1..=1e5 = read()
domain P = 1..=n
let s = read<(0..=1e9, 0..=1e9)>()
let pts = read<[P](0..=1e9, 0..=1e9)>()
let par = (s.0 + s.1) % 2
let same = List(P.filter(i => (pts[i].0 + pts[i].1) % 2 == par))
let diff = List(P.filter(i => (pts[i].0 + pts[i].1) % 2 != par))
let first = same.len >= diff.len
print(if first: "First" else: "Second")
// Our turns spend the parity we want gone; the other list is the fallback.
let pref = if first: diff else: same
let other = if first: same else: diff
var taken: [P]Bool = false
var ip = 0
var jo = 0
for turn in 0..n:
    if (turn % 2 == 0) == first:
        while ip < pref.len and taken[pref[ip]]: ip += 1
        while jo < other.len and taken[other[jo]]: jo += 1
        let j = if ip < pref.len: pref[ip] else: other[jo]
        taken[j] = true
        print(j)
    else:
        let j: P = read()
        taken[j] = true

@interactor
fn judge(w: Wire):
    let t = gen(1..=4)
    print(t)
    for _ in 0..t:
        let n = gen(1..=7)
        let pts = [(gen(0..=6), gen(0..=6)) for _ in 0..=n]
        print(n)
        for p in pts: print(p.0, p.1)
        let who: Str = read()
        assert who == "First" or who == "Second"
        let prog_first = who == "First"
        var used = List.filled(n + 1, false)
        var prev = 0
        var total = 0
        for turn in 0..n:
            var j = 0
            if (turn % 2 == 0) == prog_first:
                j = read()
            else:
                let free = [k for k in 1..=n if not used[k]]
                j = free[gen(0..=free.len - 1)]
                print(j)
            assert 1 <= j and j <= n and not used[j]
            used[j] = true
            let dx = pts[j].0 - pts[prev].0
            let dy = pts[j].1 - pts[prev].1
            total += dx * dx + dy * dy
            prev = j
        assert (total % 2 == 0) == prog_first
