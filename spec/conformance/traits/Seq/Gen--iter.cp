// expect: ok, output "0\n2\n4\n0\n2\n4"
// engines: next
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
fn walk<S: Seq<Int>>(s: S):
    for x in s: print(x)
let g = evens(5)
walk(g)
walk(g)
