// Each kind of VM call, nested deep: an int fn, a fn taking a fn value it
// calls, a fn lending a `mut` list on. They run on heap frames, not stack.
fn depth(n: Int) -> Int = if n == 0: 0 else: 1 + depth(n - 1)

fn walk(n: Int, f: Int -> Int) -> Int:
    if n == 0: return 0
    f(n) + walk(n - 1, f)

fn fill(n: Int, mut xs: List<Int>):
    if n > 0:
        xs.push(n)
        fill(n - 1, mut xs)

let n: Int = read()
print(depth(n))
print(walk(n, x => x % 2))
var xs: List<Int> = []
fill(n, mut xs)
print(xs.len, xs[0], xs[n - 1])
