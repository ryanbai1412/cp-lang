// `a > b` on a struct with its own `Ord` calls `lt` once, in the interpreter as in C++.
struct W(v: Int)
var calls = 0
impl Ord for W:
    fn lt(a, b):
        calls += 1
        a.v < b.v

let (a, b) = (W(1), W(2))
print(a < b, a > b, a <= b, a >= b, b < a, a < a)
print(calls)
