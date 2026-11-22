fn tree_sizes(n: Int, par: List<Int>) -> (Int, Int):
    var kids: List<List<Int>> = [[] for _ in 0..n]
    for i in 1..n:
        kids[par[i - 1]].push(i)
    var visits = 0
    fn size(u: Int) -> Int:
        visits += 1
        var s = 1
        for v in kids[u]:
            s += size(v)
        s
    (size(0), visits)

fn collatz(limit: Int) -> Int:
    var best = 0
    let steps = (x: Int) => x
    fn walk(x: Int, d: Int) -> Int:
        if x == 1: return d
        if x % 2 == 0: return walk(x / 2, d + 1)
        walk(3 * x + 1, d + 1)
    for x in 1..=limit:
        best = max(best, steps(walk(x, 0)))
    best

let n: 1..=100 = read()
let par: List<Int> = [read() for _ in 1..n]
print(tree_sizes(n, par))
print(collatz(30))
