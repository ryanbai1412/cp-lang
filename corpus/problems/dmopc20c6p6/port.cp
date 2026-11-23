// Runnable port (proto/cp.py lib-run) of dmopc20c6p6.cp
// original: corpus/submissions/5200785.cpp
// Runs with an inline local grader (matching `-DLOCAL` in the C++ original)
// because `proto/cpi` buffers all stdin tokens upfront.

let (n, m): (Int, Int) = read()
domain Jug = 1..=n
let hidden: [Jug]Int = read()

var cur_water: [Jug]Int = [Jug] u => hidden[u]
var queries = 0

fn empty_jug(x: Jug):
    queries += 1
    cur_water[x] = 0

fn fill_jug(x: Jug):
    queries += 1
    cur_water[x] = hidden[x]

fn pour_jug(x: Jug, y: Jug) -> Int:
    queries += 1
    let moved = min(hidden[y] - cur_water[y], cur_water[x])
    cur_water[x] = 0
    cur_water[y] += moved
    moved

var cap: [Jug]?Int = [Jug] _ => None
var best_jug: ?Jug = None

fn record_cap(u: Jug, val: Int):
    cap[u] = val
    if let b = best_jug:
        if val > cap[b]!: best_jug = u
    else:
        best_jug = u

empty_jug(Jug.first)
var full_known: ?Jug = None
var scan = 1

while scan <= n:
    while scan <= n and cap[Jug(scan)] != None:
        scan += 1
    if scan > n: break
    let cur = Jug(scan)
    scan += 1
    var acc = 0
    while true:
        while scan <= n and cap[Jug(scan)] != None:
            scan += 1
        var poured_probe: ?Int = None
        if scan <= n:
            let x1 = pour_jug(Jug(scan), cur)
            acc += x1
            poured_probe = x1
            if x1 == 0: break
        if acc == m: break
        var x2 = 0
        if let f = full_known:
            x2 = pour_jug(f, cur)
            full_known = None
        else:
            let src = best_jug ?? Jug(2)
            fill_jug(src)
            x2 = pour_jug(src, cur)
        acc += x2
        if x2 == 0 or acc == m: break
        if scan <= n:
            if let x1 = poured_probe:
                record_cap(Jug(scan), x1)
                scan += 1
    record_cap(cur, acc)
    full_known = cur

for u in Jug:
    assert cap[u]! == hidden[u]

var out = "A"
for u in Jug:
    out += " {cap[u]!}"
print(out)
