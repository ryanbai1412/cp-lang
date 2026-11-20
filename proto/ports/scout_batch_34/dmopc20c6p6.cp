// DMOPC '20 Contest 6 P6 — Water Jugs (Interactive, CP Lang v2 spec)
// original: corpus/submissions/5200785.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Interactive capacity discovery: `n` jugs start full with hidden capacities
// in `1..=m`. Operations: `E x` (empty), `F x` (fill), `D x y` (pour x -> y,
// discarding overflow from x, returning amount added to y), `A c1..cn` (answer).

let (n, m): (2..=40_000, 1..=1e9) = read()
domain Jug = 1..=n

// FRICTION UD3: `ask(...)` on the v2 cheat sheet assumes every query receives
// a reply. Here `E x` and `F x` are one-way commands that still consume the
// shared query budget (`<= 4*n`), while `D x y` returns an `Int`. Mixing
// `print` with `ask` splits the query budget unless `send(...)` or `ask<Unit>`
// shares the interactor counter.
fn empty_jug(x: Jug):
    print("E {x}")

fn fill_jug(x: Jug):
    print("F {x}")

fn pour_jug(x: Jug, y: Jug) -> Int:
    ask("D", x, y)

var cap: [Jug]?Int = none
var best_jug: ?Jug = none

fn record_cap(u: Jug, val: Int):
    cap[u] = val
    if let b = best_jug:
        if val > cap[b]!: best_jug = u
    else:
        best_jug = u

empty_jug(Jug.first)
var full_known: ?Jug = none
var scan: ?Jug = Jug.first

while let cur = scan:
    if cap[cur] != none:
        scan = cur + 1
        continue
    var acc = 0
    var probe = cur + 1
    while true:
        while let p = probe and cap[p] != none:
            probe = p + 1
        var poured_probe: ?Int = none
        if let p = probe:
            let x1 = pour_jug(p, cur)
            acc += x1
            poured_probe = x1
            if x1 == 0: break
        if acc == m: break
        let x2 = if let f = full_known:
            full_known = none
            pour_jug(f, cur)
        else:
            let src = best_jug ?? (Jug.first + 1)!
            fill_jug(src)
            pour_jug(src, cur)
        acc += x2
        if x2 == 0 or acc == m: break
        if let p = probe and let x1 = poured_probe:
            record_cap(p, x1)
            probe = p + 1
    record_cap(cur, acc)
    full_known = cur
    scan = probe

print("A " + " ".join("{cap[u]!}" for u in Jug))

@interactor
fn judge():
    let hidden: [Jug]Int = read()
    var cur = hidden.copy()
    var queries = 0
    while true:
        let op: Str = read_output()
        if op == "E":
            let x: Jug = read_output()
            cur[x] = 0
            queries += 1
        elif op == "F":
            let x: Jug = read_output()
            cur[x] = hidden[x]
            queries += 1
        elif op == "D":
            let (x, y): (Jug, Jug) = read_output()
            let moved = min(hidden[y] - cur[y], cur[x])
            cur[x] = 0
            cur[y] += moved
            queries += 1
            print(moved)
        elif op == "A":
            let guess: [Jug]Int = read_output()
            assert guess == hidden
            assert queries <= 4 * n
            break
        else:
            assert false
