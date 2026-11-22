// A fn nested in another may call itself, and change the variables of
// the fn around it: every call, however deep, sees the same `total`.
// A fn that calls itself and returns nothing needs no `->`.
fn solve():
    let n: Int = read()
    var g: List<List<Int>> = [[] for _ in 0..n]
    for _ in 0..n - 1:
        let (a, b): (Int, Int) = read()
        g[a].push(b)
        g[b].push(a)
    var total = 0
    fn dfs(u: Int, p: Int) -> Int:
        var size = 1
        for v in g[u]:
            if v != p: size += dfs(v, u)
        total += size
        return size
    print(dfs(0, -1), total)

    var order: List<Int> = []
    fn walk(u: Int, p: Int):
        order.push(u)
        for v in g[u]:
            if v != p: walk(v, u)
    walk(0, -1)
    print(order)

    // A nested fn that calls another nested fn, which calls itself.
    fn depth(u: Int, p: Int) -> Int = max(0, [depth(v, u) + 1 for v in g[u] if v != p].max() ?? 0)
    fn deepest() -> Int = depth(0, -1)
    print(deepest())

var seen: [0..16]Bool = false
fn mark(u: Int):
    seen[u] = true
    if u * 2 < 16:
        mark(u * 2)
        mark(u * 2 + 1)

solve()
mark(1)
print(count(i for i in 0..16 if seen[i]))
