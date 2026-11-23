// CF 1903E Geo Game (interactive). The squared distance between p and q is
// congruent to (p.x + p.y) + (q.x + q.y) mod 2, so the sum telescopes to
// par(start) + par(last point): the first player wins iff the last point has
// the start's parity. The player with the majority parity removes the other
// parity first and is left choosing (or forcing) the last point.
use std.rand.{Rng}

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

// Hidden data comes from typed reads (stress generates them); the opponent
// plays randomly from `seed`. Each statement line goes out with `print`; a
// reply comes back through `w.ask`.
@interactor
fn judge(w: Wire):
    let n: 1..=7 = read()
    let s = read<(0..=6, 0..=6)>()
    let pts = read<[n](0..=6, 0..=6)>()
    let seed: 0..=1e9 = read()
    var rng = Rng(seed)
    print(n)
    print(s.0, s.1)
    for k in 0..n - 1: print(pts[k].0, pts[k].1)
    let who: Str = w.ask("{pts[n - 1].0} {pts[n - 1].1}")
    assert who == "First" or who == "Second"
    let prog_first = who == "First"
    var used = List.filled(n, false)
    var prev = s
    var total = 0
    var reply = ""
    for turn in 0..n:
        var j = 0
        if (turn % 2 == 0) == prog_first:
            j = if turn == 0 and prog_first: w.ask("") else: w.ask(reply)
            reply = ""
        else:
            let free = [k for k in 1..=n if not used[k - 1]]
            j = free[rng.int(0, free.len - 1)]
            reply = "{j}"
            if turn == n - 1: print(j)
        assert 1 <= j and j <= n and not used[j - 1]
        used[j - 1] = true
        let dx = pts[j - 1].0 - prev.0
        let dy = pts[j - 1].1 - prev.1
        total += dx * dx + dy * dy
        prev = pts[j - 1]
    assert (total % 2 == 0) == prog_first
