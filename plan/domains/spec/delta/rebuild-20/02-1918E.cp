// CF 1918E ace5 and Task Order (interactive). Randomized quicksort: drive x
// to a random pivot's value by asking it until "=", compare every other
// index against x, and after each comparison ask the pivot once to move x
// back. A part whose values are lo.. holds exactly the values lo..lo+len-1.
use std.rand.{Rng}

let wire = Wire()
var rng = Rng(1918)

@!multitest
let n: 1..=2000 = read()
domain I = 1..=n
var a: [I]Int = 0
fn ask(i: I) -> Str = wire.ask("? {i}")
fn solve(items: List<I>, lo: Int):
    if items.is_empty: return
    let p = items[rng.int(0, items.len - 1)]
    var r = ask(p)
    while r != "=": r = ask(p)
    var less: List<I> = []
    var more: List<I> = []
    for j in items if j != p:
        if ask(j) == "<": less.push(j) else: more.push(j)
        r = ask(p)
    a[p] = lo + less.len
    solve(less, lo)
    solve(more, lo + less.len + 1)
solve([i for i in I], 1)
print("!", a)

// Hidden data per test: n and a seed for the permutation and the start x.
// No interactor-side read of the program exists, so `w.ask("")` stands in
// for one: the blank line it prints is skipped by the program's token read.
@interactor
fn judge(w: Wire):
    let n: 1..=12 = read()
    let seed: 0..=1e9 = read()
    var rng = Rng(seed)
    var a = List(1..=n)
    for i in (1..n).rev:
        let j = rng.int(0, i)
        a.swap(i, j)
    var x = rng.int(1, n)
    var asked = 0
    var op: Str = w.ask("{n}")
    while op == "?":
        let i: Int = w.ask("")
        asked += 1
        assert asked <= 40 * n and 1 <= i and i <= n
        let v = a[i - 1]
        var ans = "="
        if v > x:
            ans = ">"
            x += 1
        elif v < x:
            ans = "<"
            x -= 1
        op = w.ask(ans)
    assert op == "!"
    let got = [w.ask<Int>("") for _ in 0..n]
    assert zip(got, a).all((g, h) => g == h)
