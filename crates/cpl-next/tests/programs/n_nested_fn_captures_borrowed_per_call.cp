// expect: ok, output "8\n2\n9"
// rule: R05.10.3
// A nested fn holds only its captures' names and borrows them per call, so the enclosing fn
// may write a captured `var` between calls: an element write (std.graph's scc_times), a `pop`
// (std.search's split_best), and with a lent parameter captured beside them.
fn put_between(n: Int) -> Int:
    var ans = [0, 0, 0]
    fn put(i: Int):
        ans[i] = i
    put(1)
    ans[0] = 5
    put(2)
    ans[0] + ans[1] + ans[2]
fn pop_between(n: Int) -> Int:
    var heap = [3, 1, 2]
    fn better(a: Int, b: Int) -> Bool:
        heap[a] < heap[b]
    var s = 0
    if better(0, 1): s += 1
    let last = heap.pop()!
    if better(0, 1): s += 10
    s + last
fn with_lent(xs: List<Int>) -> Int:
    var ans = [0, 0, 0]
    fn solve(l: Int, r: Int):
        if l == r:
            ans[l] = xs[l]
            return
        solve(l, l)
        solve(r, r)
    solve(0, 2)
    ans[1] = 5
    ans[0] + ans[1] + ans[2]
print(put_between(3))
print(pop_between(3))
print(with_lent([1, 9, 3]))
