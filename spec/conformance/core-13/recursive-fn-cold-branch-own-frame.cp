// expect: ok, output "576 6\n35"
// rule: 13.3.6
// A recursive fn's branch past its last recursive call runs in its own frame; its result,
// and the result of a fn whose loop precedes the call, are what they would be inline.
fn ends(n: Int) -> List<Int>:
    if n == 0: return [1]
    var res = ends(n - 1)
    res.push(0)
    if n % 2 == 0:
        for i in 1..res.len: res[i] += res[i - 1]
        res
    else:
        let w = n + 1
        res.map(e => e * w)
fn sum_to(n: Int) -> Int:
    if n == 0: return 0
    var s = 0
    for i in 0..n: s += i
    s + sum_to(n - 1)
let xs = ends(5)
print(xs.sum(), xs.len)
print(sum_to(6))
