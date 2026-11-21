fn walk(n: Int, f: Int -> Int) -> Int:
    if n == 0: return 0
    f(n) + walk(n - 1, f)

fn even(n: Int, f: Int -> Int) -> Int:
    if n == 0: return f(0)
    odd(n - 1, f)

fn odd(n: Int, f: Int -> Int) -> Int:
    if n == 0: return f(1)
    even(n - 1, f)

fn wrap(n: Int, f: Int -> Int) -> Int:
    if n == 0: return f(0)
    wrap(n - 1, x => f(x) + 1)

fn visit(u: Int, kids: List<List<Int>>, mut seen: List<Int>, on: Int -> Int):
    seen.push(on(u))
    for v in kids[u]:
        visit(v, kids, mut seen, on)

let k: Int = read()
print(walk(10, x => x * k), walk(5, x => x + k))
print(even(7, x => x + k), odd(4, x => x * k))
print(wrap(3, x => x + k))
var seen: List<Int> = []
let kids = [[1, 2], [3], [], []]
visit(0, kids, mut seen, u => u * 10 + k)
visit(1, kids, mut seen, u => -u)
print(seen)
