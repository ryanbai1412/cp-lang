// BTOI '06 P5 (RLE Compression): re-encode a run-length stream with the
// fewest codes by choosing when to switch the escape symbol.
// cost[e] = shortest encoding of the remaining runs when the escape is e.
// Every update is "add to all", "assign to all" or "set one", so the table
// is an offset, a default and the symbols touched since the last assign.
// Runnable on proto/cpc today.
// original: corpus/submissions/4156310.cpp
let n: 2..=100_000 = read()
let m: 1..=2_000_000 = read()
domain Sym = 0..n
let code: [0..m]Sym = read()

// decode into maximal runs (symbol, length)
var sym: List<Sym> = []
var len: List<Int> = []
fn add_run(s: Sym, k: Int):
    if sym.len > 0 and sym.last! == s: len[len.len - 1] += k
    else:
        sym.push(s)
        len.push(k)
var esc = Sym(0)
var at = 0
while at < m:
    let c = code[at]
    if c != esc:
        add_run(c, 1)
        at += 1
    else:
        let b = code[at + 1]
        let k = code[at + 2].idx
        if b == esc: add_run(esc, k + 1)
        elif k == 0: esc = b
        else: add_run(b, k + 3)
        at += 3

let runs = sym.len
domain R = 0..runs
var offset = 0
var dflt = 0
var epoch = 0
var stamp: [Sym]Int = -1
var cost: [Sym]Int = 0
var touched = Set<(Int, Int)>()
var free = 0
fn get(s: Sym) -> Int:
    return offset + (if stamp[s] == epoch: cost[s] else: dflt)

// switch_to[i]: escape to switch to before run i when the escape is the
// run's own symbol; adopt[i]: switch to the run's symbol from anything else
var switch_to: [R]?Sym = none
var adopt: [R]Bool = false
for i in (0..runs).rev:
    let s = sym[i]
    let k = len[i]
    var x = get(s)
    offset += 3 * (k / (n + 2)) + min(3, k % (n + 2))
    if stamp[s] == epoch: touched.remove((cost[s], s.idx))
    stamp[s] = epoch
    // cheapest other escape: an untouched symbol at the default, or a touched one
    while free < n and stamp[Sym(free)] == epoch: free += 1
    var opt = offset + dflt
    var opt_sym = Sym(min(free, n - 1))
    if let (c, e) = touched.min():
        if free == n or c < dflt or (c == dflt and e < free):
            opt = offset + c
            opt_sym = Sym(e)
    x += 3 * ((k + n - 1) / n)
    if x > opt + 3:
        switch_to[R(i)] = opt_sym
        x = opt + 3
    if opt > x + 3:
        adopt[R(i)] = true
        touched.clear()
        epoch += 1
        dflt = x + 3 - offset
        free = 0
    stamp[s] = epoch
    cost[s] = x - offset
    touched.insert((cost[s], s.idx))

var out: List<Int> = []
esc = Sym(0)
for i in 0..runs:
    let s = sym[i]
    var k = len[i]
    var next: ?Sym = none
    if esc == s: next = switch_to[R(i)]
    elif adopt[R(i)]: next = s
    if let e = next:
        out.push(esc.idx)
        out.push(e.idx)
        out.push(0)
        esc = e
    if esc == s:
        while k > 0:
            let take = min(k, n)
            k -= take
            out.push(esc.idx)
            out.push(esc.idx)
            out.push(take - 1)
    else:
        while k > 3:
            let take = min(k, n + 2)
            k -= take
            out.push(esc.idx)
            out.push(s.idx)
            out.push(take - 3)
        for _ in 0..k: out.push(s.idx)
print(out.len)
print(out)
