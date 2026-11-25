// expect: ok, output "499500"
// A `let` heap value captured (docs: "a heap value is still copied"). Semantics must still hold.
fn go(n: Int) -> Int:
    var tmp: List<Int> = []
    for i in 0..n: tmp.push(i)
    let xs = tmp
    var s = 0
    for i in 0..n:
        let f = (j: Int) => xs[j]
        s += f(i)
    s
print(go(1000))
