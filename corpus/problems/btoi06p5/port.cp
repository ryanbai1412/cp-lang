// Source: BOI 2006 P5 (RLE Compression), https://dmoj.ca/problem/btoi06p5
// limits: 5000 ms, 64 MB
// Problem: alphabet 0..n-1 (n <= 1e5) and a code of m <= 2e6 symbols with escape e (initially
//   0): "e e k" is k + 1 copies of e, "e b 0" makes b the escape, "e b k" (k > 0) is k + 3
//   copies of b, any other symbol is itself. Print a shortest code for the same sequence: its
//   length, then the symbols.
// Solution: decode into maximal runs, then DP backwards: cost[e] = shortest code of the rest
//   with escape e. A run of s costs the same under every escape but s; escape s pays
//   3 ceil(k / n); then s may switch (+3) to the best other escape, or all others adopt s (+3).
//   So updates are add-all, assign-all and set-one: an offset, an epoch-stamped default and
//   an ordered set of touched (cost, symbol) for the minimum. O(m + runs log n), then a
//   forward pass replays the recorded switches to emit the code.
// original: corpus/submissions/4156310.cpp
// Bounds: n: 2..=100_000; m: 1..=2_000_000; code: 0..=n - 1
let n: 2..=100_000 = read()
let m: 1..=2_000_000 = read()
domain Sym = 0..n
let code: [0..m]Sym = read()

// decode into maximal runs (symbol, length)
var sym: List<Sym> = []
var len: List<Int> = []
fn add_run(s: Sym, k: Int):
    if sym.len > 0 and sym.last! == s: len.last! += k
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
var switch_to: [R]?Sym = None
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
    x += 3 * (ceil_div(k, n))
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
    var next: ?Sym = None
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
