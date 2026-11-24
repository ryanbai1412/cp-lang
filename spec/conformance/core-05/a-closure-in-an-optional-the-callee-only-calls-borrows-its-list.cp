// expect: ok, output "13"
// rule: 05.10.4
// A closure passed in `Some(..)` to a `?fn` parameter the callee only tests and calls lives only for the call, so it borrows the list it captures: no copy, and the parameter is a view of it, not a `std::function`.
fn total(n: Int, weight: ?(Int -> Int) = None) -> Int:
    var t = 0
    for i in 0..n:
        if let w = weight: t += w(i)
        else: t += 1
    t
fn go(xs: List<Int>) -> Int:
    var s = 0
    for weighted in [false, true]:
        s += total(3, weight: if weighted: Some((i: Int) => xs[i]) else: None)
    s
print(go([5, 2, 3]))
