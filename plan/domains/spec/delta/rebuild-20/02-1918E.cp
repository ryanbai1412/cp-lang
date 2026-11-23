// CF 1918E ace5 and Task Order (interactive). Randomized quicksort: drive x
// to a random pivot's value by asking it until "=", compare every other
// index against x, and after each comparison ask the pivot once to move x
// back. A part whose values are lo.. holds exactly the values lo..lo+len-1.
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
solve(List(I), 1)
print("!", a)

@interactor
fn judge(w: Wire):
    let t = gen(1..=3)
    print(t)
    for _ in 0..t:
        let n = gen(1..=12)
        var a = List(1..=n)
        for i in (1..n).rev:
            let j = gen(0..=i)
            a.swap(i, j)
        var x = gen(1..=n)
        print(n)
        var asked = 0
        while true:
            let op: Str = read()
            if op == "!":
                let got = read<[n]Int>()
                assert zip(got, a).all((g, h) => g == h)
                break
            assert op == "?"
            let i: Int = read()
            asked += 1
            assert asked <= 40 * n and 1 <= i and i <= n
            let v = a[i - 1]
            if v > x:
                print(">")
                x += 1
            elif v < x:
                print("<")
                x -= 1
            else:
                print("=")
